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

#[cfg(test)]
mod tests {
	use super::*;
	use beefy_primitives::BEEFY_ENGINE_ID;
	use sp_consensus_babe::BABE_ENGINE_ID;
	use std::str::FromStr;

	// new finalized header Header { parent_hash: 0xc867e11411697f671ef8eba08efbae00ff3fc4c42a3a2b38545d3d373df41e43, number: 491, state_root: 0xcee39d594ddb857a7f7140c37529ccf1119d58591c70e732391f38e9ecc9faf1, extrinsics_root: 0x8f45a786d3ec2d55f84a897a5e4e9b61e55dab9461cc062d5bc7916a583d8ece, digest: Digest { logs: [DigestItem::PreRuntime([66, 65, 66, 69], [2, 0, 0, 0, 0, 81, 209, 78, 32, 0, 0, 0, 0]), DigestItem::Other([137, 91, 155, 100, 191, 96, 26, 122, 92, 72, 177, 210, 6, 117, 122, 51, 203, 228, 79, 254, 209, 242, 141, 36, 92, 199, 136, 213, 122, 139, 223, 154]), DigestItem::Consensus([66, 69, 69, 70], [3, 144, 46, 233, 40, 162, 160, 32, 201, 229, 165, 123, 174, 198, 119, 207, 204, 173, 135, 219, 226, 208, 215, 144, 124, 31, 128, 95, 68, 116, 85, 161, 20]), DigestItem::Seal([66, 65, 66, 69], [116, 91, 239, 200, 27, 61, 19, 75, 29, 172, 210, 132, 159, 99, 239, 51, 34, 244, 254, 90, 62, 9, 151, 9, 173, 220, 33, 64, 173, 192, 236, 127, 43, 199, 171, 150, 68, 170, 45, 45, 24, 7, 223, 42, 104, 120, 12, 208, 49, 185, 81, 112, 180, 236, 183, 43, 175, 164, 220, 57, 17, 44, 246, 133])] } }
	// commitment: "895b9b64bf601a7a5c48b1d206757a33cbe44ffed1f28d245cc788d57a8bdf9a"
	// data: Bytes([4, 4, 0, 0, 0, 0, 0, 0, 0, 5, 2, 19, 0, 0, 0, 116, 101, 115, 116, 45, 115, 116, 97, 98, 108, 101, 46, 116, 101, 115, 116, 110, 101, 116, 66, 0, 0, 0, 48, 120, 57, 48, 98, 53, 97, 98, 50, 48, 53, 99, 54, 57, 55, 52, 99, 57, 101, 97, 56, 52, 49, 98, 101, 54, 56, 56, 56, 54, 52, 54, 51, 51, 100, 99, 57, 99, 97, 56, 97, 51, 53, 55, 56, 52, 51, 101, 101, 97, 99, 102, 50, 51, 49, 52, 54, 52, 57, 57, 54, 53, 102, 101, 50, 50, 16, 0, 0, 0, 121, 117, 97, 110, 99, 104, 97, 111, 46, 116, 101, 115, 116, 110, 101, 116, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
	// hash: 0x895b9b64bf601a7a5c48b1d206757a33cbe44ffed1f28d245cc788d57a8bdf9a
	// messages: [Message { nonce: 4, payload: [19, 0, 0, 0, 116, 101, 115, 116, 45, 115, 116, 97, 98, 108, 101, 46, 116, 101, 115, 116, 110, 101, 116, 66, 0, 0, 0, 48, 120, 57, 48, 98, 53, 97, 98, 50, 48, 53, 99, 54, 57, 55, 52, 99, 57, 101, 97, 56, 52, 49, 98, 101, 54, 56, 56, 56, 54, 52, 54, 51, 51, 100, 99, 57, 99, 97, 56, 97, 51, 53, 55, 56, 52, 51, 101, 101, 97, 99, 102, 50, 51, 49, 52, 54, 52, 57, 57, 54, 53, 102, 101, 50, 50, 16, 0, 0, 0, 121, 117, 97, 110, 99, 104, 97, 111, 46, 116, 101, 115, 116, 110, 101, 116, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }]
	// decoded_message: XTransferPayload { token_id: [116, 101, 115, 116, 45, 115, 116, 97, 98, 108, 101, 46, 116, 101, 115, 116, 110, 101, 116], sender: [48, 120, 57, 48, 98, 53, 97, 98, 50, 48, 53, 99, 54, 57, 55, 52, 99, 57, 101, 97, 56, 52, 49, 98, 101, 54, 56, 56, 56, 54, 52, 54, 51, 51, 100, 99, 57, 99, 97, 56, 97, 51, 53, 55, 56, 52, 51, 101, 101, 97, 99, 102, 50, 51, 49, 52, 54, 52, 57, 57, 54, 53, 102, 101, 50, 50], receiver_id: [121, 117, 97, 110, 99, 104, 97, 111, 46, 116, 101, 115, 116, 110, 101, 116], amount: 123 }
	// header hash:0xed28ce8a10a46916f3946a5dcd179d2c606c11753e7e4f78724782e83bea056f
	//
	//
	//
	// new finalized header Header { parent_hash: 0xed28ce8a10a46916f3946a5dcd179d2c606c11753e7e4f78724782e83bea056f, number: 492, state_root: 0x32c19eef3af60cdd784d11acaed7758fc0cab7bf33a228dff108f972b286cfd8, extrinsics_root: 0xad820064ceb4e65e1a161720e7cff911df0e8aec58f002345a1345d8372a2729, digest: Digest { logs: [DigestItem::PreRuntime([66, 65, 66, 69], [2, 0, 0, 0, 0, 82, 209, 78, 32, 0, 0, 0, 0]), DigestItem::Consensus([66, 69, 69, 70], [3, 64, 54, 16, 137, 74, 1, 238, 106, 114, 214, 18, 138, 254, 150, 180, 131, 184, 72, 207, 132, 242, 22, 151, 67, 140, 47, 161, 42, 109, 253, 67, 129]), DigestItem::Seal([66, 65, 66, 69], [16, 131, 25, 240, 7, 169, 28, 217, 253, 24, 196, 121, 160, 254, 184, 142, 74, 235, 146, 16, 103, 97, 160, 92, 204, 187, 11, 217, 198, 136, 188, 108, 28, 139, 50, 136, 14, 144, 193, 142, 62, 164, 31, 175, 119, 247, 194, 38, 76, 28, 218, 58, 68, 164, 188, 122, 45, 39, 184, 91, 212, 186, 157, 131])] } }
	// proof: LeafProof { block_hash: 0x2ff6a8ed872f2d07243c6e5c5eab99df125e5f9513add75763eaff9996f30bb3, leaf: Bytes([144, 235, 1, 0, 0, 237, 40, 206, 138, 16, 164, 105, 22, 243, 148, 106, 93, 205, 23, 157, 44, 96, 108, 17, 117, 62, 126, 79, 120, 114, 71, 130, 232, 59, 234, 5, 111]), proof: Bytes([235, 1, 0, 0, 0, 0, 0, 0, 236, 1, 0, 0, 0, 0, 0, 0, 28, 118, 68, 86, 92, 108, 15, 101, 13, 245, 5, 194, 169, 253, 64, 85, 219, 164, 239, 212, 120, 199, 153, 159, 99, 212, 88, 3, 126, 196, 55, 123, 168, 209, 112, 113, 172, 97, 204, 149, 133, 146, 41, 249, 196, 123, 113, 8, 88, 208, 48, 52, 30, 131, 89, 161, 8, 185, 119, 74, 53, 68, 53, 108, 129, 1, 221, 27, 23, 33, 224, 226, 38, 200, 79, 198, 234, 130, 132, 151, 42, 91, 197, 99, 241, 229, 228, 133, 62, 10, 138, 201, 189, 198, 199, 246, 177, 72, 67, 244, 167, 117, 194, 219, 212, 119, 127, 88, 39, 121, 169, 127, 13, 70, 220, 218, 35, 80, 155, 14, 107, 170, 247, 10, 116, 199, 97, 106, 211, 194, 40, 53, 255, 145, 9, 159, 9, 8, 249, 75, 58, 235, 142, 44, 203, 6, 72, 157, 147, 95, 19, 142, 124, 17, 36, 9, 54, 48, 54, 178, 244, 29, 63, 218, 103, 205, 68, 172, 12, 146, 46, 144, 225, 35, 87, 153, 235, 74, 47, 75, 95, 101, 67, 63, 162, 206, 104, 193, 93, 182, 135, 131, 166, 90, 25, 222, 191, 167, 180, 235, 190, 125, 114, 115, 124, 129, 93, 192, 95, 182, 19, 29, 89, 198, 17, 10, 153, 189, 247, 107, 5, 37, 24, 248, 18]) }
	// root_hash: 0x403610894a01ee6a72d6128afe96b483b848cf84f21697438c2fa12a6dfd4381
	// leaf: DataOrHash::Data((491, 0xed28ce8a10a46916f3946a5dcd179d2c606c11753e7e4f78724782e83bea056f))
	// leaf_proof: Proof { leaf_index: 491, leaf_count: 492, items: [0x7644565c6c0f650df505c2a9fd4055dba4efd478c7999f63d458037ec4377ba8, 0xd17071ac61cc95859229f9c47b710858d030341e8359a108b9774a3544356c81, 0x01dd1b1721e0e226c84fc6ea8284972a5bc563f1e5e4853e0a8ac9bdc6c7f6b1, 0x4843f4a775c2dbd4777f582779a97f0d46dcda23509b0e6baaf70a74c7616ad3, 0xc22835ff91099f0908f94b3aeb8e2ccb06489d935f138e7c112409363036b2f4, 0x1d3fda67cd44ac0c922e90e1235799eb4a2f4b5f65433fa2ce68c15db68783a6, 0x5a19debfa7b4ebbe7d72737c815dc05fb6131d59c6110a99bdf76b052518f812] }
	// is_valid: Ok(true)

