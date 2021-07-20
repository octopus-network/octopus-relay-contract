use std::convert::TryInto;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::{env, AccountId, Balance, Timestamp};

use crate::storage_key::StorageKey;
use crate::types::{AppchainStatus, Fact, ValidatorSet};
use crate::{AppchainId, SeqNum, ValidatorId, VALIDATOR_SET_CYCLE};

use super::fact::{AppchainFact, AppchainLockedToken, AppchainValidatorSet};
use super::validator::AppchainValidator;

/// Appchain state of an appchain of Octopus Network
#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainState {
    /// Id of the appchain
    pub appchain_id: AppchainId,
    /// Validators collection of the appchain
    pub validators: UnorderedMap<ValidatorId, LazyOption<AppchainValidator>>,
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
    pub removed_validators: UnorderedMap<ValidatorId, LazyOption<AppchainValidator>>,
    /// History records of facts happened which were related to the appchain
    pub facts: Vector<LazyOption<AppchainFact>>,
    /// Current status of the appchain
    pub status: AppchainStatus,
    /// Total staked balance of OCT token of the appchain
    pub staked_balance: Balance,
    /// Collection of total amount of locked tokens
    pub total_locked_tokens: UnorderedMap<AccountId, u128>,
    /// Total upvote balance of OCT token of the appchain
    pub upvote_balance: Balance,
    /// Total downvote balance of OCT token of the appchain
    pub downvote_balance: Balance,
}

