use crate::types::{
    Appchain, AppchainStatus, BridgeStatus, BridgeToken, Delegator, Fact, LiteValidator, Locked,
    StorageBalance, Validator, ValidatorSet,
};
use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_self, env, ext_contract, log, near_bindgen, wee_alloc, AccountId, Balance, BlockHeight,
    Promise, PromiseOrValue, PromiseResult,
};

pub trait AppchainPipeline {
    /// Finish auditing of an appchain (change its status to `AppchainStatus::Voting`).
    /// Can only be called by the owner of Octopus relay.
    fn pass_appchain(&mut self, appchain_id: AppchainId);
    /// Select an appchain for staging (change its status to `AppchainStatus::Staging`).
    /// Can only be called by the owner of Octopus relay.
    fn appchain_go_staging(&mut self, appchain_id: AppchainId);
    /// Remove an appchain from pipeline.
    /// Can only be called by the owner of Octopus relay.
    fn remove_appchain(&mut self, appchain_id: AppchainId);
    /// Callback of function `remove_appchain`
    /// Can only be called by the owner of Octopus relay.
    fn resolve_remove_appchain(&mut self, appchain_id: AppchainId);
    /// Activate an appchain
    /// If success, the status of booting appchain should change to `AppchainStatus::Booting`.
    fn activate_appchain(
        &mut self,
        appchain_id: AppchainId,
        boot_nodes: String,
        rpc_endpoint: String,
        chain_spec_url: String,
        chain_spec_hash: String,
        chain_spec_raw_url: String,
        chain_spec_raw_hash: String,
    ) -> PromiseOrValue<Option<AppchainStatus>>;
    /// Callback of function `activate_appchain`
    /// Can only be called by the owner of Octopus relay.
    fn resolve_activate_appchain(
        &mut self,
        appchain_id: AppchainId,
        boot_nodes: String,
        rpc_endpoint: String,
        chain_spec_url: String,
        chain_spec_hash: String,
        chain_spec_raw_url: String,
        chain_spec_raw_hash: String,
    ) -> Option<AppchainStatus>;
    /// Freeze an appchain
    fn freeze_appchain(&mut self, appchain_id: AppchainId);
}

