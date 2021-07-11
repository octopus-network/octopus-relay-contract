use crate::*;

#[near_bindgen]
impl OctopusRelay {
    pub fn pause_bridge_token(&mut self, token_id: AccountId) {
        let status = self
            .bridge_token_data_status
            .get(&token_id)
            .expect("bridge token not registered");
        assert!(
            status == BridgeStatus::Active,
            "The bridge is already paused"
        );
        self.assert_owner();
        self.bridge_token_data_status
            .insert(&token_id, &BridgeStatus::Paused);
    }

    pub fn resume_bridge_token(&mut self, token_id: AccountId) {
        let status = self
            .bridge_token_data_status
            .get(&token_id)
            .expect("bridge token not registered");
        assert!(status == BridgeStatus::Paused, "The bridge is active");
        self.assert_owner();
        self.bridge_token_data_status
            .insert(&token_id, &BridgeStatus::Active);
    }

    pub fn register_bridge_token(
        &mut self,
        token_id: AccountId,
        symbol: String,
        price: U128,
        decimals: u32,
    ) {
        self.assert_owner();
        assert!(
            !self.bridge_token_data_symbol.get(&token_id).is_some(),
            "The token_id is already registered"
        );
        assert!(
            !self.bridge_symbol_to_token.contains_key(&symbol),
            "The symbol is already registered"
        );

        self.bridge_token_data_symbol.insert(&token_id, &symbol);
        self.bridge_symbol_to_token.insert(&symbol, &token_id);
        self.bridge_token_data_status
            .insert(&token_id, &BridgeStatus::default());
        self.bridge_token_data_price
            .insert(&token_id, &price.into());
        self.bridge_token_data_decimals.insert(&token_id, &decimals);
    }

    pub fn set_bridge_permitted(
        &mut self,
        token_id: AccountId,
        appchain_id: AppchainId,
        permitted: bool,
    ) {
        // assert!(
        //     self.appchain_data_name.contains_key(&appchain_id),
        //     "Appchain not found"
        // );
        self.token_appchain_bridge_permitted
            .insert(&(token_id, appchain_id), &permitted);
    }

    pub fn set_oct_token_price(&mut self, price: U128) {
        self.assert_owner();
        self.oct_token_price = price.into();
    }

    pub fn set_bridge_token_price(&mut self, token_id: AccountId, price: U128) {
        self.assert_owner();
        self.bridge_token_data_price
            .insert(&token_id, &price.into());
    }

    pub fn get_bridge_token(&self, token_id: AccountId) -> Option<BridgeToken> {
        let bridge_token_symbol_option = self.bridge_token_data_symbol.get(&token_id);
        if bridge_token_symbol_option.is_some() {
            Some(BridgeToken {
                symbol: bridge_token_symbol_option.unwrap(),
                status: self.bridge_token_data_status.get(&token_id).unwrap(),
                price: self.bridge_token_data_price.get(&token_id).unwrap().into(),
                decimals: self.bridge_token_data_decimals.get(&token_id).unwrap(),
                token_id,
            })
        } else {
            None
        }
    }

    pub fn get_bridge_allowed_amount(&self, appchain_id: AppchainId, token_id: AccountId) -> U128 {
        let appchain_is_active = self
            .appchain_data_status
            .get(&appchain_id)
            .unwrap_or(AppchainStatus::Auditing)
            == AppchainStatus::Booting;
        assert!(appchain_is_active, "The appchain isn't at booting");

        let bridge_is_active = self
            .bridge_token_data_status
            .get(&token_id)
            .expect("This token isn't registered")
            == BridgeStatus::Active
            && self
                .token_appchain_bridge_permitted
                .get(&(token_id.clone(), appchain_id.clone()))
                .unwrap_or(false);
        assert!(bridge_is_active, "The bridge is paused or does not exist");

        let staked_balance = self
            .appchain_data_staked_balance
            .get(&appchain_id)
            .unwrap_or(0);
        let token_price = self.bridge_token_data_price.get(&token_id).unwrap();
        let limit_val = staked_balance / OCT_DECIMALS_BASE
            * self.oct_token_price
            * (self.bridge_limit_ratio as u128)
            / 10000;
        let mut total_used_val: Balance = 0;
        self.bridge_token_data_symbol.iter().for_each(|(bt_id, _)| {
            let bt_price = self.bridge_token_data_price.get(&bt_id).unwrap();
            let bt_locked = self
                .token_appchain_total_locked
                .get(&(bt_id.clone(), appchain_id.clone()))
                .unwrap_or(0);
            let bt_decimals = self.bridge_token_data_decimals.get(&bt_id).unwrap();
            let bt_decimals_base = (10 as u128).pow(bt_decimals);
            let used_val: Balance = bt_locked * bt_price / bt_decimals_base;
            total_used_val += used_val;
        });

        let rest_val = limit_val - total_used_val;
        let token_decimals = self.bridge_token_data_decimals.get(&token_id).unwrap();
        let token_decimals_base = (10 as u128).pow(token_decimals);

        let allowed_amount = rest_val * token_decimals_base / token_price;
        allowed_amount.into()
    }
}
