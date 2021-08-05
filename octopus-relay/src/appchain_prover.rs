use near_sdk::log;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainProver;

impl AppchainProver {
	pub fn verify(
		&self,
		encoded_messages: Vec<u8>,
		header_partial: Vec<u8>,
		leaf_proof: Vec<u8>,
		mmr_root: Vec<u8>,
	) -> bool {
		log!("in appchain prover");
		true
	}
}