use crate::*;

/// Interfaces for manager bridge tokens
pub trait NativeTokenManager {
    /// Register a new bridge token
    fn register_native_token(&mut self, appchain_id: AppchainId, token_id: AccountId);
    fn get_native_token(&self, appchain_id: AppchainId) -> Option<AccountId>;
}

#[near_bindgen]
impl NativeTokenManager for OctopusRelay {
    /// Register a new native token
    fn register_native_token(&mut self, appchain_id: AppchainId, token_id: AccountId) {
        self.assert_owner();
        assert!(
            self.appchain_native_tokens.get(&appchain_id).is_none(),
            "The native token of this appchain is already registered."
        );
        self.appchain_native_tokens.insert(&appchain_id, &token_id);
    }

    fn get_native_token(&self, appchain_id: AppchainId) -> Option<AccountId> {
        self.appchain_native_tokens.get(&appchain_id)
    }
}