impl AppchainState {
    /// Return a new instance of AppchainState with the given `AppchainId`
    pub fn new(appchain_id: &AppchainId) -> Self {
        Self {
            appchain_id: appchain_id.clone(),
            validators: UnorderedMap::new(
                StorageKey::AppchainValidators(appchain_id.clone()).into_bytes(),
            ),
            validators_nonce: 0,
            currently_valid_validators_nonce: 0,
            validators_timestamp: 0,
            booting_timestamp: 0,
            removed_validators: UnorderedMap::new(
                StorageKey::RemovedAppchainValidators(appchain_id.clone()).into_bytes(),
            ),
            facts: Vector::new(StorageKey::AppchainFacts(appchain_id.clone()).into_bytes()),
            status: AppchainStatus::Auditing,
            staked_balance: 0,
            total_locked_tokens: UnorderedMap::new(
                StorageKey::AppchainTotalLockedTokens(appchain_id.clone()).into_bytes(),
            ),
            upvote_balance: 0,
            downvote_balance: 0,
        }
    }
    /// Clear extra storage used by the appchain
    ///
    /// **This function must be called before remove `AppchainState` from storage**
    pub fn clear_extra_storage(&mut self) {
        self.validators.values_as_vector().iter().for_each(|mut d| {
            if let Some(validator) = d.get() {
                validator.clear_extra_storage();
            }
            d.remove();
        });
    }
    /// Get all validators of the appchain
    pub fn get_validators(&self) -> Vec<AppchainValidator> {
        self.validators
            .values_as_vector()
            .iter()
            .filter(|v| v.is_some())
            .map(|v| v.get().unwrap())
            .collect()
    }
    /// Get validator by `ValidatorId`
    pub fn get_validator(&self, validator_id: &ValidatorId) -> Option<AppchainValidator> {
        if let Some(appchain_validator_option) = self.validators.get(validator_id) {
            return appchain_validator_option.get();
        }
        Option::None
    }
    /// Get validator set of the next epoch
    pub fn get_next_validator_set(&self) -> Option<ValidatorSet> {
        if !self.status.eq(&AppchainStatus::Booting) {
            return Option::None;
        }
        let validators = self
            .get_sorted_validators()
            .iter()
            .map(|v| v.to_lite_validator())
            .collect();
        Option::from(ValidatorSet {
            seq_num: self.facts.len().try_into().unwrap(),
            set_id: self.validators_nonce + 1,
            validators,
        })
    }
    // Sort current validators array by `ValidatorId`
    fn get_sorted_validators(&self) -> Vec<AppchainValidator> {
        let mut validators: Vec<AppchainValidator> = self
            .validators
            .values_as_vector()
            .iter()
            .filter(|v| v.is_some())
            .map(|v| v.get().unwrap())
            .collect();
        validators.sort_by(|a, b| a.validator_id.cmp(&b.validator_id));
        validators
    }
    /// Get validator set of current epoch
    ///
    /// The return data is come from the facts of the appchain
    pub fn get_current_validator_set(&self) -> Option<ValidatorSet> {
        if !self.status.eq(&AppchainStatus::Booting) {
            return Option::None;
        }
        let current_epoch_number: u32 = ((env::block_timestamp() - self.booting_timestamp)
            / VALIDATOR_SET_CYCLE)
            .try_into()
            .unwrap();
        let mut current_validator_sets = self
            .facts
            .iter()
            .filter(|f| match f.get().unwrap() {
                AppchainFact::UpdateValidatorSet(validator_set) => {
                    validator_set.epoch_number <= current_epoch_number
                }
                _ => false,
            })
            .collect::<Vec<_>>();
        if current_validator_sets.len() > 0 {
            if let Some(fact) = current_validator_sets.pop() {
                match fact.get().unwrap() {
                    AppchainFact::UpdateValidatorSet(validator_set) => {
                        return Option::from(validator_set.to_validator_set());
                    }
                    _ => (),
                }
            }
        }
        Option::None
    }
    /// Boot the appchain
    pub fn boot(&mut self) {
        self.status = AppchainStatus::Booting;
        self.validators_timestamp = env::block_timestamp();
        self.booting_timestamp = env::block_timestamp();
        self.create_validators_history();
    }
    /// Stake some OCT tokens to the appchain
    pub fn stake(&mut self, validator_id: &ValidatorId, amount: &Balance) -> bool {
        let account_id = env::signer_account_id();
        match self.status {
            AppchainStatus::Staging => {
                self.update_validator_amount(validator_id, &account_id, amount);
                true
            }
            AppchainStatus::Booting => {
                self.update_validator_amount(validator_id, &account_id, amount);
                self.validators_nonce += 1;
                self.create_validators_history();
                true
            }
            _ => false,
        }
    }
    // Internal logic for updating staking amount of a validator
    fn update_validator_amount(
        &mut self,
        validator_id: &ValidatorId,
        account_id: &AccountId,
        amount: &Balance,
    ) {
        match self.validators.get(validator_id) {
            Some(mut validator_option) => {
                if let Some(mut validator) = validator_option.get() {
                    validator.amount += amount;
                    validator_option.set(&validator);
                }
            }
            None => {
                self.validators.insert(
                    &validator_id,
                    &LazyOption::new(
                        StorageKey::AppchainValidator(
                            self.appchain_id.clone(),
                            validator_id.clone(),
                        )
                        .into_bytes(),
                        Some(&AppchainValidator {
                            validator_id: validator_id.clone(),
                            account_id: account_id.clone(),
                            amount: amount.clone(),
                            block_height: env::block_index(),
                            delegators: UnorderedMap::new(
                                StorageKey::AppchainDelegators(
                                    self.appchain_id.clone(),
                                    validator_id.clone(),
                                )
                                .into_bytes(),
                            ),
                        }),
                    ),
                );
            }
        }
        self.validators_timestamp = env::block_timestamp();
        self.staked_balance += amount;
    }
    // Internal logic for creating validators history record
    fn create_validators_history(&mut self) {
        let validators = self.get_sorted_validators();
        if validators.len() > 0 {
            let next_sequence_number = self.facts.len().try_into().unwrap();
            let next_epoch_number: u32 =
                ((env::block_timestamp() - self.booting_timestamp) / VALIDATOR_SET_CYCLE + 1)
                    .try_into()
                    .unwrap();
            let mut validators_vector = Vector::new(
                StorageKey::AppchainFactValidators {
                    appchain_id: self.appchain_id.clone(),
                    fact_index: next_sequence_number,
                }
                .into_bytes(),
            );
            for index in 0..validators.len() {
                validators_vector.push(&LazyOption::new(
                    StorageKey::AppchainFactValidator {
                        appchain_id: self.appchain_id.clone(),
                        fact_index: next_sequence_number,
                        validator_index: index.try_into().unwrap(),
                    }
                    .into_bytes(),
                    Some(validators.get(index).unwrap()),
                ));
            }
            let fact = LazyOption::new(
                StorageKey::AppchainFact {
                    appchain_id: self.appchain_id.clone(),
                    fact_index: next_sequence_number,
                }
                .into_bytes(),
                Some(&AppchainFact::UpdateValidatorSet(AppchainValidatorSet {
                    sequence_number: next_sequence_number,
                    set_id: self.validators_nonce,
                    validators: validators_vector,
                    timestamp: env::block_timestamp(),
                    epoch_number: next_epoch_number,
                })),
            );
            self.facts.push(&fact);
        }
    }
    /// Remove a validator from the appchain
    pub fn remove_validator(&mut self, validator_id: &ValidatorId) -> Balance {
        if let Some(validator) = self.get_validator(validator_id) {
            let removed_balance = validator.get_staked_balance_including_delegators();
            self.staked_balance -= removed_balance;
            self.removed_validators.insert(
                &validator_id,
                &LazyOption::new(
                    StorageKey::AppchainValidator(self.appchain_id.clone(), validator_id.clone())
                        .into_bytes(),
                    Some(&validator),
                ),
            );
            self.validators.remove(&validator_id);
            self.validators_timestamp = env::block_timestamp();
            if self.status.eq(&AppchainStatus::Booting) {
                self.validators_nonce += 1;
                self.create_validators_history();
            }
            removed_balance
        } else {
            0
        }
    }
    /// Get a validators history record by nonce
    pub fn get_validators_history_by_nonce(&self, validators_nonce: &u32) -> Option<ValidatorSet> {
        let update_validator_set_facts = self
            .facts
            .iter()
            .filter(|f| match f.get().unwrap() {
                AppchainFact::UpdateValidatorSet(validator_set) => {
                    validator_set.set_id.eq(validators_nonce)
                }
                _ => false,
            })
            .collect::<Vec<_>>();
        if update_validator_set_facts.len() > 0 {
            match update_validator_set_facts.get(0).unwrap().get().unwrap() {
                AppchainFact::UpdateValidatorSet(validator_set) => {
                    Option::from(validator_set.to_validator_set())
                }
                _ => Option::None,
            }
        } else {
            Option::None
        }
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
    /// Lock some token on current appchain
    pub fn lock_token(
        &mut self,
        receiver: String,
        sender_id: AccountId,
        token_id: AccountId,
        amount: u128,
    ) {
        let new_amount = self.total_locked_tokens.get(&token_id).unwrap_or(0) + amount;
        self.total_locked_tokens.insert(&token_id, &new_amount);
        let next_sequence_number = self.facts.len().try_into().unwrap();
        let epoch_number: u32 = ((env::block_timestamp() - self.booting_timestamp)
            / VALIDATOR_SET_CYCLE)
            .try_into()
            .unwrap();
        self.facts.push(&LazyOption::new(
            StorageKey::AppchainFact {
                appchain_id: self.appchain_id.clone(),
                fact_index: next_sequence_number,
            }
            .into_bytes(),
            Some(&AppchainFact::LockToken(AppchainLockedToken {
                sequence_number: next_sequence_number,
                token_id,
                sender_id,
                receiver,
                amount: U128::from(amount),
                block_height: env::block_index(),
                timestamp: env::block_timestamp(),
                epoch_number,
            })),
        ));
    }
    /// Unlock some token on current appchain
    pub fn unlock_token(&mut self, token_id: AccountId, amount: u128) {
        let new_amount = self.total_locked_tokens.get(&token_id).unwrap_or(0) - amount;
        self.total_locked_tokens.insert(&token_id, &new_amount);
    }
    /// Get total locked amount of a token
    pub fn get_total_locked_amount_of(&self, token_id: &AccountId) -> u128 {
        self.total_locked_tokens.get(token_id).unwrap_or(0)
    }
    /// Get facts by limit number
    pub fn get_facts(&self, start: &SeqNum, limit: &SeqNum) -> Vec<Fact> {
        let facts_len = self.facts.len().try_into().unwrap_or(0);
        assert!(facts_len.gt(start), "Invalid start index of facts.");
        let end = std::cmp::min(start + limit, facts_len);
        (start.clone()..end)
            .map(|index| {
                self.facts
                    .get(index.into())
                    .unwrap()
                    .get()
                    .unwrap()
                    .to_fact()
            })
            .collect::<Vec<_>>()
    }
}
