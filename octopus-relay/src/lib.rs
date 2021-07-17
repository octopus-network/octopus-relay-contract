pub mod appchain;
pub mod bridge;
pub mod bridging;
pub mod pipeline;
pub mod storage_key;
pub mod storage_migration;
pub mod types;

use std::convert::{From, TryInto};

use crate::storage_key::StorageKey;
// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use crate::types::{
    Appchain, AppchainStatus, BridgeStatus, BridgeToken, Delegator, Fact, Locked, StorageBalance,
    Validator, ValidatorSet,
};
use appchain::metadata::AppchainMetadata;
use appchain::state::AppchainState;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, Vector};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_self, env, ext_contract, log, near_bindgen, wee_alloc, AccountId, Balance, BlockHeight,
    Promise, PromiseOrValue, PromiseResult,
};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const NO_DEPOSIT: Balance = 0;
const GAS_FOR_FT_TRANSFER_CALL: u64 = 35_000_000_000_000;
const SINGLE_CALL_GAS: u64 = 50_000_000_000_000;
const COMPLEX_CALL_GAS: u64 = 70_000_000_000_000;
const SIMPLE_CALL_GAS: u64 = 5_000_000_000_000;
const OCT_DECIMALS_BASE: Balance = 1000_000_000_000_000_000_000_000;

const APPCHAIN_METADATA_NOT_FOUND: &'static str = "Appchain metadata not found";
const APPCHAIN_STATE_NOT_FOUND: &'static str = "Appchain state not found";

// 20 minutes
const VALIDATOR_SET_CYCLE: u64 = 20 * 60000000000;
// const VALIDATOR_SET_CYCLE: u64 = 86400000000000;

pub type AppchainId = String;
pub type ValidatorId = String;
pub type DelegatorId = String;
pub type SeqNum = u32;

// Structs in Rust are similar to other languages, and may include impl keyword as shown below
// Note: the names of the structs are not important when calling the smart contract, but the function names are
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct OctopusRelay {
    pub version: u32,
    pub token_contract_id: AccountId,
    pub appchain_minimum_validators: u32,
    pub minimum_staking_amount: Balance,
    pub total_staked_balance: Balance,
    pub appchain_id_list: Vector<AppchainId>,

    // Using lookupmap instead of vectors to clean up historical data
    // in the future without affecting the sequence numbers
    pub appchain_data_fact_sets_len: LookupMap<AppchainId, SeqNum>,
    pub appchain_data_fact_set: LookupMap<(AppchainId, SeqNum), Fact>,

    pub bridge_token_data_symbol: UnorderedMap<AccountId, String>,
    pub bridge_symbol_to_token: LookupMap<String, AccountId>,
    pub bridge_token_data_status: LookupMap<AccountId, BridgeStatus>,
    pub bridge_token_data_price: LookupMap<AccountId, Balance>,
    pub bridge_token_data_decimals: LookupMap<AccountId, u32>,
    pub bridge_limit_ratio: u16, // 100 as 1%
    pub owner: AccountId,
    pub oct_token_price: u128, // 1_000_000 as 1usd

    pub token_appchain_bridge_permitted: LookupMap<(AccountId, AppchainId), bool>,
    pub token_appchain_total_locked: LookupMap<(AccountId, AppchainId), Balance>,

    /// Collection of metadata of all appchains
    pub appchain_metadatas: UnorderedMap<AppchainId, LazyOption<AppchainMetadata>>,
    /// Collection of state data of all appchains
    pub appchain_states: UnorderedMap<AppchainId, LazyOption<AppchainState>>,
}

#[ext_contract(ext_self)]
pub trait ExtOctopusRelay {
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
    fn resolve_remove_appchain(&mut self, appchain_id: AppchainId);
    fn resolve_remove_validator(
        &mut self,
        appchain_id: AppchainId,
        validator_id: ValidatorId,
        amount: U128,
    );
    fn resolve_unlock_token(&mut self, token_id: AccountId, appchain_id: AppchainId, amount: U128);
    fn resolve_bridge_token_storage_deposit(
        &mut self,
        deposit: u128,
        receiver_id: ValidAccountId,
        amount: U128,
        token_id: AccountId,
    );
    fn check_bridge_token_storage_deposit(
        &mut self,
        deposit: Balance,
        receiver_id: ValidAccountId,
        token_id: AccountId,
        appchain_id: AppchainId,
        amount: U128,
    );
}

