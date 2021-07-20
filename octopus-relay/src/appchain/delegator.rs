/// Appchain delegator of an appchain validator
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, Balance, BlockHeight};

use crate::types::Delegator;
use crate::DelegatorId;

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
