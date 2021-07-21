use crate::relayed_bridge_token::BridgingStatus;
use crate::*;

const UNREGISTERED_TOKEN_ID: &'static str = "Unregistered token id";

/// Interfaces for manager bridge tokens
pub trait BridgeTokenManager {
    /// Register a new bridge token
    fn register_bridge_token(
        &mut self,
        token_id: AccountId,
        symbol: String,
        price: U128,
        decimals: u32,
    );
    /// Pause bridging a token
    fn pause_bridge_token(&mut self, token_id: AccountId);
    /// Resume bridging a token
    fn resume_bridge_token(&mut self, token_id: AccountId);
    /// Set bridging permission of token to an appchain
    fn set_bridge_permitted(
        &mut self,
        token_id: AccountId,
        appchain_id: AppchainId,
        permitted: bool,
    );
    /// Set the price of a token
    ///
    /// This function should be called by an oracle which can offer the price of certain token.
    fn set_bridge_token_price(&mut self, token_id: AccountId, price: U128);
    /// Get information of a bridge token
    fn get_bridge_token(&self, token_id: AccountId) -> Option<BridgeToken>;
    /// Get permitted amount of a token
    ///
    /// The result is calculated by the total price of all staked balance of OCT token in an appchain
    /// and the price of certain token.
    fn get_bridge_allowed_amount(&self, appchain_id: AppchainId, token_id: AccountId) -> U128;
}

#[near_bindgen]
impl BridgeTokenManager for OctopusRelay {
    /// Pause bridging a token
    fn pause_bridge_token(&mut self, token_id: AccountId) {
        self.assert_owner();
        let mut bridge_token = self
            .get_relayed_bridge_token(&token_id)
            .expect(UNREGISTERED_TOKEN_ID);
        assert!(
            bridge_token.bridging_status() == BridgingStatus::Activated,
            "The bridge is already paused"
        );
        bridge_token.pause_bridging();
        self.set_relayed_bridge_token(&bridge_token);
    }
    /// Resume bridging a token
    fn resume_bridge_token(&mut self, token_id: AccountId) {
        self.assert_owner();
        let mut bridge_token = self
            .get_relayed_bridge_token(&token_id)
            .expect(UNREGISTERED_TOKEN_ID);
        assert!(
            bridge_token.bridging_status() == BridgingStatus::Paused,
            "Bridge is already activated."
        );
        bridge_token.activate_bridging();
        self.set_relayed_bridge_token(&bridge_token);
    }
    /// Register a new bridge token
    fn register_bridge_token(
        &mut self,
        token_id: AccountId,
        symbol: String,
        price: U128,
        decimals: u32,
    ) {
        self.assert_owner();
        assert!(
            self.bridge_tokens.get(&token_id).is_none(),
            "The token_id is already registered"
        );
        self.bridge_tokens.insert(
            &token_id,
            &LazyOption::new(
                StorageKey::RelayedBridgeToken {
                    token_id: token_id.clone(),
                }
                .into_bytes(),
                Some(&RelayedBridgeToken::new(
                    token_id.clone(),
                    symbol.clone(),
                    BridgingStatus::default(),
                    price,
                    decimals,
                )),
            ),
        );
    }
    /// Set bridging permission of token to an appchain
    fn set_bridge_permitted(
        &mut self,
        token_id: AccountId,
        appchain_id: AppchainId,
        permitted: bool,
    ) {
        self.assert_owner();
        let mut bridge_token = self
            .get_relayed_bridge_token(&token_id)
            .expect(UNREGISTERED_TOKEN_ID);
        bridge_token.set_bridging_permission(&appchain_id, &permitted);
        self.set_relayed_bridge_token(&bridge_token);
    }
    /// Set the price of a token
    ///
    /// This function should be called by an oracle which can offer the price of certain token.
    fn set_bridge_token_price(&mut self, token_id: AccountId, price: U128) {
        self.assert_owner();
        let mut bridge_token = self
            .get_relayed_bridge_token(&token_id)
            .expect(UNREGISTERED_TOKEN_ID);
        bridge_token.set_price(&price);
        self.set_relayed_bridge_token(&bridge_token);
    }
    /// Get information of a bridge token
    fn get_bridge_token(&self, token_id: AccountId) -> Option<BridgeToken> {
        self.get_relayed_bridge_token(&token_id)
            .map(|token| token.to_bridge_token())
    }
    /// Get permitted amount of a token
    ///
    /// The result is calculated by the total price of all staked balance of OCT token in an appchain
    /// and the price of certain token.
    fn get_bridge_allowed_amount(&self, appchain_id: AppchainId, token_id: AccountId) -> U128 {
        let appchain_state = self.get_appchain_state(&appchain_id);
        assert_eq!(
            appchain_state.status,
            AppchainStatus::Booting,
            "The appchain isn't at booting"
        );
        let bridge_token = self
            .get_relayed_bridge_token(&token_id)
            .expect(UNREGISTERED_TOKEN_ID);
        assert!(
            bridge_token.bridging_status() == BridgingStatus::Activated
                && bridge_token.is_permitted_of(&appchain_id),
            "The bridge is paused or does not exist"
        );

        let staked_balance = appchain_state.staked_balance;
        let token_price = bridge_token.price().0;
        let limit_val = staked_balance / OCT_DECIMALS_BASE
            * self.oct_token_price
            * (self.bridge_limit_ratio as u128)
            / 10000;
        let mut total_used_val: Balance = 0;
        self.bridge_tokens
            .values_as_vector()
            .iter()
            .map(|f| f.get().unwrap())
            .for_each(|token| {
                let appchain_state = self.get_appchain_state(&appchain_id);
                let bt_price = token.price().0;
                let bt_locked = appchain_state.get_total_locked_amount_of(&token_id);
                let bt_decimals = token.decimals();
                let bt_decimals_base = (10 as u128).pow(bt_decimals);
                let used_val: Balance = bt_locked * bt_price / bt_decimals_base;
                total_used_val += used_val;
            });

        if total_used_val >= limit_val {
            return 0.into();
        }
        let rest_val = limit_val - total_used_val;
        let token_decimals = bridge_token.decimals();
        let token_decimals_base = (10 as u128).pow(token_decimals);

        let allowed_amount = rest_val * token_decimals_base / token_price;
        allowed_amount.into()
    }
}

#[near_bindgen]
impl OctopusRelay {
    /// Set the price of OCT token
    ///
    /// This function should be called by an oracle which can offer the price of OCT token.
    pub fn set_oct_token_price(&mut self, price: U128) {
        self.assert_owner();
        self.oct_token_price = price.into();
    }
    // Get relayed bridge token by id
    fn get_relayed_bridge_token(&self, token_id: &AccountId) -> Option<RelayedBridgeToken> {
        self.bridge_tokens
            .get(&token_id)
            .expect(UNREGISTERED_TOKEN_ID)
            .get()
    }
    // Set relayed bridge token
    fn set_relayed_bridge_token(&mut self, bridge_token: &RelayedBridgeToken) {
        self.bridge_tokens
            .get(&bridge_token.id())
            .expect(UNREGISTERED_TOKEN_ID)
            .set(bridge_token);
    }
}
