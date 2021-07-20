use crate::*;

#[near_bindgen]
impl OctopusRelay {
    pub fn lock_token(
        &mut self,
        appchain_id: AppchainId,
        receiver: String,
        sender_id: AccountId,
        token_id: AccountId,
        amount: u128,
    ) -> U128 {
        let allowed_amount: u128 = self
            .get_bridge_allowed_amount(appchain_id.clone(), token_id.clone())
            .into();
        assert!(
            allowed_amount >= amount.into(),
            "Bridge not allowed: Insufficient staked amount"
        );

        let mut appchain_state = self.get_appchain_state(&appchain_id);
        appchain_state.lock_token(receiver, sender_id, token_id, amount);
        self.set_appchain_state(&appchain_id, &appchain_state);

        amount.into()
    }

    #[payable]
    pub fn unlock_token(
        &mut self,
        appchain_id: AppchainId,
        token_id: AccountId,
        sender: String,
        receiver_id: ValidAccountId,
        amount: U128,
    ) {
        let deposit: Balance = env::attached_deposit();
        let appchain_state = self.get_appchain_state(&appchain_id);
        let total_locked_amount = appchain_state.get_total_locked_amount_of(&token_id);

        assert!(
            total_locked_amount > 0,
            "You should lock token before unlock."
        );
        assert!(
            deposit >= 1250000000000000000000,
            "Attached deposit should be at least 0.00125."
        );
        assert!(
            total_locked_amount >= amount.0,
            "Insufficient locked balance!"
        );

        // TODO: madwiki
        // assert!(
        //     appchain_state.prover.verify(encoded_messages, header_partial, leaf_proof, mmr_root),
        //     "Invalid cross-chain message."
        // );

        // let (appchain_id, token_id, sender, receiver_id, amount) = Decode::decode(encoded_messages);

        ext_token::storage_balance_of(receiver_id.clone(), &token_id, deposit, SIMPLE_CALL_GAS)
            .then(ext_self::check_bridge_token_storage_deposit(
                deposit,
                receiver_id,
                token_id,
                appchain_id,
                amount,
                &env::current_account_id(),
                NO_DEPOSIT,
                env::prepaid_gas() - SINGLE_CALL_GAS,
            ));
    }

    pub fn check_bridge_token_storage_deposit(
        &mut self,
        deposit: Balance,
        receiver_id: ValidAccountId,
        token_id: AccountId,
        appchain_id: AppchainId,
        amount: U128,
    ) {
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(data) => {
                if let Ok(storage_balance) =
                    near_sdk::serde_json::from_slice::<StorageBalance>(&data)
                {
                    if storage_balance.total.0 > 0 {
                        ext_token::ft_transfer(
                            receiver_id.clone().into(),
                            amount,
                            None,
                            &token_id,
                            1,
                            GAS_FOR_FT_TRANSFER_CALL,
                        )
                        .then(Promise::new(env::signer_account_id()).transfer(deposit));
                    }
                } else {
                    ext_token::storage_deposit(
                        Some(receiver_id.clone()),
                        None,
                        &token_id,
                        deposit,
                        GAS_FOR_FT_TRANSFER_CALL,
                    )
                    .then(ext_self::resolve_bridge_token_storage_deposit(
                        deposit,
                        receiver_id.clone(),
                        amount,
                        token_id.clone(),
                        &env::current_account_id(),
                        NO_DEPOSIT,
                        SINGLE_CALL_GAS,
                    ))
                    .then(ext_self::resolve_unlock_token(
                        token_id,
                        appchain_id.clone(),
                        amount,
                        &env::current_account_id(),
                        NO_DEPOSIT,
                        SINGLE_CALL_GAS,
                    ));
                }
            }
            PromiseResult::Failed => {}
        }
    }

    pub fn resolve_bridge_token_storage_deposit(
        &mut self,
        deposit: Balance,
        receiver_id: AccountId,
        amount: U128,
        token_id: AccountId,
    ) -> Promise {
        assert_self();
        let signer = env::signer_account_id();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(data) => {
                if let Ok(storage_balance) =
                    near_sdk::serde_json::from_slice::<StorageBalance>(&data)
                {
                    let refund = deposit - storage_balance.total.0;
                    if refund > 0 {
                        Promise::new(signer).transfer(refund);
                    }
                    ext_token::ft_transfer(
                        receiver_id,
                        amount,
                        None,
                        &token_id,
                        1,
                        GAS_FOR_FT_TRANSFER_CALL,
                    )
                } else {
                    Promise::new(signer).transfer(deposit)
                }
            }
            PromiseResult::Failed => Promise::new(signer).transfer(deposit),
        }
    }

    pub fn resolve_unlock_token(
        &mut self,
        token_id: AccountId,
        appchain_id: AppchainId,
        amount: U128,
    ) {
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let mut appchain_state = self.get_appchain_state(&appchain_id);
                appchain_state.unlock_token(token_id, amount.0);
                self.set_appchain_state(&appchain_id, &appchain_state);
            }
            PromiseResult::Failed => {}
        }
    }

    pub fn get_facts(&self, appchain_id: AppchainId, start: SeqNum, limit: SeqNum) -> Vec<Fact> {
        let appchain_state = self.get_appchain_state(&appchain_id);
        appchain_state.get_facts(&start, &limit)
    }
}
