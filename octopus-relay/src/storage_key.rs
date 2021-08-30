use near_sdk::AccountId;

use crate::types::{AppchainId, DelegatorId, HistoryIndex, ValidatorId};

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
    RawFacts(AppchainId),
    ValidatorHistoryLists(AppchainId),
    ValidatorIndexToId(AppchainId),
    ValidatorIdToIndex(AppchainId),
    ValidatorIndexes(AccountId),
    AppchainFact {
        appchain_id: AppchainId,
        fact_index: u32,
    },
    RawFact {
        appchain_id: AppchainId,
        fact_index: u32,
    },
    RawFactHistoryKeys {
        appchain_id: AppchainId,
        fact_index: u32,
    },
    RawFactHistoryKey {
        appchain_id: AppchainId,
        fact_index: u32,
        validator_index: u32,
    },
    ValidatorHistoryList {
        appchain_id: AppchainId,
        validator_index: u32,
    },
    ValidatorHistoryListInner {
        appchain_id: AppchainId,
        validator_index: u32,
    },
    ValidatorHistory {
        appchain_id: AppchainId,
        validator_index: u32,
        history_index: HistoryIndex,
    },
    AppchainTotalLockedTokens(AppchainId),
    UsedMessage(AppchainId),
    AppchainValidator(AppchainId, ValidatorId),
    AppchainDelegators(AppchainId, ValidatorId),
    AppchainDelegator(AppchainId, ValidatorId, DelegatorId),
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
            StorageKey::RawFacts(appchain_id) => format!("{}%rfs", appchain_id),
            StorageKey::ValidatorHistoryLists(appchain_id) => format!("{}%vhs", appchain_id),
            StorageKey::ValidatorIndexToId(appchain_id) => format!("{}%vi", appchain_id),
            StorageKey::ValidatorIdToIndex(appchain_id) => format!("{}%iv", appchain_id),
            StorageKey::ValidatorIndexes(appchain_id) => format!("{}%vis", appchain_id),
            StorageKey::AppchainFact {
                appchain_id,
                fact_index,
            } => {
                format!("{}{:010}", appchain_id, fact_index)
            }
            StorageKey::RawFact {
                appchain_id,
                fact_index,
            } => {
                format!("{}{:010}%rf", appchain_id, fact_index)
            }
            StorageKey::RawFactHistoryKeys {
                appchain_id,
                fact_index,
            } => {
                format!("{}{:010}%rfvs", appchain_id, fact_index)
            }
            StorageKey::RawFactHistoryKey {
                appchain_id,
                fact_index,
                validator_index,
            } => {
                format!(
                    "{}{:010}{:010}%rfv",
                    appchain_id, fact_index, validator_index
                )
            }
            StorageKey::ValidatorHistoryList {
                appchain_id,
                validator_index,
            } => {
                format!("{}{:010}%vhl", appchain_id, validator_index)
            }
            StorageKey::ValidatorHistoryListInner {
                appchain_id,
                validator_index,
            } => {
                format!("{}{:010}%vhi", appchain_id, validator_index)
            }
            StorageKey::ValidatorHistory {
                appchain_id,
                validator_index,
                history_index,
            } => {
                format!(
                    "{}{:010}{:010}%vh",
                    appchain_id, validator_index, history_index
                )
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
