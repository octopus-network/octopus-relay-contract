use near_sdk::AccountId;

use crate::{AppchainId, DelegatorId, ValidatorId};

/// Storage keys for collections of sub-struct in main contract
pub enum StorageKey {
    AppchainIdList,
    AppchainMetadatas,
    AppchainMetadata(AppchainId),
    AppchainStates,
    AppchainState(AppchainId),
    AppchainValidators(AppchainId),
    RemovedAppchainValidators(AppchainId),
    AppchainFacts(AppchainId),
    AppchainFact {
        appchain_id: AppchainId,
        fact_index: u32,
    },
    AppchainTotalLockedTokens(AppchainId),
    UsedMessage(AppchainId),
    AppchainValidator(AppchainId, ValidatorId),
    AppchainDelegators(AppchainId, ValidatorId),
    AppchainDelegator(AppchainId, ValidatorId, DelegatorId),
    AppchainFactValidators {
        appchain_id: AppchainId,
        fact_index: u32,
    },
    AppchainFactValidator {
        appchain_id: AppchainId,
        fact_index: u32,
        validator_index: u32,
    },
    BridgeTokens,
    RelayedBridgeToken {
        token_id: AccountId,
    },
    RelayedBridgeTokenPermissions {
        token_id: AccountId,
    },
    AppchainNativeTokens,
}

impl StorageKey {
    pub fn to_string(&self) -> String {
        match self {
            StorageKey::AppchainIdList => "ail".to_string(),
            StorageKey::AppchainMetadatas => "am".to_string(),
            StorageKey::AppchainMetadata(appchain_id) => format!("{}m", appchain_id),
            StorageKey::AppchainStates => "as".to_string(),
            StorageKey::AppchainState(appchain_id) => format!("{}s", appchain_id),
            StorageKey::AppchainValidators(appchain_id) => format!("{}v", appchain_id),
            StorageKey::RemovedAppchainValidators(appchain_id) => format!("{}r", appchain_id),
            StorageKey::AppchainFacts(appchain_id) => format!("{}f", appchain_id),
            StorageKey::AppchainFact {
                appchain_id,
                fact_index,
            } => {
                format!("{}{:010}", appchain_id, fact_index)
            }
            StorageKey::AppchainTotalLockedTokens(appchain_id) => format!("{}t", appchain_id),
            StorageKey::UsedMessage(appchain_id) => format!("{}%um", appchain_id),
            StorageKey::AppchainValidator(appchain_id, validator_id) => {
                format!("{}{}", appchain_id, validator_id)
            }
            StorageKey::AppchainDelegators(appchain_id, validator_id) => {
                format!("{}{}d", appchain_id, validator_id)
            }
            StorageKey::AppchainDelegator(appchain_id, validator_id, delegator_id) => {
                format!("{}{}{}", appchain_id, validator_id, delegator_id)
            }
            StorageKey::AppchainFactValidators {
                appchain_id,
                fact_index,
            } => {
                format!("{}{:010}v", appchain_id, fact_index)
            }
            StorageKey::AppchainFactValidator {
                appchain_id,
                fact_index,
                validator_index,
            } => {
                format!("{}{:010}{:010}", appchain_id, fact_index, validator_index)
            }
            StorageKey::BridgeTokens => "bts".to_string(),
            StorageKey::RelayedBridgeToken { token_id } => {
                format!("rt{}", token_id)
            }
            StorageKey::RelayedBridgeTokenPermissions { token_id } => {
                format!("rt{}ps", token_id)
            }
            StorageKey::AppchainNativeTokens => "ant".to_string(),
        }
    }
    pub fn into_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}
