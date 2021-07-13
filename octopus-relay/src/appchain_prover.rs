use near_sdk::log;

use sp_core::H256;
use sp_runtime::generic::{Digest, DigestItem, Header};
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::traits::{Hash, Keccak256};

// TODO
use pallet_mmr_primitives::{DataOrHash, FullLeaf, Proof};

/// A node stored in the MMR.
pub type Node<H, L> = DataOrHash<H, L>;

/// Default Merging & Hashing behavior for MMR.
pub struct Hasher<H, L>(sp_std::marker::PhantomData<(H, L)>);

impl<H: sp_runtime::traits::Hash, L: FullLeaf> mmr_lib::Merge for Hasher<H, L> {
	type Item = Node<H, L>;

	fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
		let mut concat = left.hash().as_ref().to_vec();
		concat.extend_from_slice(right.hash().as_ref());

		Node::Hash(<H as sp_runtime::traits::Hash>::hash(&concat))
	}
}

/// MMR nodes & size -related utilities.
pub struct NodesUtils {
	no_of_leaves: u64,
}

impl NodesUtils {
	/// Create new instance of MMR nodes utilities for given number of leaves.
	pub fn new(no_of_leaves: u64) -> Self {
		Self { no_of_leaves }
	}

	/// Calculate number of peaks in the MMR.
	pub fn number_of_peaks(&self) -> u64 {
		self.number_of_leaves().count_ones() as u64
	}

	/// Return the number of leaves in the MMR.
	pub fn number_of_leaves(&self) -> u64 {
		self.no_of_leaves
	}

	/// Calculate the total size of MMR (number of nodes).
	pub fn size(&self) -> u64 {
		2 * self.no_of_leaves - self.number_of_peaks()
	}

	/// Calculate maximal depth of the MMR.
	pub fn depth(&self) -> u32 {
		if self.no_of_leaves == 0 {
			return 0;
		}

		64 - self.no_of_leaves.next_power_of_two().leading_zeros()
	}
}

/// Stateless verification of the leaf proof.
pub fn verify_leaf_proof<H, L>(
	root: H::Output,
	leaf: Node<H, L>,
	proof: Proof<H::Output>,
) -> Result<bool, pallet_mmr_primitives::Error>
where
	H: sp_runtime::traits::Hash,
	L: FullLeaf,
{
	let size = NodesUtils::new(proof.leaf_count).size();
	let leaf_position = mmr_lib::leaf_index_to_pos(proof.leaf_index);

	let p = mmr_lib::MerkleProof::<Node<H, L>, Hasher<H, L>>::new(
		size,
		proof.items.into_iter().map(Node::Hash).collect(),
	);
	p.verify(Node::Hash(root), vec![(leaf_position, leaf)])
		.map_err(|e| pallet_mmr_primitives::Error::Verify.log_debug(e))
}

struct HeaderPartial {
	/// The parent hash.
	parent_hash: H256,
	/// The block number.
	number: u32,
	/// The state trie merkle root
	state_root: H256,
	/// The merkle root of the extrinsics.
	extrinsics_root: H256,
	/// A chain-specific digest of data useful for light clients or referencing auxiliary data.
	digest: Digest<H256>,
}

struct AppchainProver {
	method_name: String,
}

impl AppchainProver {
	pub fn verify(
		&self,
		encoded_messages: Vec<u8>,
		mut header_partial: HeaderPartial,
		leaf_proof: Proof<H256>,
		mmr_root: H256,
	) -> bool {
		let commitment = Keccak256::hash(&encoded_messages);
		log!("commitment: {:?}", commitment);

		// TODO: strict order?
		header_partial.digest.push(DigestItem::Other(commitment.as_bytes().to_vec()));

		let header = Header::<u32, BlakeTwo256> {
			parent_hash: header_partial.parent_hash,
			number: header_partial.number,
			state_root: header_partial.state_root,
			extrinsics_root: header_partial.extrinsics_root,
			digest: header_partial.digest,
		};
		let header_hash = header.hash();
		log!("header_hash: {:?}", header_hash);

		let leaf: Node<Keccak256, (u32, H256)> =
			DataOrHash::Data((header_partial.number, header_hash));
		let is_valid = verify_leaf_proof::<Keccak256, (u32, H256)>(mmr_root, leaf, leaf_proof);
		if let Err(error) = is_valid {
			log!("failed to verify_leaf_proof: {:?}", error);
			return false;
		}

		true
	}
}
