use crate::bridge_token_manager::BridgeTokenManager;
use crate::native_token_manager::NativeTokenManager;
use crate::proof_decoder::ProofDecoder;
use crate::types::{Message, MessagePayload};
use crate::*;

const STORAGE_DEPOSIT_AMOUNT: Balance = 1250000000000000000000;

/// Trait for bridging tokens between token contracts and appchains
pub trait TokenBridging {
    /// Lock token in relay contract for an appchain
    fn lock_token(
        &mut self,
        appchain_id: AppchainId,
        receiver: String,
        sender_id: AccountId,
        token_id: AccountId,
        amount: u128,
    ) -> U128;
    /// Unlock token in relay contract for an appchain
    fn unlock_token(
        &mut self,
        appchain_id: AppchainId,
        token_id: AccountId,
        sender: String,
        receiver_id: ValidAccountId,
        amount: U128,
    ) -> Promise;
    /// TODO! add comment for this function
    fn check_bridge_token_storage_deposit(
        &mut self,
        deposit: Balance,
        receiver_id: ValidAccountId,
        token_id: AccountId,
        appchain_id: AppchainId,
        amount: U128,
    ) -> Promise;
    fn create_unlock_promise(
        &mut self,
        deposit: Balance,
        receiver_id: ValidAccountId,
        token_id: AccountId,
        appchain_id: AppchainId,
        amount: U128,
        data: Vec<u8>,
    ) -> Promise;
    fn deposit_and_ft_transfer(
        &mut self,
        deposit: Balance,
        receiver_id: ValidAccountId,
        token_id: AccountId,
        appchain_id: AppchainId,
        amount: U128,
    ) -> Promise;
    /// Callback for checking bridge token storage deposit
    fn resolve_bridge_token_storage_deposit(
        &mut self,
        deposit: Balance,
        receiver_id: AccountId,
        amount: U128,
        token_id: AccountId,
    ) -> Promise;
    /// Callback for result of unlock token action
    fn resolve_unlock_token(&mut self, token_id: AccountId, appchain_id: AppchainId, amount: U128);
    fn resolve_mint_native_token(&mut self, appchain_id: AppchainId);
    fn mint_native_token(&mut self, appchain_id: AppchainId, receiver_id: AccountId, amount: U128);
    /// Burn native token on near, then mint on appchain
    fn burn_native_token(&mut self, appchain_id: AppchainId, receiver: AccountId, amount: U128);
    fn resolve_burn_native_token(
        &mut self,
        appchain_id: AppchainId,
        sender_id: AccountId,
        receiver: String,
        amount: u128,
    );
    fn relay(
        &mut self,
        appchain_id: AppchainId,
        encoded_messages: Vec<u8>,
        header_partial: Vec<u8>,
        leaf_proof: Vec<u8>,
        mmr_root: Vec<u8>,
    );
    fn execute(&mut self, messages: Vec<Message>, appchain_id: AppchainId, deposit: Balance);
}

#[near_bindgen]
impl TokenBridging for OctopusRelay {
    fn lock_token(
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

        // Try to create validators_history before lock_token.
        appchain_state.create_validators_history(false);
        appchain_state.lock_token(receiver, sender_id, token_id, amount);
        self.set_appchain_state(&appchain_id, &appchain_state);

        amount.into()
    }

