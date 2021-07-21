use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::AccountId;

use crate::storage_key::StorageKey;
use crate::types::{BridgeStatus, BridgeToken};
use crate::AppchainId;

/// Bridging status of bridge token
#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq)]
pub enum BridgingStatus {
    Activated,
    Paused,
    Closed,
}

impl Default for BridgingStatus {
    fn default() -> Self {
        BridgingStatus::Activated
    }
}

/// Struct for relayed bridge token
#[derive(BorshDeserialize, BorshSerialize)]
pub struct RelayedBridgeToken {
    token_id: AccountId,
    symbol: String,
    bridging_status: BridgingStatus,
    price: U128,
    decimals: u32,
    appchain_permitted: UnorderedMap<AppchainId, bool>,
}

impl RelayedBridgeToken {
    /// Create a new instance of the struct
    pub fn new(
        token_id: AccountId,
        symbol: String,
        bridging_status: BridgingStatus,
        price: U128,
        decimals: u32,
    ) -> Self {
        RelayedBridgeToken {
            token_id: token_id.clone(),
            symbol,
            bridging_status,
            price,
            decimals,
            appchain_permitted: UnorderedMap::new(
                StorageKey::RelayedBridgeTokenPermissions { token_id }.into_bytes(),
            ),
        }
    }
    /// Get id of the bridge token
    pub fn id(&self) -> AccountId {
        self.token_id.clone()
    }
    /// Get decimals of the bridge token
    pub fn decimals(&self) -> u32 {
        self.decimals
    }
    /// Get price of the bridge token
    pub fn price(&self) -> U128 {
        self.price.clone()
    }
    /// Get symbol of the bridge token
    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }
    /// Get status of the bridge token
    pub fn bridging_status(&self) -> BridgingStatus {
        self.bridging_status.clone()
    }
    /// Get permitted flag of an appchain
    pub fn is_permitted_of(&self, appchain_id: &AppchainId) -> bool {
        self.appchain_permitted.get(appchain_id).unwrap_or(false)
    }
    /// Convert to struct `BridgeToken`
    pub fn to_bridge_token(&self) -> BridgeToken {
        let status = match self.bridging_status {
            BridgingStatus::Activated => BridgeStatus::Active,
            BridgingStatus::Paused => BridgeStatus::Paused,
            BridgingStatus::Closed => BridgeStatus::Closed,
        };
        BridgeToken {
            token_id: self.token_id.clone(),
            symbol: self.symbol.clone(),
            status,
            price: self.price,
            decimals: self.decimals,
        }
    }
    /// Set price of the bridge token
    pub fn set_price(&mut self, price: &U128) {
        self.price = price.clone();
    }
    /// Activate the bridging of the token
    pub fn activate_bridging(&mut self) {
        self.bridging_status = BridgingStatus::Activated;
    }
    /// Pause the bridging of the token
    pub fn pause_bridging(&mut self) {
        self.bridging_status = BridgingStatus::Paused;
    }
    /// Close the bridging of the token
    pub fn close_bridging(&mut self) {
        self.bridging_status = BridgingStatus::Closed;
    }
    /// Set bridging permission for appchain
    pub fn set_bridging_permission(&mut self, appchain_id: &AppchainId, permitted: &bool) {
        self.appchain_permitted.insert(appchain_id, &permitted);
    }
}
