use std::convert::TryInto;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::{env, AccountId, Balance, BlockHeight, Timestamp};

use crate::types::{AppchainStatus, Delegator, LiteValidator, Validator, ValidatorSet};
use crate::{AppchainId, DelegatorId, ValidatorId, VALIDATOR_SET_CYCLE};

/// Appchain delegator of an appchain validator
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
    pub delegators: UnorderedMap<DelegatorId, AppchainDelegator>,
}

/// Appchain state of an appchain of Octopus Network
#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainState {
    /// Id of the appchain
    pub id: AppchainId,
    /// Validators collection of the appchain
    pub validators: UnorderedMap<ValidatorId, AppchainValidator>,
    /// Nonce of validator set of the appchain.
    ///
    /// This nonce will be increased by 1 for each staking action to the appchain,
    /// the staking action includes a new validator stakes some OCT tokens or
    /// an existed validator changes its staking balance.
    pub validators_nonce: u32,
    /// Last update time of validator set of the appchain, will be updated for each staking action
    pub validators_timestamp: Timestamp,
    /// Timestamp when the appchain boots
    pub booting_timestamp: Timestamp,
    /// Nonce of currently valid validators set of the appchain,
    /// the nonce can be used to get a history validator set from `AppchainHistory`.
    pub currently_valid_validators_nonce: u32,
    /// Collection of validators which were removed from the appchain
    ///
    /// Each remove action for validator will create a new key in this collection,
    /// for users to withdraw their tokens.
    pub removed_validators: UnorderedMap<ValidatorId, AppchainValidator>,
    /// History records of validator set of appchain
    ///
    /// Each validator staking action after booting and validator removing action to the appchain
    /// also need to create a history record of validator set in the vector.
    pub validators_histories: Vector<ValidatorSet>,
    /// Current status of the appchain
    pub status: AppchainStatus,
    /// Total staked balance of OCT token of the appchain
    pub staked_balance: Balance,
    /// Total upvote balance of OCT token of the appchain
    pub upvote_balace: Balance,
    /// Total downvote balance of OCT token of the appchain
    pub downvote_balance: Balance,
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
                .map(|v| v.to_delegator())
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
                .map(|v| v.to_delegator())
                .collect(),
        }
    }
    /// Get delegator by `DelegatorId`
    pub fn get_delegator(&self, delegator_id: &DelegatorId) -> Option<AppchainDelegator> {
        if let Some(appchain_delegator) = self.delegators.get(delegator_id) {
            return Option::from(appchain_delegator);
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
                .map(|d| d.amount)
                .sum::<u128>()
    }
}

