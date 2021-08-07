use crate::types::{BurnAssetPayload, LockPayload, Message, MessagePayload, PayloadType};
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
	payload_type: PayloadType,
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
			.map(|m| match m.payload_type {
				PayloadType::BurnAsset => {
					let payload_result: Result<BurnAssetPayload, std::io::Error> =
						BorshDeserialize::deserialize(&mut &m.payload[..]);
					let payload = payload_result.unwrap();
					log!("in appchain payload {:?}", payload);
					Message {
						nonce: m.nonce,
						payload: MessagePayload::BurnAsset(payload),
					}
				}
				PayloadType::Lock => {
					let payload_result: Result<LockPayload, std::io::Error> =
						BorshDeserialize::deserialize(&mut &m.payload[..]);
					let payload = payload_result.unwrap();
					log!("in appchain payload {:?}", payload);
					Message {
						nonce: m.nonce,
						payload: MessagePayload::Lock(payload),
					}
				}
			})
			.collect()
	}
}