    #[payable]
    fn unlock_token(
        &mut self,
        appchain_id: AppchainId,
        token_id: AccountId,
        sender: String,
        receiver_id: ValidAccountId,
        amount: U128,
    ) -> Promise {
        assert_self();
        let deposit: Balance = env::attached_deposit();
        let appchain_state = self.get_appchain_state(&appchain_id);
        let total_locked_amount = appchain_state.get_total_locked_amount_of(&token_id);
        assert!(
            total_locked_amount > 0,
            "You should lock token before unlock."
        );
        // assert!(
        //     deposit >= STORAGE_DEPOSIT_AMOUNT,
        //     "Attached deposit should be at least 0.00125."
        // );
        assert!(
            total_locked_amount >= amount.0,
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
                env::prepaid_gas() - 6 * SIMPLE_CALL_GAS,
            ))
    }

    fn check_bridge_token_storage_deposit(
        &mut self,
        deposit: Balance,
        receiver_id: ValidAccountId,
        token_id: AccountId,
        appchain_id: AppchainId,
        amount: U128,
    ) -> Promise {
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(data) => {
                let unlock_promise = self.create_unlock_promise(
                    deposit,
                    receiver_id,
                    token_id.clone(),
                    appchain_id.clone(),
                    amount,
                    data,
                );
                unlock_promise.then(ext_self::resolve_unlock_token(
                    token_id,
                    appchain_id.clone(),
                    amount,
                    &env::current_account_id(),
                    NO_DEPOSIT,
                    GAS_FOR_FT_TRANSFER_CALL,
                ))
            }
            PromiseResult::Failed => unreachable!(),
        }
    }

    fn create_unlock_promise(
        &mut self,
        deposit: Balance,
        receiver_id: ValidAccountId,
        token_id: AccountId,
        appchain_id: AppchainId,
        amount: U128,
        data: Vec<u8>,
    ) -> Promise {
        assert_self();
        if let Ok(storage_balance) = near_sdk::serde_json::from_slice::<StorageBalance>(&data) {
            if storage_balance.total.0 > 0 {
                return ext_token::ft_transfer(
                    receiver_id.clone().into(),
                    amount,
                    None,
                    &token_id,
                    1,
                    FT_TRANSFER_GAS,
                )
                .then(Promise::new(env::signer_account_id()).transfer(deposit));
            }
        }
        self.deposit_and_ft_transfer(
            deposit,
            receiver_id,
            token_id.clone(),
            appchain_id.clone(),
            amount,
        )
    }

    fn deposit_and_ft_transfer(
        &mut self,
        deposit: Balance,
        receiver_id: ValidAccountId,
        token_id: AccountId,
        appchain_id: AppchainId,
        amount: U128,
    ) -> Promise {
        ext_token::storage_deposit(
            Some(receiver_id.clone()),
            None,
            &token_id,
            deposit,
            SIMPLE_CALL_GAS,
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
    }

    fn resolve_bridge_token_storage_deposit(
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
                    ext_token::ft_transfer(receiver_id, amount, None, &token_id, 1, FT_TRANSFER_GAS)
                } else {
                    Promise::new(signer).transfer(deposit)
                }
            }
            PromiseResult::Failed => Promise::new(signer).transfer(deposit),
        }
    }

    fn resolve_unlock_token(&mut self, token_id: AccountId, appchain_id: AppchainId, amount: U128) {
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let mut appchain_state = self.get_appchain_state(&appchain_id);
                appchain_state.unlock_token(token_id, amount.0);
                appchain_state.increase_message_nonce();
                self.set_appchain_state(&appchain_id, &appchain_state);
            }
            PromiseResult::Failed => unreachable!(),
        }
    }

    #[payable]
    fn mint_native_token(&mut self, appchain_id: AppchainId, receiver_id: AccountId, amount: U128) {
        let deposit: Balance = env::attached_deposit();
        assert!(
            deposit == STORAGE_DEPOSIT_AMOUNT,
            "Attached deposit should be 0.00125."
        );
        let native_token_id = self
            .get_native_token(appchain_id.clone())
            .expect("Native token is not registered.");
        ext_token::mint(
            receiver_id,
            amount,
            &native_token_id,
            deposit,
            GAS_FOR_FT_TRANSFER_CALL,
        )
        .then(ext_self::resolve_mint_native_token(
            appchain_id,
            &env::current_account_id(),
            0,
            GAS_FOR_FT_TRANSFER_CALL,
        ));
    }

    fn resolve_mint_native_token(&mut self, appchain_id: AppchainId) {
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let mut appchain_state = self.get_appchain_state(&appchain_id);
                appchain_state.increase_message_nonce();
                self.set_appchain_state(&appchain_id, &appchain_state);
            }
            PromiseResult::Failed => unreachable!(),
        }
    }

    fn relay(
        &mut self,
        appchain_id: AppchainId,
        encoded_messages: Vec<u8>,
        header_partial: Vec<u8>,
        leaf_proof: Vec<u8>,
        mmr_root: Vec<u8>,
    ) {
        let deposit: Balance = env::attached_deposit();
        let appchain_state = self.get_appchain_state(&appchain_id);
        let verified: bool = appchain_state.prover.verify(
            encoded_messages.clone(),
            header_partial.clone(),
            leaf_proof.clone(),
            mmr_root.clone(),
        );
        assert!(verified, "verification failed");
        let messages = self.decode(encoded_messages, header_partial, leaf_proof, mmr_root);
        self.execute(messages, appchain_id, deposit);
    }

    fn execute(
        &mut self,
        messages: Vec<Message>,
        appchain_id: AppchainId,
        remaining_deposit: Balance,
    ) {
        if messages.len() > 0 {
            let appchain_state = self.get_appchain_state(&appchain_id);
            let message = messages.get(0).unwrap();
            // assert_eq!(
            //     message.nonce,
            //     appchain_state.message_nonce + 1,
            //     "nonce not correct"
            // );
            let execution_promise;
            let next_messages = (&messages[1..messages.len()]).to_vec();
            let next_remaining_deposit = remaining_deposit - STORAGE_DEPOSIT_AMOUNT;
            match &message.payload {
                MessagePayload::BurnAsset(p) => {
                    execution_promise = ext_self::unlock_token(
                        appchain_id.clone(),
                        p.token_id.clone(),
                        p.sender.clone(),
                        p.receiver_id.clone(),
                        p.amount,
                        &env::current_account_id(),
                        STORAGE_DEPOSIT_AMOUNT,
                        COMPLEX_CALL_GAS,
                    );
                }
                MessagePayload::Lock(p) => {
                    execution_promise = ext_self::mint_native_token(
                        appchain_id.clone(),
                        p.receiver_id.clone().into(),
                        p.amount,
                        &env::current_account_id(),
                        STORAGE_DEPOSIT_AMOUNT,
                        2 * SINGLE_CALL_GAS,
                    );
                }
            }
            execution_promise.then(ext_self::execute(
                next_messages,
                appchain_id.clone(),
                next_remaining_deposit,
                &env::current_account_id(),
                NO_DEPOSIT,
                COMPLEX_CALL_GAS + SIMPLE_CALL_GAS,
            ));
        }
    }

    #[payable]
    fn burn_native_token(&mut self, appchain_id: AppchainId, receiver: String, amount: U128) {
        assert_one_yocto();
        let native_token_id = self
            .get_native_token(appchain_id.clone())
            .expect("Native token is not registered.");

        let sender_id = env::signer_account_id();
        ext_token::burn(
            sender_id.clone(),
            amount,
            &native_token_id,
            1,
            GAS_FOR_FT_TRANSFER_CALL,
        )
        .then(ext_self::resolve_burn_native_token(
            appchain_id,
            sender_id,
            receiver,
            amount.0,
            &env::current_account_id(),
            0,
            SINGLE_CALL_GAS,
        ));
    }

    fn resolve_burn_native_token(
        &mut self,
        appchain_id: AppchainId,
        sender_id: AccountId,
        receiver: String,
        amount: u128,
    ) {
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let mut appchain_state = self.get_appchain_state(&appchain_id);

                // Try to create validators_history before burn_native_token.
                appchain_state.create_validators_history(false);
                appchain_state.burn_native_token(receiver, sender_id, amount);
                self.set_appchain_state(&appchain_id, &appchain_state);
            }
            PromiseResult::Failed => unreachable!(),
        }
    }
}