impl AppchainState {
    /// Return a new instance of AppchainState with the given `AppchainId`
    pub fn new(appchain_id: AppchainId) -> Self {
        let mut validators_storage_key = appchain_id.clone();
        validators_storage_key.push_str("_validators");
        let mut removed_validators_storage_key = appchain_id.clone();
        removed_validators_storage_key.push_str("_removed_validators");
        let mut validators_histories_storage_key = appchain_id.clone();
        validators_histories_storage_key.push_str("_validators_histories");
        Self {
            id: appchain_id,
            validators: UnorderedMap::new(validators_storage_key.into_bytes()),
            validators_nonce: 0,
            currently_valid_validators_nonce: 0,
            validators_timestamp: 0,
            booting_timestamp: 0,
            removed_validators: UnorderedMap::new(removed_validators_storage_key.into_bytes()),
            validators_histories: Vector::new(validators_histories_storage_key.into_bytes()),
            status: AppchainStatus::Auditing,
            staked_balance: 0,
            upvote_balace: 0,
            downvote_balance: 0,
        }
    }
    /// Get all validators of the appchain
    pub fn get_validators(&self) -> Vec<AppchainValidator> {
        self.validators.values_as_vector().iter().collect()
    }
    /// Get validator by `ValidatorId`
    pub fn get_validator(&self, validator_id: &ValidatorId) -> Option<AppchainValidator> {
        if let Some(appchain_validator) = self.validators.get(validator_id) {
            return Option::from(appchain_validator);
        }
        Option::None
    }
    /// Get validator set of the next period of cycle of current appchain
    pub fn get_next_validator_set(&self) -> Option<ValidatorSet> {
        if !self.status.eq(&AppchainStatus::Booting) {
            return Option::None;
        }
        let mut validators: Vec<LiteValidator> = self
            .validators
            .values_as_vector()
            .iter()
            .map(|v| v.to_lite_validator())
            .collect();
        validators.sort_by(|a, b| u128::from(b.weight).cmp(&a.weight.into()));
        let next_period_number =
            (env::block_timestamp() - self.booting_timestamp) / VALIDATOR_SET_CYCLE + 1;
        Option::from(ValidatorSet {
            seq_num: next_period_number.try_into().unwrap_or(0),
            set_id: self.validators_nonce,
            validators,
        })
    }
    // Get validator set of current period of cycle of current validators
    pub fn get_current_validator_set(&self) -> Option<ValidatorSet> {
        if !self.status.eq(&AppchainStatus::Booting) {
            return Option::None;
        }
        let period_number: u32 = ((env::block_timestamp() - self.booting_timestamp)
            / VALIDATOR_SET_CYCLE)
            .try_into()
            .unwrap_or(0);
        let mut current_validator_sets = self
            .validators_histories
            .iter()
            .filter(|s| s.seq_num == period_number)
            .collect::<Vec<_>>();
        if current_validator_sets.len() > 0 {
            current_validator_sets.pop()
        } else {
            match self
                .validators_histories
                .get(self.validators_histories.len() - 1)
            {
                Some(validator_set) => Option::from(ValidatorSet {
                    seq_num: period_number,
                    set_id: validator_set.set_id,
                    validators: validator_set.validators.clone(),
                }),
                None => Option::None,
            }
        }
    }
    /// Boot the appchain
    pub fn boot(&mut self) {
        self.status = AppchainStatus::Booting;
        self.validators_timestamp = env::block_timestamp();
        self.booting_timestamp = env::block_timestamp();
        let mut validators: Vec<LiteValidator> = self
            .validators
            .values_as_vector()
            .iter()
            .map(|v| v.to_lite_validator())
            .collect();
        validators.sort_by(|a, b| u128::from(b.weight).cmp(&a.weight.into()));
        self.validators_histories.push(&ValidatorSet {
            seq_num: 0,
            set_id: 0,
            validators,
        });
    }
    /// Stake some OCT tokens to the appchain
    pub fn stake(&mut self, validator_id: &ValidatorId, amount: Balance) -> bool {
        match self.status {
            AppchainStatus::Staging => {
                self.update_validator_amount(validator_id, amount);
                true
            }
            AppchainStatus::Booting => {
                self.update_validator_amount(validator_id, amount);
                self.validators_nonce += 1;
                self.create_validators_history();
                true
            }
            _ => false,
        }
    }
    // Internal logic for updating staking amount of a validator
    fn update_validator_amount(&mut self, validator_id: &ValidatorId, amount: Balance) {
        match self.validators.get(validator_id) {
            Some(mut validator) => validator.amount += amount,
            None => {
                let mut storage_key = self.id.clone();
                storage_key.push_str("_validator_");
                storage_key.push_str(validator_id.clone().as_str());
                storage_key.push_str("_delegators");
                self.validators.insert(
                    &validator_id,
                    &AppchainValidator {
                        validator_id: validator_id.clone(),
                        account_id: validator_id.clone(),
                        amount,
                        block_height: env::block_index(),
                        delegators: UnorderedMap::new(storage_key.into_bytes()),
                    },
                );
            }
        }
        self.validators_timestamp = env::block_timestamp();
        self.staked_balance += amount;
    }
    // Internal logic for creating validators history record
    fn create_validators_history(&mut self) {
        if let Some(validator_set) = self.get_next_validator_set() {
            self.validators_histories.push(&validator_set);
        }
    }
    /// Remove a validator from the appchain
    pub fn remove_validator(&mut self, validator_id: &ValidatorId) -> Balance {
        if let Some(validator) = self.get_validator(validator_id) {
            let removed_balance = validator.get_staked_balance_including_delegators();
            self.staked_balance -= removed_balance;
            self.removed_validators.insert(&validator_id, &validator);
            self.validators.remove(&validator_id);
            self.validators_timestamp = env::block_timestamp();
            if self.status.eq(&AppchainStatus::Booting) {
                self.create_validators_history();
            }
            removed_balance
        } else {
            0
        }
    }
    /// Get a validators history record by nonce
    pub fn get_validators_history_by_nonce(&self, validators_nonce: u32) -> Option<ValidatorSet> {
        if let Some(validator_set) = self.validators_histories.get(validators_nonce.into()) {
            Option::from(validator_set)
        } else {
            Option::None
        }
    }
    /// Get a validators history record by period number
    pub fn get_validators_history_by_period(&self, period_number: u32) -> Option<ValidatorSet> {
        let mut current_validator_sets = self
            .validators_histories
            .iter()
            .filter(|s| s.seq_num == period_number)
            .collect::<Vec<_>>();
        current_validator_sets.pop()
    }
    /// Freeze current appchain
    pub fn freeze(&mut self) {
        // TODO!
    }
    /// Pass auditing of current appchain
    pub fn pass_auditing(&mut self) {
        self.status = AppchainStatus::Voting;
    }
    /// Go staging of current appchain
    pub fn go_staging(&mut self) {
        self.status = AppchainStatus::Staging;
    }
}