#[ext_contract(ext_token)]
pub trait ExtContract {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn storage_deposit(
        &mut self,
        account_id: Option<ValidAccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;
    fn storage_balance_of(&self, account_id: ValidAccountId) -> Option<StorageBalance>;
}

impl Default for OctopusRelay {
    fn default() -> Self {
        env::panic(b"The contract should be initialized before usage")
    }
}

#[near_bindgen]
impl OctopusRelay {
    #[init]
    pub fn new(
        token_contract_id: AccountId,
        appchain_minimum_validators: u32,
        minimum_staking_amount: U128,
        bridge_limit_ratio: u16,
        oct_token_price: U128,
    ) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized");
        assert_self();
        Self {
            version: 0,
            token_contract_id,
            total_staked_balance: 0,
            appchain_minimum_validators,
            minimum_staking_amount: minimum_staking_amount.0,
            appchain_id_list: Vector::new(b"ail".to_vec()),

            appchain_data_fact_sets_len: LookupMap::new(b"fsl".to_vec()),
            appchain_data_fact_set: LookupMap::new(b"fs".to_vec()),

            bridge_token_data_symbol: UnorderedMap::new(b"ts".to_vec()),
            bridge_symbol_to_token: LookupMap::new(b"stt".to_vec()),
            bridge_token_data_status: LookupMap::new(b"tst".to_vec()),
            bridge_token_data_price: LookupMap::new(b"tp".to_vec()),
            bridge_token_data_decimals: LookupMap::new(b"td".to_vec()),

            owner: env::current_account_id(),
            bridge_limit_ratio,
            oct_token_price: oct_token_price.into(),

            token_appchain_bridge_permitted: LookupMap::new(b"tas".to_vec()),
            token_appchain_total_locked: LookupMap::new(b"tab".to_vec()),

            appchain_metadatas: UnorderedMap::new(StorageKey::AppchainMetadatas.into_bytes()),
            appchain_states: UnorderedMap::new(StorageKey::AppchainStates.into_bytes()),
        }
    }

    pub fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // Verifying that we were called by fungible token contract that we expect.
        log!(
            "in {} tokens from @{} ft_on_transfer, msg = {}",
            amount.0,
            sender_id.as_ref(),
            msg
        );

        let msg_vec: Vec<String> = msg.split(",").map(|s| s.to_string()).collect();

