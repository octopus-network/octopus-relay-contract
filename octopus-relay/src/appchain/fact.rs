use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, Vector};
use near_sdk::json_types::U128;
use near_sdk::{AccountId, BlockHeight, Timestamp};

use crate::types::{Fact, Locked, ValidatorSet};
use crate::SeqNum;

use super::validator::AppchainValidator;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainValidatorSet {
    pub sequence_number: SeqNum,
    pub set_id: u32,
    pub validators: Vector<LazyOption<AppchainValidator>>,
    pub timestamp: Timestamp,
    pub epoch_number: u32,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainLockedToken {
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
pub enum AppchainFact {
    UpdateValidatorSet(AppchainValidatorSet),
    LockToken(AppchainLockedToken),
}

impl AppchainValidatorSet {
    ///
    pub fn to_validator_set(&self) -> ValidatorSet {
        ValidatorSet {
            seq_num: self.sequence_number,
            set_id: self.set_id,
            validators: self
                .validators
                .iter()
                .map(|v| v.get().unwrap().to_lite_validator())
                .collect::<Vec<_>>(),
        }
    }
}

impl AppchainLockedToken {
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

impl AppchainFact {
    ///
    pub fn to_fact(&self) -> Fact {
        match self {
            AppchainFact::UpdateValidatorSet(appchain_validator_set) => {
                Fact::UpdateValidatorSet(appchain_validator_set.to_validator_set())
            }
            AppchainFact::LockToken(appchain_locked_token) => {
                Fact::LockToken(appchain_locked_token.to_locked())
            }
        }
    }
    /// Clear extra storage used by the fact
    ///
    /// **This function must be called before remove `AppchainFact` from storage**
    pub fn clear_extra_storage(&self) {
        match self {
            AppchainFact::UpdateValidatorSet(appchain_validator_set) => {
                appchain_validator_set.validators.iter().for_each(|mut v| {
                    v.remove();
                });
            }
            _ => (),
        }
    }
}
