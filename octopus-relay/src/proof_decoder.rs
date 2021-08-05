use crate::types::{Excecution, Message};
use crate::*;
use codec::{Decode, Encode, Input};

pub trait ProofDecoder {
	fn decode(
		&self,
		encoded_messages: Vec<u8>,
		header_partial: Vec<u8>,
		leaf_proof: Vec<u8>,
		mmr_root: Vec<u8>,
	) -> Vec<Message>;
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct RawMessage {
	nonce: u64,
	payload: Vec<u8>,
}

impl ProofDecoder for OctopusRelay {
	fn decode(
		&self,
		encoded_messages: Vec<u8>,
		header_partial: Vec<u8>,
		leaf_proof: Vec<u8>,
		mmr_root: Vec<u8>,
	) -> Vec<Message> {
		let decoded_messages: Vec<RawMessage> = Decode::decode(&mut &encoded_messages[..]).unwrap();
		log!("in appchain message {:?}", decoded_messages);

		decoded_messages
			.iter()
			.map(|m| {
				let payload_result: Result<Excecution, std::io::Error> =
					BorshDeserialize::deserialize(&mut &m.payload[..]);
				let excecution = payload_result.unwrap();
				log!("in appchain payload {:?}", excecution);
				Message {
					nonce: m.nonce,
					excecution,
				}
			})
			.collect()
	}
}
