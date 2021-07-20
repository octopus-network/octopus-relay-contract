use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap};
use near_sdk::{AccountId, Balance, BlockHeight};

use crate::types::{LiteValidator, Validator};
use crate::{DelegatorId, ValidatorId};

use super::delegator::AppchainDelegator;

const INVALID_DELEGATORS_DATA_OF_VALIDATOR: &'static str = "Invalid delegators data of validator";

/// Appchain validator of an appchain
#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainValidator {
    /// Id of appchain validator
    pub validator_id: ValidatorId,
    /// Account id of the validator
    pub account_id: AccountId,
    /// Staked balance of the validator
    pub amount: Balance,
    /// Block height which the validator started staking
    pub block_height: BlockHeight,
    /// Delegators of the validator
    pub delegators: UnorderedMap<DelegatorId, LazyOption<AppchainDelegator>>,
}

impl AppchainValidator {
    /// Convert to struct `Validator`
    pub fn to_validator(&self) -> Validator {
        Validator {
            id: self.validator_id.clone(),
            account_id: self.account_id.clone(),
            staked_amount: self.amount.into(),
            block_height: self.block_height,
            delegators: self
                .delegators
                .values_as_vector()
                .iter()
                .map(|d| {
                    d.get()
                        .expect(INVALID_DELEGATORS_DATA_OF_VALIDATOR)
                        .to_delegator()
                })
                .collect(),
        }
    }
    /// Convert to struct `LiteValidator`
    pub fn to_lite_validator(&self) -> LiteValidator {
        LiteValidator {
            id: self.validator_id.clone(),
            account_id: self.account_id.clone(),
            weight: self.amount.into(),
            block_height: self.block_height,
            delegators: self
                .delegators
                .values_as_vector()
                .iter()
                .map(|d| {
                    d.get()
                        .expect(INVALID_DELEGATORS_DATA_OF_VALIDATOR)
                        .to_delegator()
                })
                .collect(),
        }
    }
    /// Get delegator by `DelegatorId`
    pub fn get_delegator(&self, delegator_id: &DelegatorId) -> Option<AppchainDelegator> {
        if let Some(appchain_delegator_option) = self.delegators.get(delegator_id) {
            return appchain_delegator_option.get();
        }
        Option::None
    }
    /// Get total staked amount of OCT tokens of the validator,
    /// this function will also count all balances of delegators.
    pub fn get_staked_balance_including_delegators(&self) -> Balance {
        self.amount
            + self
                .delegators
                .values_as_vector()
                .iter()
                .filter(|d| d.is_some())
                .map(|d| d.get().unwrap().amount)
                .sum::<u128>()
    }
    /// Clear extra storage used by the validator
    ///
    /// **This function must be called before remove `AppchainValidator` from storage**
    pub fn clear_extra_storage(&self) {
        self.delegators.values_as_vector().iter().for_each(|mut d| {
            d.remove();
        });
    }
}
