use std::convert::TryInto;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::{env, log, AccountId, Balance, Timestamp};

use crate::appchain_prover::AppchainProver;
use crate::storage_key::StorageKey;
use crate::types::{
    AppchainId, AppchainStatus, Burned, Fact, LiteValidator, Locked, SeqNum, ValidatorId,
    ValidatorIndex, ValidatorSet,
};
use crate::VALIDATOR_SET_CYCLE;

use super::fact::{AppchainBurnedNativeToken, AppchainLockedAsset, AppchainValidatorSet, RawFact};
use super::validator::{
    AppchainValidator, ValidatorHistory, ValidatorHistoryKeySet, ValidatorHistoryList,
};

/// Appchain state of an appchain of Octopus Network
#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainState {
    /// Id of the appchain
    pub appchain_id: AppchainId,
    /// Validators collection of the appchain
    pub validators: UnorderedMap<ValidatorId, LazyOption<AppchainValidator>>,
    /// Nonce of validator set of the appchain.
    ///
    /// This nonce will be increased by 1 for each validator_set updated.
    pub validators_nonce: u32,
    /// Last update time of validator_set of the appchain, will be updated for each staking action
    pub validators_timestamp: Timestamp,
    /// Last validators_timestamp when create the validator_set history
    pub validator_set_timestamp: Timestamp,
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
    pub raw_facts: Vector<LazyOption<RawFact>>,
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
    /// The cross-chain prover of the appchain
    pub prover: AppchainProver,
    /// used_messages of the appchain
    pub used_messages: UnorderedMap<u64, bool>,
    /// map of validator_history_list
    pub validator_history_lists: LookupMap<ValidatorIndex, LazyOption<ValidatorHistoryList>>,
    pub validator_index_to_id: LookupMap<ValidatorIndex, ValidatorId>,
    pub validator_last_index: ValidatorIndex,
    pub validator_id_to_index: LookupMap<ValidatorId, ValidatorIndex>,
    /// Current validators by index
    pub validator_indexes: UnorderedMap<ValidatorIndex, bool>,
}