#[near_bindgen]
impl AppchainPipeline for OctopusRelay {
    //
    fn remove_appchain(&mut self, appchain_id: AppchainId) {
        self.assert_owner();
        let appchain_metadata = self
            .appchain_metadatas
            .get(&appchain_id)
            .expect("Appchain not found");
        let appchain_state = self
            .appchain_states
            .get(&appchain_id)
            .expect("Appchain not found");
        assert_eq!(
            appchain_state.status,
            AppchainStatus::Auditing,
            "appchain can only be removed in auditing status"
        );

        let bond_tokens = appchain_metadata.bond_tokens;
        let account_id = appchain_metadata.founder_id;

        ext_token::ft_transfer(
            account_id,
            (bond_tokens / 10).into(),
            None,
            &self.token_contract_id,
            1,
            GAS_FOR_FT_TRANSFER_CALL,
        )
        .then(ext_self::resolve_remove_appchain(
            appchain_id.clone(),
            &env::current_account_id(),
            NO_DEPOSIT,
            env::prepaid_gas() / 2,
        ));
    }
    //
    fn resolve_remove_appchain(&mut self, appchain_id: AppchainId) {
        assert_self();
        // Update state
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                self.appchain_metadatas.remove(&appchain_id);
                self.appchain_states.remove(&appchain_id);
            }
            PromiseResult::Failed => {}
        }
    }
    //
    fn pass_appchain(&mut self, appchain_id: AppchainId) {
        self.assert_owner();
        let mut appchain_state = self
            .appchain_states
            .get(&appchain_id)
            .expect("Appchain not found");
        assert_eq!(
            &appchain_state.status,
            &AppchainStatus::Auditing,
            "Appchain is not in auditing."
        );
        appchain_state.pass_auditing();
    }
    //
    fn appchain_go_staging(&mut self, appchain_id: AppchainId) {
        self.assert_owner();
        let mut appchain_state = self
            .appchain_states
            .get(&appchain_id)
            .expect("Appchain not found");
        assert_eq!(
            &appchain_state.status,
            &AppchainStatus::Voting,
            "Appchain is not in queue."
        );
        appchain_state.go_staging();
    }
    //
    fn activate_appchain(
        &mut self,
        appchain_id: AppchainId,
        boot_nodes: String,
        rpc_endpoint: String,
        chain_spec_url: String,
        chain_spec_hash: String,
        chain_spec_raw_url: String,
        chain_spec_raw_hash: String,
    ) -> PromiseOrValue<Option<AppchainStatus>> {
        self.assert_owner();
        let appchain_metadata = self
            .appchain_metadatas
            .get(&appchain_id)
            .expect("Appchain not found");
        let appchain_state = self
            .appchain_states
            .get(&appchain_id)
            .expect("Appchain not found");
        assert_eq!(
            appchain_state.status,
            AppchainStatus::Staging,
            "Appchain is not in staging."
        );
        // Check validators
        assert!(
            appchain_state.validators.len().try_into().unwrap_or(0)
                >= self.appchain_minimum_validators,
            "Insufficient number of appchain validators"
        );

        let account_id = appchain_metadata.founder_id;
        let bond_tokens = appchain_metadata.bond_tokens;
        if bond_tokens > 0 {
            ext_token::ft_transfer(
                account_id,
                (bond_tokens / 10).into(),
                None,
                &self.token_contract_id,
                1,
                GAS_FOR_FT_TRANSFER_CALL,
            )
            .then(ext_self::resolve_activate_appchain(
                appchain_id,
                boot_nodes,
                rpc_endpoint,
                chain_spec_url,
                chain_spec_hash,
                chain_spec_raw_url,
                chain_spec_raw_hash,
                &env::current_account_id(),
                NO_DEPOSIT,
                env::prepaid_gas() / 2,
            ))
            .into()
        } else {
            PromiseOrValue::Value(self.internal_activate_appchain(
                appchain_id,
                boot_nodes,
                rpc_endpoint,
                chain_spec_url,
                chain_spec_hash,
                chain_spec_raw_url,
                chain_spec_raw_hash,
            ))
        }
    }
    //
    fn resolve_activate_appchain(
        &mut self,
        appchain_id: AppchainId,
        boot_nodes: String,
        rpc_endpoint: String,
        chain_spec_url: String,
        chain_spec_hash: String,
        chain_spec_raw_url: String,
        chain_spec_raw_hash: String,
    ) -> Option<AppchainStatus> {
        // Update state
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => self.internal_activate_appchain(
                appchain_id,
                boot_nodes,
                rpc_endpoint,
                chain_spec_url,
                chain_spec_hash,
                chain_spec_raw_url,
                chain_spec_raw_hash,
            ),
            PromiseResult::Failed => Option::from(AppchainStatus::Staging),
        }
    }
    //
    fn freeze_appchain(&mut self, appchain_id: AppchainId) {
        self.assert_owner();
        let mut appchain_state = self
            .appchain_states
            .get(&appchain_id)
            .expect("Appchain not found");
        // Check status
        assert_eq!(
          appchain_state.status, AppchainStatus::Booting,
            "Appchain status incorrect"
        );

        // Update state
        appchain_state.freeze();
    }
}

impl OctopusRelay {
    //
    fn internal_activate_appchain(
        &mut self,
        appchain_id: AppchainId,
        boot_nodes: String,
        rpc_endpoint: String,
        chain_spec_url: String,
        chain_spec_hash: String,
        chain_spec_raw_url: String,
        chain_spec_raw_hash: String,
    ) -> Option<AppchainStatus> {
        let mut appchain_metadata = self
            .appchain_metadatas
            .get(&appchain_id)
            .expect("Appchain not found");
        appchain_metadata.update_booting_info(
            boot_nodes,
            rpc_endpoint,
            chain_spec_url,
            chain_spec_hash,
            chain_spec_raw_url,
            chain_spec_raw_hash,
        );
        let mut appchain_state = self
            .appchain_states
            .get(&appchain_id)
            .expect("Appchain not found");
        appchain_state.boot();
        Option::from(appchain_state.status)
    }
}
