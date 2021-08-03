use borsh::maybestd::string::{String, ToString};
use borsh::maybestd::vec::Vec;

use crate::{AppchainId, DelegatorId, ValidatorId};

/// Storage keys for collections of sub-struct in main contract
pub enum StorageKey {
    AppchainMetadatas,
    AppchainMetadata(AppchainId),
    AppchainStates,
    AppchainState(AppchainId),
    AppchainValidators(AppchainId),
    RemovedAppchainValidators(AppchainId),
    AppchainFacts(AppchainId),
    AppchainTotalLockedTokens(AppchainId),
    AppchainValidator(AppchainId, ValidatorId),
    AppchainDelegators(AppchainId, ValidatorId),
    AppchainDelegator(AppchainId, ValidatorId, DelegatorId),
}

impl StorageKey {
    pub fn to_string(&self) -> String {
        match self {
            StorageKey::AppchainMetadatas => "am".to_string(),
            StorageKey::AppchainMetadata(appchain_id) => {
                let mut key = appchain_id.clone();
                key.push_str("m");
                key
            }
            StorageKey::AppchainStates => "as".to_string(),
            StorageKey::AppchainState(appchain_id) => {
                let mut key = appchain_id.clone();
                key.push_str("s");
                key
            }
            StorageKey::AppchainValidators(appchain_id) => {
                let mut key = appchain_id.clone();
                key.push_str("v");
                key
            }
            StorageKey::RemovedAppchainValidators(appchain_id) => {
                let mut key = appchain_id.clone();
                key.push_str("r");
                key
            }
            StorageKey::AppchainFacts(appchain_id) => {
                let mut key = appchain_id.clone();
                key.push_str("f");
                key
            }
            StorageKey::AppchainTotalLockedTokens(appchain_id) => {
                let mut key = appchain_id.clone();
                key.push_str("t");
                key
            }
            StorageKey::AppchainValidator(appchain_id, validator_id) => {
                let mut key = appchain_id.clone();
                key.push_str(validator_id.as_str());
                key
            }
            StorageKey::AppchainDelegators(appchain_id, validator_id) => {
                let mut key = appchain_id.clone();
                key.push_str(validator_id.as_str());
                key.push_str("d");
                key
            }
            StorageKey::AppchainDelegator(appchain_id, validator_id, delegator_id) => {
                let mut key = appchain_id.clone();
                key.push_str(validator_id.as_str());
                key.push_str(delegator_id.as_str());
                key
            }
        }
    }
    pub fn into_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}
