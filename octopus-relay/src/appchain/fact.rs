use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, Vector};
use near_sdk::json_types::U128;
use near_sdk::{AccountId, BlockHeight, Timestamp};

use crate::types::{Burned, Fact, Locked, SeqNum, ValidatorSet};

use super::validator::{AppchainValidator, ValidatorHistoryIndexSet};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum RawFact {
    ValidatorHistoryIndexSet(ValidatorHistoryIndexSet),
    LockAsset(Locked),
    Burn(Burned),
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainLockedAsset {
    pub sequence_number: SeqNum,
    pub token_id: AccountId,
    pub sender_id: AccountId,
    pub receiver: String,
    pub amount: U128,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub epoch_number: u32,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainBurnedNativeToken {
    pub sequence_number: SeqNum,
    pub sender_id: AccountId,
    pub receiver: String,
    pub amount: U128,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub epoch_number: u32,
}

impl AppchainLockedAsset {
    ///
    pub fn to_locked(&self) -> Locked {
        Locked {
            seq_num: self.sequence_number,
            token_id: self.token_id.clone(),
            sender_id: self.sender_id.clone(),
            receiver: self.receiver.clone(),
            amount: self.amount,
        }
    }
}

impl AppchainBurnedNativeToken {
    ///
    pub fn to_burned(&self) -> Burned {
        Burned {
            seq_num: self.sequence_number,
            sender_id: self.sender_id.clone(),
            receiver: self.receiver.clone(),
            amount: self.amount,
        }
    }
}
