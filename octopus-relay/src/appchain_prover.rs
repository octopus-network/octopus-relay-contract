use near_sdk::log;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use sp_core::H256;
use sp_runtime::generic::{Digest, DigestItem, Header};
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::traits::{Hash, Keccak256};
use sp_std::marker;

// TODO
use pallet_mmr_primitives::{DataOrHash, FullLeaf, Proof};

/// A node stored in the MMR.
pub type Node<H, L> = DataOrHash<H, L>;

/// Default Merging & Hashing behavior for MMR.
pub struct Hasher<H, L>(marker::PhantomData<(H, L)>);

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

pub struct HeaderPartial {
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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainProver;

impl AppchainProver {
	pub fn verify(
		&self,
		encoded_messages: Vec<u8>,
		header_partial: HeaderPartial,
		leaf_proof: Proof<H256>,
		mmr_root: H256,
	) -> bool {
		let commitment = Keccak256::hash(&encoded_messages);
		log!("commitment: {:?}", commitment);

		if let Some(other_data) = header_partial.digest.log(DigestItem::as_other) {
			if other_data != commitment.as_bytes() {
				log!("inconsistent commitment with header: {:?}", other_data);
				return false;
			}
		} else {
			log!("there is no commitment in header");
			return false;
		}

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

		is_valid.unwrap()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use beefy_primitives::BEEFY_ENGINE_ID;
	use near_sdk::test_utils::{accounts, VMContextBuilder};
	use near_sdk::{testing_env, MockedBlockchain};
	use sp_consensus_babe::BABE_ENGINE_ID;
	use std::str::FromStr;

	#[test]
	fn test_message_is_valid() {
		let mut context = VMContextBuilder::new();
		testing_env!(context.predecessor_account_id(accounts(1)).build());
		let messages = vec![
			4, 4, 0, 0, 0, 0, 0, 0, 0, 5, 2, 19, 0, 0, 0, 116, 101, 115, 116, 45, 115, 116, 97, 98,
			108, 101, 46, 116, 101, 115, 116, 110, 101, 116, 66, 0, 0, 0, 48, 120, 57, 48, 98, 53,
			97, 98, 50, 48, 53, 99, 54, 57, 55, 52, 99, 57, 101, 97, 56, 52, 49, 98, 101, 54, 56,
			56, 56, 54, 52, 54, 51, 51, 100, 99, 57, 99, 97, 56, 97, 51, 53, 55, 56, 52, 51, 101,
			101, 97, 99, 102, 50, 51, 49, 52, 54, 52, 57, 57, 54, 53, 102, 101, 50, 50, 16, 0, 0,
			0, 121, 117, 97, 110, 99, 104, 97, 111, 46, 116, 101, 115, 116, 110, 101, 116, 123, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		];

		let item1 = DigestItem::PreRuntime(
			BABE_ENGINE_ID,
			vec![2, 0, 0, 0, 0, 81, 209, 78, 32, 0, 0, 0, 0],
		);
		let item2 = DigestItem::Other(vec![
			137, 91, 155, 100, 191, 96, 26, 122, 92, 72, 177, 210, 6, 117, 122, 51, 203, 228, 79,
			254, 209, 242, 141, 36, 92, 199, 136, 213, 122, 139, 223, 154,
		]);
		let item3 = DigestItem::Consensus(
			BEEFY_ENGINE_ID,
			vec![
				3, 144, 46, 233, 40, 162, 160, 32, 201, 229, 165, 123, 174, 198, 119, 207, 204,
				173, 135, 219, 226, 208, 215, 144, 124, 31, 128, 95, 68, 116, 85, 161, 20,
			],
		);
		let item4 = DigestItem::Seal(
			BABE_ENGINE_ID,
			vec![
				116, 91, 239, 200, 27, 61, 19, 75, 29, 172, 210, 132, 159, 99, 239, 51, 34, 244,
				254, 90, 62, 9, 151, 9, 173, 220, 33, 64, 173, 192, 236, 127, 43, 199, 171, 150,
				68, 170, 45, 45, 24, 7, 223, 42, 104, 120, 12, 208, 49, 185, 81, 112, 180, 236,
				183, 43, 175, 164, 220, 57, 17, 44, 246, 133,
			],
		);
		let digest = Digest { logs: vec![item1, item2, item3, item4] };
		let header_partial = HeaderPartial {
			parent_hash: H256::from_str(
				"0xc867e11411697f671ef8eba08efbae00ff3fc4c42a3a2b38545d3d373df41e43",
			)
			.unwrap(),
			number: 491,
			state_root: H256::from_str(
				"0xcee39d594ddb857a7f7140c37529ccf1119d58591c70e732391f38e9ecc9faf1",
			)
			.unwrap(),
			extrinsics_root: H256::from_str(
				"0x8f45a786d3ec2d55f84a897a5e4e9b61e55dab9461cc062d5bc7916a583d8ece",
			)
			.unwrap(),
			digest,
		};
		let leaf_proof = Proof::<H256> {
			leaf_index: 491,
			leaf_count: 492,
			items: vec![
				H256::from_str(
					"0x7644565c6c0f650df505c2a9fd4055dba4efd478c7999f63d458037ec4377ba8",
				)
				.unwrap(),
				H256::from_str(
					"0xd17071ac61cc95859229f9c47b710858d030341e8359a108b9774a3544356c81",
				)
				.unwrap(),
				H256::from_str(
					"0x01dd1b1721e0e226c84fc6ea8284972a5bc563f1e5e4853e0a8ac9bdc6c7f6b1",
				)
				.unwrap(),
				H256::from_str(
					"0x4843f4a775c2dbd4777f582779a97f0d46dcda23509b0e6baaf70a74c7616ad3",
				)
				.unwrap(),
				H256::from_str(
					"0xc22835ff91099f0908f94b3aeb8e2ccb06489d935f138e7c112409363036b2f4",
				)
				.unwrap(),
				H256::from_str(
					"0x1d3fda67cd44ac0c922e90e1235799eb4a2f4b5f65433fa2ce68c15db68783a6",
				)
				.unwrap(),
				H256::from_str(
					"0x5a19debfa7b4ebbe7d72737c815dc05fb6131d59c6110a99bdf76b052518f812",
				)
				.unwrap(),
			],
		};

		let mmr_root =
			H256::from_str("0x403610894a01ee6a72d6128afe96b483b848cf84f21697438c2fa12a6dfd4381")
				.unwrap();

		let ap = AppchainProver;
		assert_eq!(ap.verify(messages, header_partial, leaf_proof, mmr_root), true);
	}
}
