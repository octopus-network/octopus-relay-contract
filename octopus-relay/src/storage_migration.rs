//! Sample implementation of storage migration for adding a field
//! to an internal struct of relay contract
//!
//! Every time we change the fields of a struct in relay contract,
//! we need to write an one-time migration function for relay contract like this module.
//!
//! The following implementation shows how to migrate storage of OctopusRelay contract
//! when we need to add a field `note` to struct `AppchainValidator`.
use crate::appchain::state::{AppchainDelegator, AppchainValidator};
use crate::*;

/// Appchain validator of an appchain
#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldAppchainValidator {
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

impl OldAppchainValidator {
    pub fn migrate_state(
        appchain_id: &AppchainId,
        validator_id: &ValidatorId,
        new_note_of_validator: &String,
    ) {
        let storage_key =
            StorageKey::AppchainValidator(appchain_id.clone(), validator_id.clone()).into_bytes();
        if let Some(data) = env::storage_read(&storage_key) {
            if let Ok(validator) = OldAppchainValidator::try_from_slice(&data) {
                env::log(
                    format!("Migrating state of validator '{}'", &validator.account_id).as_bytes(),
                );
                let mut delegators: UnorderedMap<DelegatorId, LazyOption<AppchainDelegator>> =
                    UnorderedMap::new(
                        StorageKey::AppchainDelegators(appchain_id.clone(), validator_id.clone())
                            .into_bytes(),
                    );
                validator.delegators.iter().for_each(|(k, v)| {
                    delegators.insert(&k, &v);
                });
                let new_state = AppchainValidator {
                    validator_id: validator.validator_id.clone(),
                    account_id: validator.account_id.clone(),
                    amount: validator.amount,
                    block_height: validator.block_height,
                    delegators,
                    note: new_note_of_validator.clone(),
                };
                if let Ok(new_data) = new_state.try_to_vec() {
                    assert!(
                        env::storage_write(&storage_key, &new_data),
                        "Migration for validator '{}' failed",
                        &validator.account_id
                    );
                }
            }
        }
    }
}

impl AppchainState {
    pub fn migrate_validator_state(&self, new_note_of_validator: &String) {
        self.validators.keys_as_vector().iter().for_each(|v| {
            OldAppchainValidator::migrate_state(&self.id, &v, new_note_of_validator);
        });
        self.removed_validators.keys_as_vector().iter().for_each(|v| {
            OldAppchainValidator::migrate_state(&self.id, &v, new_note_of_validator);
        });
    }
}

#[near_bindgen]
impl OctopusRelay {
    #[init(ignore_state)]
    pub fn migrate_state(new_note_of_validator: String) -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: OctopusRelay = env::state_read().expect("Old state doesn't exist");
        // Verify that the migration can only be done by the owner.
        // This is not necessary, if the upgrade is done internally.
        assert_eq!(
            &env::predecessor_account_id(),
            &old_contract.owner,
            "Can only be called by the owner"
        );

        // Add new field `note` of `AppchainValidator` to old state
        old_contract
            .appchain_states
            .values_as_vector()
            .iter()
            .for_each(|s| {
                let state = s.get().unwrap();
                env::log(format!("Migrating state of appchain '{}'", state.id).as_bytes());
                state.migrate_validator_state(&new_note_of_validator);
            });

        // Create the new contract using the data from the old contract.
        old_contract
    }
}
