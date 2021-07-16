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

        // update_validator_set for checking if there is a validator_set fact
        // before new lock_token fact be created.
        // self.update_validator_set(appchain_id.clone());

        let total_locked: Balance = self
            .token_appchain_total_locked
            .get(&(token_id.clone(), appchain_id.clone()))
            .unwrap_or(0);
        let next_total_locked = total_locked + u128::from(amount);
        self.token_appchain_total_locked.insert(
            &(token_id.clone(), appchain_id.clone()),
            &(next_total_locked),
        );

        let seq_num = self.appchain_data_fact_sets_len.get(&appchain_id).unwrap();
        self.appchain_data_fact_set.insert(
            &(appchain_id.clone(), seq_num),
            &Fact::LockToken(Locked {
                seq_num,
                token_id,
                sender_id,
                receiver,
                amount: amount.into(),
            }),
        );
        self.appchain_data_fact_sets_len
            .insert(&appchain_id, &(seq_num + 1));
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
        // prover todo
        let token_appchain_total_locked = self
            .token_appchain_total_locked
            .get(&(token_id.clone(), appchain_id.clone()))
            .expect("You should lock token before unlock.");

        assert!(
            deposit >= 1250000000000000000000,
            "Attached deposit should be at least 0.00125."
        );
        assert!(
            token_appchain_total_locked >= amount.0,
            "Insufficient locked balance!"
        );

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
                let total_locked: Balance = self
                    .token_appchain_total_locked
                    .get(&(token_id.clone(), appchain_id.clone()))
                    .unwrap_or(0);
                let next_total_locked = total_locked - u128::from(amount);
                self.token_appchain_total_locked
                    .insert(&(token_id, appchain_id), &(next_total_locked));
            }
            PromiseResult::Failed => {}
        }
    }

    pub fn get_facts(&self, appchain_id: AppchainId, start: SeqNum, limit: SeqNum) -> Vec<Fact> {
        let fact_sets_len = self.appchain_data_fact_sets_len.get(&appchain_id).unwrap();
        let end = std::cmp::min(start + limit, fact_sets_len);
        let fact_sets: Vec<Fact> = (start..end)
            .map(|index| {
                let fact = self
                    .appchain_data_fact_set
                    .get(&(appchain_id.clone(), index))
                    .unwrap();
                fact
            })
            .collect();
        // Commented out the following logic because of the refactoring of relay contract.
        // Now the fact collection only for facts of cross chain tokens transformation.
        //
        // let next_validator_set_option = self.next_validator_set(appchain_id.clone(), false);
        // if next_validator_set_option.is_some() {
        //     let next_validator_set = next_validator_set_option.unwrap();
        //     if next_validator_set.seq_num < end && next_validator_set.seq_num >= start {
        //         fact_sets.push(Fact::UpdateValidatorSet(next_validator_set));
        //     }
        // }
        fact_sets
    }
}