        match msg_vec.get(0).unwrap().as_str() {
            "register_appchain" => {
                assert_eq!(
                    &env::predecessor_account_id(),
                    &self.token_contract_id,
                    "Only supports the OCT token contract"
                );
                assert_eq!(msg_vec.len(), 7, "params length wrong!");
                self.register_appchain(
                    msg_vec.get(1).unwrap().to_string(),
                    msg_vec.get(2).unwrap().to_string(),
                    msg_vec.get(3).unwrap().to_string(),
                    msg_vec.get(4).unwrap().to_string(),
                    msg_vec.get(5).unwrap().to_string(),
                    msg_vec.get(6).unwrap().to_string(),
                    amount.into(),
                );
                PromiseOrValue::Value(0.into())
            }
            "stake" => {
                assert_eq!(
                    &env::predecessor_account_id(),
                    &self.token_contract_id,
                    "Only supports the OCT token contract"
                );
                assert_eq!(msg_vec.len(), 3, "params length wrong!");
                self.stake(
                    msg_vec.get(1).unwrap().to_string(),
                    msg_vec.get(2).unwrap().to_string(),
                    amount.0,
                );
                PromiseOrValue::Value(0.into())
            }
            "stake_more" => {
                assert_eq!(
                    &env::predecessor_account_id(),
                    &self.token_contract_id,
                    "Only supports the OCT token contract"
                );
                assert_eq!(msg_vec.len(), 2, "params length wrong!");
                self.stake_more(msg_vec.get(1).unwrap().to_string(), amount.0);
                PromiseOrValue::Value(0.into())
            }
            "lock_token" => {
                let token_id = env::predecessor_account_id();
                assert_eq!(msg_vec.len(), 3, "params length wrong!");
                self.lock_token(
                    msg_vec.get(1).unwrap().to_string(),
                    msg_vec.get(2).unwrap().to_string(),
                    sender_id.into(),
                    token_id,
                    amount.0,
                );
                PromiseOrValue::Value(0.into())
            }
            _ => {
                log!("Function name not matched, msg = {}", msg);
                PromiseOrValue::Value(amount)
            }
        }
    }

    fn validate_hex_address(&self, address: String) -> String {
        let address_str = &address.as_str();
        let suffix_str = &address_str[..2];
        let hex_str;
        if suffix_str == "0x" {
            hex_str = &address_str[2..address_str.len()];
        } else {
            hex_str = address_str;
        }
        let data = hex::decode(hex_str).expect("address should be a valid hex string.");
        assert_eq!(data.len(), 32, "address should be 32 bytes long");
        let mut hex_address: String = "0x".to_owned();
        hex_address.push_str(hex_str);
        hex_address
    }

    fn register_appchain(
        &mut self,
        appchain_id: String,
        website_url: String,
        github_address: String,
        github_release: String,
        commit_id: String,
        email: String,
        bond_tokens: u128,
    ) {
        let founder_id = env::signer_account_id();
        assert!(
            self.appchain_metadatas.get(&appchain_id).is_none(),
            "Appchain_id is already registered"
        );
        self.appchain_id_list.push(&appchain_id);
        self.appchain_data_fact_sets_len.insert(&appchain_id, &0);

        self.appchain_metadatas.insert(
            &appchain_id,
            &LazyOption::new(
                StorageKey::AppchainMetadata(appchain_id.clone()).into_bytes(),
                Some(&AppchainMetadata::new(
                    appchain_id.clone(),
                    founder_id,
                    website_url,
                    github_address,
                    github_release,
                    commit_id,
                    email,
                    bond_tokens,
                )),
            ),
        );
        self.appchain_states.insert(
            &appchain_id,
            &LazyOption::new(
                StorageKey::AppchainState(appchain_id.clone()).into_bytes(),
                Some(&AppchainState::new(&appchain_id)),
            ),
        );

        log!(
            "Appchain added, appchain_id is {}, bund_tokens is {}.",
            appchain_id,
            u128::from(bond_tokens)
        );
    }

    fn get_appchain_metadata(&self, appchain_id: &AppchainId) -> AppchainMetadata {
        self.appchain_metadatas
            .get(appchain_id)
            .expect(APPCHAIN_METADATA_NOT_FOUND)
            .get()
            .expect(APPCHAIN_METADATA_NOT_FOUND)
    }

    fn set_appchain_metadata(
        &mut self,
        appchain_id: &AppchainId,
        appchain_metadata: &AppchainMetadata,
    ) {
        self.appchain_metadatas
            .get(appchain_id)
            .expect(APPCHAIN_METADATA_NOT_FOUND)
            .set(appchain_metadata);
    }

    fn get_appchain_state(&self, appchain_id: &AppchainId) -> AppchainState {
        self.appchain_states
            .get(appchain_id)
            .expect(APPCHAIN_STATE_NOT_FOUND)
            .get()
            .expect(APPCHAIN_STATE_NOT_FOUND)
    }

    fn set_appchain_state(&mut self, appchain_id: &AppchainId, appchain_state: &AppchainState) {
        self.appchain_states
            .get(appchain_id)
            .expect(APPCHAIN_STATE_NOT_FOUND)
            .set(appchain_state);
    }

    pub fn update_appchain(
        &mut self,
        appchain_id: AppchainId,
        website_url: String,
        github_address: String,
        github_release: String,
        commit_id: String,
        email: String,
    ) {
        let required_status_vec = vec![AppchainStatus::Booting];
        let appchain_status = self.get_appchain_state(&appchain_id).status;
        let mut appchain_metadata = self.get_appchain_metadata(&appchain_id);
        assert!(
            required_status_vec.iter().any(|s| *s == appchain_status),
            "Appchain can't be updated at current status."
        );

        let account_id = env::signer_account_id();
        // Only appchain founder can do this
        assert!(
            account_id.eq(&appchain_metadata.founder_id),
            "You aren't the appchain founder!"
        );

        appchain_metadata.update_basic_info(
            website_url,
            github_address,
            github_release,
            commit_id,
            email,
        );
        self.set_appchain_metadata(&appchain_id, &appchain_metadata);
    }

    pub fn get_appchains(&self, from_index: u32, limit: u32) -> Vec<Appchain> {
        (from_index..std::cmp::min(from_index + limit, self.appchain_id_list.len() as u32))
            .map(|index| {
                let appchain_id = self.appchain_id_list.get(index as u64).unwrap();
                self.get_appchain(appchain_id).unwrap()
            })
            .collect()
    }

    pub fn get_num_appchains(&self) -> u32 {
        self.appchain_metadatas.len() as u32
    }

    /// Returns the total staking balance.
    pub fn get_total_staked_balance(&self) -> U128 {
        self.total_staked_balance.into()
    }

    pub fn get_minimum_staking_amount(&self) -> U128 {
        self.minimum_staking_amount.into()
    }

    pub fn get_appchain(&self, appchain_id: AppchainId) -> Option<Appchain> {
        let appchain_metadata = self.get_appchain_metadata(&appchain_id);
        let appchain_state = self.get_appchain_state(&appchain_id);
        Some(Appchain {
            id: appchain_id.clone(),
            founder_id: appchain_metadata.founder_id.clone(),
            website_url: appchain_metadata.website_url.clone(),
            github_address: appchain_metadata.github_address.clone(),
            github_release: appchain_metadata.github_release.clone(),
            commit_id: appchain_metadata.commit_id.clone(),
            email: appchain_metadata.email.clone(),
            chain_spec_url: appchain_metadata.chain_spec_url.clone(),
            chain_spec_hash: appchain_metadata.chain_spec_hash.clone(),
            chain_spec_raw_url: appchain_metadata.chain_spec_raw_url.clone(),
            chain_spec_raw_hash: appchain_metadata.chain_spec_raw_hash.clone(),
            boot_nodes: appchain_metadata.boot_nodes.clone(),
            rpc_endpoint: appchain_metadata.rpc_endpoint.clone(),
            bond_tokens: appchain_metadata.bond_tokens.into(),
            validators: self.get_validators(appchain_id.clone()).unwrap_or_default(),
            validators_timestamp: appchain_state.validators_timestamp,
            status: appchain_state.status,
            block_height: appchain_metadata.block_height,
            staked_balance: appchain_state.staked_balance.into(),
            subql_url: appchain_metadata.subql_url.clone(),
            fact_sets_len: self
                .appchain_data_fact_sets_len
                .get(&appchain_id)
                .unwrap_or(0)
                .into(),
            validator_sets_len: appchain_state.currently_valid_validators_nonce,
        })
    }

    pub fn get_version(&self) -> u32 {
        self.version
    }

    pub fn get_appchain_minimum_validators(&self) -> u32 {
        self.appchain_minimum_validators
    }

    pub fn get_validators(&self, appchain_id: AppchainId) -> Option<Vec<Validator>> {
        let appchain_state = self.get_appchain_state(&appchain_id);
        Option::from(
            appchain_state
                .get_validators()
                .iter()
                .map(|v| v.to_validator())
                .collect::<Vec<_>>(),
        )
    }

    pub fn next_validator_set(
        &self,
        appchain_id: AppchainId,
        boot_time: bool,
    ) -> Option<ValidatorSet> {
        if let Some(state_option) = self.appchain_states.get(&appchain_id) {
            if let Some(appchain_state) = state_option.get() {
                return appchain_state.get_next_validator_set();
            }
        }
        Option::None
    }

    pub fn get_validator(
        &self,
        appchain_id: AppchainId,
        validator_id: ValidatorId,
    ) -> Option<Validator> {
        if let Some(state_option) = self.appchain_states.get(&appchain_id) {
            if let Some(appchain_state) = state_option.get() {
                if let Some(appchain_validator) = appchain_state.get_validator(&validator_id) {
                    return Option::from(appchain_validator.to_validator());
                }
            }
        }
        Option::None
    }

    pub fn get_delegator(
        &self,
        appchain_id: AppchainId,
        validator_id: ValidatorId,
        delegator_id: DelegatorId,
    ) -> Option<Delegator> {
        if let Some(state_option) = self.appchain_states.get(&appchain_id) {
            if let Some(appchain_state) = state_option.get() {
                if let Some(appchain_validator) = appchain_state.get_validator(&validator_id) {
                    if let Some(appchain_delegator) =
                        appchain_validator.get_delegator(&delegator_id)
                    {
                        return Option::from(appchain_delegator.to_delegator());
                    }
                }
            }
        }
        Option::None
    }

    // Returns the appchain current validator_set len
    pub fn get_curr_validator_set_len(&self, appchain_id: AppchainId) -> u32 {
        self.get_appchain_state(&appchain_id)
            .currently_valid_validators_nonce
    }

    pub fn get_validator_set(&self, appchain_id: AppchainId) -> Option<ValidatorSet> {
        if let Some(state_option) = self.appchain_states.get(&appchain_id) {
            if let Some(appchain_state) = state_option.get() {
                return appchain_state.get_current_validator_set();
            }
        }
        Option::None
    }

    pub fn get_validator_set_by_set_id(
        &self,
        appchain_id: AppchainId,
        set_id: u32,
    ) -> Option<ValidatorSet> {
        self.get_appchain_state(&appchain_id)
            .get_validators_history_by_nonce(set_id)
    }

    fn in_staking_period(&mut self, appchain_id: AppchainId) -> bool {
        let required_status_vec = vec![AppchainStatus::Staging, AppchainStatus::Booting];
        required_status_vec
            .iter()
            .any(|s| *s == self.get_appchain_state(&appchain_id).status)
    }

    fn stake(&mut self, appchain_id: AppchainId, id: String, amount: u128) {
        // Check to update validator set before all
        let validator_id = self.validate_hex_address(id);

        assert!(
            self.in_staking_period(appchain_id.clone()),
            "It's not in staking period."
        );
        let account_id = env::signer_account_id();
        // Check amount
        assert!(
            amount >= self.minimum_staking_amount,
            "Insufficient staking amount"
        );

        let validators = self.get_validators(appchain_id.clone()).unwrap();
        for v in validators {
            assert!(
                v.account_id != account_id,
                "Your account is already staked on the appchain!"
            );
            assert!(
                v.id != validator_id,
                "This validator is already staked on the appchain!"
            );
        }

        let mut appchain_state = self.get_appchain_state(&appchain_id);
        appchain_state.stake(&validator_id, amount);
        self.total_staked_balance += amount;
        self.set_appchain_state(&appchain_id, &appchain_state);
    }

    fn stake_more(&mut self, appchain_id: AppchainId, amount: u128) {
        assert!(
            self.in_staking_period(appchain_id.clone()),
            "Appchain can't be staked in current status."
        );
        let account_id = env::signer_account_id();

        let mut appchain_state = self.get_appchain_state(&appchain_id);
        appchain_state
            .get_validator(&account_id)
            .expect("You are not staking on the appchain");
        appchain_state.stake(&account_id, amount);
        self.total_staked_balance += amount;
        self.set_appchain_state(&appchain_id, &appchain_state);
    }

    pub fn remove_validator(&mut self, appchain_id: AppchainId, validator_id: String) {
        self.assert_owner();
        assert!(
            self.in_staking_period(appchain_id.clone()),
            "Appchain can't be staked in current status."
        );

        let validator = self
            .get_validator(appchain_id.clone(), validator_id.clone())
            .expect("This validator not exists");

        let account_id = validator.account_id;

        ext_token::ft_transfer(
            account_id.clone(),
            validator.staked_amount.into(),
            None,
            &self.token_contract_id,
            1,
            GAS_FOR_FT_TRANSFER_CALL,
        )
        .then(ext_self::resolve_remove_validator(
            appchain_id,
            validator_id,
            validator.staked_amount.into(),
            &env::current_account_id(),
            NO_DEPOSIT,
            env::prepaid_gas() / 2,
        ));
    }

    pub fn resolve_remove_validator(
        &mut self,
        appchain_id: AppchainId,
        validator_id: ValidatorId,
        amount: U128,
    ) {
        assert_self();
        // Update state
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let mut appchain_state = self.get_appchain_state(&appchain_id);
                self.total_staked_balance -= appchain_state.remove_validator(&validator_id);
                self.set_appchain_state(&appchain_id, &appchain_state);
            }
            PromiseResult::Failed => {}
        }
    }

    pub fn unstake(&mut self, appchain_id: AppchainId) {
        assert!(
            self.in_staking_period(appchain_id.clone()),
            "Appchain can't be staked in current status."
        );
        let account_id = env::signer_account_id();
        let validators = self.get_validators(appchain_id.clone()).unwrap();

        let validator = validators
            .iter()
            .find(|v| v.account_id == account_id)
            .expect("You are not staked on the appchain");

        ext_token::ft_transfer(
            account_id.clone(),
            validator.staked_amount.into(),
            None,
            &self.token_contract_id,
            1,
            GAS_FOR_FT_TRANSFER_CALL,
        )
        .then(ext_self::resolve_remove_validator(
            appchain_id,
            validator.id.clone(),
            validator.staked_amount.into(),
            &env::current_account_id(),
            NO_DEPOSIT,
            env::prepaid_gas() / 2,
        ));
    }

    pub fn update_subql_url(&mut self, appchain_id: AppchainId, subql_url: String) {
        self.assert_owner();
        let mut appchain_metadata = self.get_appchain_metadata(&appchain_id);
        appchain_metadata.update_subql(subql_url);
        self.set_appchain_metadata(&appchain_id, &appchain_metadata);
    }
}

pub trait Ownable {
    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.get_owner(),
            "You are not the contract owner."
        );
    }
    fn get_owner(&self) -> AccountId;
    fn set_owner(&mut self, owner: AccountId);
}

#[near_bindgen]
impl Ownable for OctopusRelay {
    fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    fn set_owner(&mut self, owner: AccountId) {
        self.assert_owner();
        self.owner = owner;
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 *
 * To run from contract directory:
 * cargo test -- --nocapture
 *
 * From project root, to run in combination with frontend tests:
 * yarn test
 *
 */
#[cfg(test)]
mod tests {}