	// 1. 忽略mmr_root的同步和验证，即认为后面获取的mmr_root是正确的, root_hash: 0x403610894a01ee6a72d6128afe96b483b848cf84f21697438c2fa12a6dfd4381
	// 2. block_height: 491中发现跨链消息, 取出491header中的commiment为895b9b64bf601a7a5c48b1d206757a33cbe44ffed1f28d245cc788d57a8bdf9a, 计算跨链消息和这个commitment相同，于是消息属于这个header
	// 3. 计算header的hash， 备用. hash is 0xed28ce8a10a46916f3946a5dcd179d2c606c11753e7e4f78724782e83bea056f
	// 4. 区块492敲定后，获取mmr_tree中491块的mmr_proof
	// 5. 验证proof通过，proof的内容为leaf: DataOrHash::Data((491, 0xed28ce8a10a46916f3946a5dcd179d2c606c11753e7e4f78724782e83bea056f)), 于是appchain中确定存在高度为491，hash为0xed28ce8a10a46916f3946a5dcd179d2c606c11753e7e4f78724782e83bea056f 的区块
	// 6. 验证之前计算的header hash与proof的相等，于是header正确，跨链消息正确，证毕
	#[test]
	fn test_message_is_valid() {
		// [
		// DigestItem::PreRuntime([66, 65, 66, 69], [2, 0, 0, 0, 0, 81, 209, 78, 32, 0, 0, 0, 0]),
		// DigestItem::Other([137, 91, 155, 100, 191, 96, 26, 122, 92, 72, 177, 210, 6, 117, 122, 51, 203, 228, 79, 254, 209, 242, 141, 36, 92, 199, 136, 213, 122, 139, 223, 154]),
		// DigestItem::Consensus([66, 69, 69, 70], [3, 144, 46, 233, 40, 162, 160, 32, 201, 229, 165, 123, 174, 198, 119, 207, 204, 173, 135, 219, 226, 208, 215, 144, 124, 31, 128, 95, 68, 116, 85, 161, 20]),
		// DigestItem::Seal([66, 65, 66, 69], [116, 91, 239, 200, 27, 61, 19, 75, 29, 172, 210, 132, 159, 99, 239, 51, 34, 244, 254, 90, 62, 9, 151, 9, 173, 220, 33, 64, 173, 192, 236, 127, 43, 199, 171, 150, 68, 170, 45, 45, 24, 7, 223, 42, 104, 120, 12, 208, 49, 185, 81, 112, 180, 236, 183, 43, 175, 164, 220, 57, 17, 44, 246, 133])
		// ]

		let message = [
			4, 4, 0, 0, 0, 0, 0, 0, 0, 5, 2, 19, 0, 0, 0, 116, 101, 115, 116, 45, 115, 116, 97, 98,
			108, 101, 46, 116, 101, 115, 116, 110, 101, 116, 66, 0, 0, 0, 48, 120, 57, 48, 98, 53,
			97, 98, 50, 48, 53, 99, 54, 57, 55, 52, 99, 57, 101, 97, 56, 52, 49, 98, 101, 54, 56,
			56, 56, 54, 52, 54, 51, 51, 100, 99, 57, 99, 97, 56, 97, 51, 53, 55, 56, 52, 51, 101,
			101, 97, 99, 102, 50, 51, 49, 52, 54, 52, 57, 57, 54, 53, 102, 101, 50, 50, 16, 0, 0,
			0, 121, 117, 97, 110, 99, 104, 97, 111, 46, 116, 101, 115, 116, 110, 101, 116, 123, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		];
		let commitment = Keccak256::hash(&message);
		let item1 = DigestItem::PreRuntime(
			BABE_ENGINE_ID,
			vec![2, 0, 0, 0, 0, 81, 209, 78, 32, 0, 0, 0, 0],
		);
		let item2 = DigestItem::Consensus(
			BEEFY_ENGINE_ID,
			vec![
				3, 144, 46, 233, 40, 162, 160, 32, 201, 229, 165, 123, 174, 198, 119, 207, 204,
				173, 135, 219, 226, 208, 215, 144, 124, 31, 128, 95, 68, 116, 85, 161, 20,
			],
		);
		// let item3 = DigestItem::Other(commitment.as_bytes().to_vec());
		let item4 = DigestItem::Seal(
			BABE_ENGINE_ID,
			vec![
				116, 91, 239, 200, 27, 61, 19, 75, 29, 172, 210, 132, 159, 99, 239, 51, 34, 244,
				254, 90, 62, 9, 151, 9, 173, 220, 33, 64, 173, 192, 236, 127, 43, 199, 171, 150,
				68, 170, 45, 45, 24, 7, 223, 42, 104, 120, 12, 208, 49, 185, 81, 112, 180, 236,
				183, 43, 175, 164, 220, 57, 17, 44, 246, 133,
			],
		);
		let digest = Digest::new(&[item1, item2, item4]);
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

		let ap = AppchainProver { method_name: "foo".to_string() };
		assert_eq!(ap.verify(messages, header_partial, leaf_proof, mmr_root), true);
	}
}
