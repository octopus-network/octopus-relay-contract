/// Appchain delegator of an appchain validator
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, Vector};
use near_sdk::{AccountId, Balance, BlockHeight};

use crate::types::{Delegator, DelegatorId, DelegatorIndex};

pub type ValidatorHistoryList = Vector<LazyOption<DelegatorHistory>>;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DelegatorHistoryIndexSet {
    pub set_id: u32,
    // Use LookupMap instead of Vector to save gas.
    pub indexes: Vec<DelegatorIndex>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DelegatorHistory {
    pub delegator_id: DelegatorId,
    pub account_id: AccountId,
    pub amount: Balance,
    pub block_height: BlockHeight,
    pub set_id: u32,
}

pub type DelegatorHistoryList = Vector<LazyOption<DelegatorHistory>>;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainDelegator {
    /// Id of appchain delegator
    pub delegator_id: DelegatorId,
    /// Account id of the delegator
    pub account_id: AccountId,
    /// Authorized balance of the delegator
    pub amount: Balance,
    /// Block height of the delegation of the delegator
    pub block_height: BlockHeight,
}

impl AppchainDelegator {
    /// Convert to struct `Delegator`
    pub fn to_delegator(&self) -> Delegator {
        Delegator {
            id: self.delegator_id.clone(),
            account_id: self.account_id.clone(),
            amount: self.amount.into(),
            block_height: self.block_height,
        }
    }
}