impl AppchainState {
    /// Return a new instance of AppchainState with the given `AppchainId`
    pub fn new(appchain_id: &AppchainId) -> Self {
        Self {
            appchain_id: appchain_id.clone(),
            validators: UnorderedMap::new(
                StorageKey::AppchainValidators(appchain_id.clone()).into_bytes(),
            ),
            // set_id counts from 1
            validators_nonce: 1,
            currently_valid_validators_nonce: 1,
            validators_timestamp: 0,
            validator_set_timestamp: 0,
            booting_timestamp: 0,
            removed_validators: UnorderedMap::new(
                StorageKey::RemovedAppchainValidators(appchain_id.clone()).into_bytes(),
            ),
            raw_facts: Vector::new(StorageKey::RawFacts(appchain_id.clone()).into_bytes()),
            status: AppchainStatus::Auditing,
            staked_balance: 0,
            total_locked_tokens: UnorderedMap::new(
                StorageKey::AppchainTotalLockedTokens(appchain_id.clone()).into_bytes(),
            ),
            upvote_balance: 0,
            downvote_balance: 0,
            prover: AppchainProver,
            used_messages: UnorderedMap::new(
                StorageKey::UsedMessage(appchain_id.clone()).into_bytes(),
            ),
            validator_history_lists: LookupMap::new(
                StorageKey::ValidatorHistoryLists(appchain_id.clone()).into_bytes(),
            ),
            validator_index_to_id: LookupMap::new(
                StorageKey::ValidatorIndexToId(appchain_id.clone()).into_bytes(),
            ),
            validator_last_index: 0,
            validator_id_to_index: LookupMap::new(
                StorageKey::ValidatorIdToIndex(appchain_id.clone()).into_bytes(),
            ),
            validator_indexes: UnorderedMap::new(
                StorageKey::ValidatorIndexes(appchain_id.clone()).into_bytes(),
            ),
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

    fn history_key_set_to_validator_set(
        &self,
        key_set_view: ValidatorHistoryKeySet,
    ) -> ValidatorSet {
        log!("history_key_set_to_validator_set");
        let mut validators = Vec::new();
        log!(
            "key_set_view.history_keys.len(), {}",
            key_set_view.history_keys.len()
        );
        for index in 0..key_set_view.history_keys.len() {
            let (v_index, history_index) = key_set_view.history_keys.get(index).unwrap();
            let history_list = self
                .validator_history_lists
                .get(&v_index)
                .unwrap()
                .get()
                .unwrap();
            let validator = history_list
                .get(*history_index as u64)
                .unwrap()
                .get()
                .unwrap()
                .to_lite_validator();

            validators.push(validator);
        }
        ValidatorSet {
            seq_num: key_set_view.seq_num,
            set_id: key_set_view.set_id,
            validators,
        }
    }

    fn raw_fact_to_fact(&self, raw_fact: RawFact) -> Fact {
        match raw_fact {
            RawFact::ValidatorHistoryKeySet(key_set) => {
                Fact::UpdateValidatorSet(self.history_key_set_to_validator_set(key_set))
            }
            RawFact::LockAsset(locked) => Fact::LockAsset(locked),
            RawFact::Burn(burned) => Fact::Burn(burned),
        }
    }

    /// Get validator set of the next set_id
    pub fn should_next_validator_set(&self) -> bool {
        let updated_time_from_booting = self.validators_timestamp - self.booting_timestamp;
        let updated_cycles_from_booting = updated_time_from_booting / VALIDATOR_SET_CYCLE;
        let now_cycles_from_booting =
            (env::block_timestamp() - self.booting_timestamp) / VALIDATOR_SET_CYCLE;

        let time_for_next = self.validator_set_timestamp != self.validators_timestamp
            && updated_time_from_booting > 0
            && now_cycles_from_booting - updated_cycles_from_booting > 0;

        return time_for_next && self.status.eq(&AppchainStatus::Booting);
    }

    pub fn get_next_validator_set(&self) -> Option<ValidatorSet> {
        if self.should_next_validator_set() {
            return Option::from(self.history_key_set_to_validator_set(
                self.get_latest_validator_history_key_set_view(),
            ));
        }
        None
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

    // Convert current validators array to struct `ValidatorSet`
    fn get_latest_validator_history_key_set_view(&self) -> ValidatorHistoryKeySet {
        let next_seq_num = self.raw_facts.len().try_into().unwrap();
        let mut validator_history_keys = Vec::new();
        let mut h_key_index: u32 = 0;
        self.validator_indexes.keys().for_each(|v_index| {
            let validator_history_list = self
                .validator_history_lists
                .get(&v_index)
                .unwrap()
                .get()
                .unwrap();
            validator_history_keys.push((v_index, validator_history_list.len() as u32 - 1));
            h_key_index += 1;
        });
        ValidatorHistoryKeySet {
            seq_num: next_seq_num,
            set_id: self.validators_nonce,
            history_keys: validator_history_keys,
        }
    }

    /// Get validator set of current epoch
    ///
    /// The return data is come from the facts of the appchain
    // pub fn get_current_validator_set(&self) -> Option<ValidatorSet> {
    //     if !self.status.eq(&AppchainStatus::Booting) {
    //         return Option::None;
    //     }
    //     let current_epoch_number: u32 = ((env::block_timestamp() - self.booting_timestamp)
    //         / VALIDATOR_SET_CYCLE)
    //         .try_into()
    //         .unwrap();
    //     let mut current_validator_sets = self
    //         .facts
    //         .iter()
    //         .filter(|f| match f.get().unwrap() {
    //             RawFact::UpdateValidatorHistoryKeySet(key_set) => {
    //                 validator_set.epoch_number <= current_epoch_number
    //             }
    //             _ => false,
    //         })
    //         .collect::<Vec<_>>();
    //     if current_validator_sets.len() > 0 {
    //         if let Some(fact) = current_validator_sets.pop() {
    //             match fact.get().unwrap() {
    //                 RawFact::UpdateValidatorSet(validator_set) => {
    //                     return Option::from(validator_set.to_validator_set());
    //                 }
    //                 _ => (),
    //             }
    //         }
    //     }
    //     Option::None
    // }
    /// Boot the appchain
    pub fn boot(&mut self) {
        self.status = AppchainStatus::Booting;
        self.booting_timestamp = env::block_timestamp();
        self.validators_timestamp = env::block_timestamp();
        self.validator_set_timestamp = env::block_timestamp();
        self.create_validators_history(true);
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
                // Try to create validators_history before stake.
                self.create_validators_history(false);
                self.update_validator_amount(validator_id, &account_id, amount);
                self.validators_timestamp = env::block_timestamp();
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
        self.staked_balance += amount;
        self.create_index_for_validator(validator_id.clone());
        self.record_validator_history(validator_id.clone());
    }

    fn create_index_for_validator(&mut self, validator_id: ValidatorId) {
        if self.validator_id_to_index.get(&validator_id).is_none() {
            let validator_index = self.validator_last_index + 1;
            self.validator_id_to_index
                .insert(&validator_id, &validator_index);
            self.validator_index_to_id
                .insert(&validator_index, &validator_id);
        }
        let index_of_validator = self.validator_id_to_index.get(&validator_id).unwrap();
        log!("create_index_for_validator {}", index_of_validator);
        self.validator_indexes.insert(&index_of_validator, &true);
        self.validator_last_index += 1;
    }

    fn record_validator_history(&mut self, validator_id: ValidatorId) {
        let v_index = self.validator_id_to_index.get(&validator_id).unwrap();
        let validator_history_list_option = self.validator_history_lists.get(&v_index);
        let mut validator_history_list;
        if validator_history_list_option.is_none() {
            validator_history_list = Vector::new(
                StorageKey::ValidatorHistoryList {
                    appchain_id: self.appchain_id.clone(),
                    validator_index: v_index,
                }
                .into_bytes(),
            );
        } else {
            validator_history_list = validator_history_list_option.unwrap().get().unwrap();
        }
        let validator_history = self
            .validators
            .get(&validator_id)
            .unwrap()
            .get()
            .unwrap()
            .to_validator_history();
        let next_validator_history_index = validator_history_list.len().try_into().unwrap();
        validator_history_list.push(&LazyOption::new(
            StorageKey::ValidatorHistory {
                appchain_id: self.appchain_id.clone(),
                validator_index: v_index,
                history_index: next_validator_history_index,
            }
            .into_bytes(),
            Some(&validator_history),
        ));
        self.validator_history_lists.insert(
            &v_index,
            &LazyOption::new(
                StorageKey::ValidatorHistoryListInner {
                    appchain_id: self.appchain_id.clone(),
                    validator_index: v_index,
                }
                .into_bytes(),
                Some(&validator_history_list),
            ),
        );
    }

    // Internal logic for creating validators history record
    pub fn create_validators_history(&mut self, for_boot: bool) {
        if self.should_next_validator_set() || for_boot {
            log!("validator_indexes length {}", self.validator_indexes.len());
            if self.validator_indexes.len() > 0 {
                let next_seq_num = self.raw_facts.len().try_into().unwrap();
                let mut validator_history_keys = Vec::new();

                self.validator_indexes.keys().for_each(|v_index| {
                    let validator_history_list = self
                        .validator_history_lists
                        .get(&v_index)
                        .unwrap()
                        .get()
                        .unwrap();
                    validator_history_keys.push((v_index, validator_history_list.len() as u32 - 1));
                });

                let raw_fact = LazyOption::new(
                    StorageKey::RawFact {
                        appchain_id: self.appchain_id.clone(),
                        fact_index: next_seq_num,
                    }
                    .into_bytes(),
                    Some(&RawFact::ValidatorHistoryKeySet(ValidatorHistoryKeySet {
                        seq_num: next_seq_num,
                        set_id: self.validators_nonce,
                        history_keys: validator_history_keys,
                    })),
                );
                self.raw_facts.push(&raw_fact);
                self.validators_nonce += 1;
                log!("validators_nonce {}", self.validators_nonce);
                self.validator_set_timestamp = self.validators_timestamp;
            }
        }
    }
    /// Remove a validator from the appchain
    pub fn remove_validator(&mut self, validator_id: &ValidatorId) -> Balance {
        if let Some(validator) = self.get_validator(validator_id) {
            if self.status.eq(&AppchainStatus::Booting) {
                // Try to create validators_history before remove.
                self.create_validators_history(false);
                self.validators_timestamp = env::block_timestamp();
            }
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
            let v_index = self.validator_id_to_index.get(&validator_id).unwrap();
            self.validator_indexes.remove(&v_index);
            self.validators.remove(&validator_id);
            removed_balance
        } else {
            0
        }
    }
    /// Get a validators history record by nonce
    pub fn get_validator_set_by_nonce(&self, validators_nonce: &u32) -> Option<ValidatorSet> {
        let validator_history_set_facts = self
            .raw_facts
            .iter()
            .filter(|f| match f.get().unwrap() {
                RawFact::ValidatorHistoryKeySet(key_set) => key_set.set_id.eq(validators_nonce),
                _ => false,
            })
            .collect::<Vec<_>>();
        if validator_history_set_facts.len() > 0 {
            match validator_history_set_facts.get(0).unwrap().get().unwrap() {
                RawFact::ValidatorHistoryKeySet(key_set) => {
                    Option::from(self.history_key_set_to_validator_set(key_set))
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
        let next_seq_num = self.raw_facts.len().try_into().unwrap();
        let epoch_number: u32 = ((env::block_timestamp() - self.booting_timestamp)
            / VALIDATOR_SET_CYCLE)
            .try_into()
            .unwrap();
        self.raw_facts.push(&LazyOption::new(
            StorageKey::RawFact {
                appchain_id: self.appchain_id.clone(),
                fact_index: next_seq_num,
            }
            .into_bytes(),
            Some(&RawFact::LockAsset(Locked {
                seq_num: next_seq_num,
                token_id,
                sender_id,
                receiver,
                amount: U128::from(amount),
            })),
        ));
    }

    pub fn message_set_used(&mut self, nonce: u64) {
        self.used_messages.insert(&nonce, &true);
    }

    pub fn is_message_used(&self, nonce: u64) -> bool {
        self.used_messages.get(&nonce).is_some()
    }

    pub fn burn_native_token(&mut self, receiver: String, sender_id: AccountId, amount: u128) {
        let next_seq_num = self.raw_facts.len().try_into().unwrap();
        let epoch_number: u32 = ((env::block_timestamp() - self.booting_timestamp)
            / VALIDATOR_SET_CYCLE)
            .try_into()
            .unwrap();
        self.raw_facts.push(&LazyOption::new(
            StorageKey::RawFact {
                appchain_id: self.appchain_id.clone(),
                fact_index: next_seq_num,
            }
            .into_bytes(),
            Some(&RawFact::Burn(Burned {
                seq_num: next_seq_num,
                sender_id,
                receiver,
                amount: U128::from(amount),
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
    // Get facts by limit number
    pub fn get_facts(&self, start: &SeqNum, limit: &SeqNum) -> Vec<Fact> {
        let facts_len = self.raw_facts.len().try_into().unwrap_or(0);
        let end = std::cmp::min(start + limit, facts_len);
        let mut facts = (start.clone()..end)
            .map(|index| {
                self.raw_fact_to_fact(self.raw_facts.get(index.into()).unwrap().get().unwrap())
            })
            .collect::<Vec<_>>();

        let next_validator_set_option = self.get_next_validator_set();
        let next_end = std::cmp::min(start + limit, facts_len + 1);
        if let Some(next_validator_set) = next_validator_set_option {
            if next_validator_set.seq_num >= *start && next_validator_set.seq_num < next_end {
                facts.push(Fact::UpdateValidatorSet(next_validator_set));
            }
        }
        facts
    }
}
