pub mod bridge;
pub mod types;

// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use crate::types::{
    Appchain, AppchainStatus, BridgeStatus, BridgeToken, Delegation, LiteValidator, Locked,
    LockerStatus, Validator, ValidatorSet,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_self, env, ext_contract, log, near_bindgen, wee_alloc, AccountId, Balance, BlockHeight,
    PromiseOrValue, PromiseResult,
};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const NO_DEPOSIT: Balance = 0;
const GAS_FOR_FT_TRANSFER_CALL: u64 = 30_000_000_000_000;
const SINGLE_CALL_GAS: u64 = 10_000_000_000_000;
const OCT_DECIMALS_BASE: Balance = 1000_000_000_000_000_000_000_000;

const VALIDATOR_SET_CYCLE: u64 = 60000000000;
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
    pub appchain_minium_validators: u32,
    pub minium_staking_amount: Balance,
    pub total_staked_balance: Balance,
    pub appchain_id_list: Vector<AppchainId>,

    // data for Appchain
    pub appchain_data_founder_id: LookupMap<AppchainId, AccountId>,
    pub appchain_data_website_url: LookupMap<AppchainId, String>,
    pub appchain_data_github_address: LookupMap<AppchainId, String>,
    pub appchain_data_github_release: LookupMap<AppchainId, String>,
    pub appchain_data_commit_id: LookupMap<AppchainId, String>,
    pub appchain_data_chain_spec_url: LookupMap<AppchainId, String>,
    pub appchain_data_chain_spec_hash: LookupMap<AppchainId, String>,
    pub appchain_data_chain_spec_raw_url: LookupMap<AppchainId, String>,
    pub appchain_data_chain_spec_raw_hash: LookupMap<AppchainId, String>,
    pub appchain_data_boot_nodes: LookupMap<AppchainId, String>,
    pub appchain_data_rpc_endpoint: LookupMap<AppchainId, String>,
    pub appchain_data_bond_tokens: LookupMap<AppchainId, Balance>,
    pub appchain_data_validator_ids: LookupMap<AppchainId, Vec<ValidatorId>>,
    pub appchain_data_validators_timestamp: LookupMap<AppchainId, u64>,
    pub appchain_data_status: LookupMap<AppchainId, AppchainStatus>,
    pub appchain_data_block_height: LookupMap<AppchainId, BlockHeight>,
    pub appchain_data_staked_balance: LookupMap<AppchainId, Balance>,

    pub appchain_data_validator_sets_len: LookupMap<AppchainId, SeqNum>,
    pub appchain_data_validator_set: LookupMap<(AppchainId, SeqNum), ValidatorSet>,

    // data for Validator
    pub validator_data_account_id: LookupMap<(AppchainId, ValidatorId), AccountId>,
    pub validator_data_weight: LookupMap<(AppchainId, ValidatorId), u32>,
    pub validator_data_staked_amount: LookupMap<(AppchainId, ValidatorId), Balance>,
    pub validator_data_block_height: LookupMap<(AppchainId, ValidatorId), BlockHeight>,
    pub validator_data_delegation_ids: LookupMap<(AppchainId, ValidatorId), Vec<AccountId>>,

    // data for Delegation
    pub delegation_data_amount: LookupMap<(AppchainId, ValidatorId, DelegatorId), Balance>,
    pub delegation_data_account_id: LookupMap<(AppchainId, ValidatorId, DelegatorId), AccountId>,
    pub delegation_data_block_height:
        LookupMap<(AppchainId, ValidatorId, DelegatorId), BlockHeight>,

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

    status: LockerStatus,
    locked_appchain_map: LookupMap<(AppchainId, SeqNum), Locked>,
    locked_len_appchain_map: LookupMap<AppchainId, SeqNum>,
}

#[ext_contract(ext_self)]
pub trait ExtOctopusRelay {
    fn resolve_unstake(&mut self, appchain_id: AppchainId, account_id: AccountId, amount: U128);
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
    fn resolve_remove_appchain(&mut self, index: u32, appchain_id: AppchainId);
}

#[ext_contract(ext_token)]
pub trait ExtContract {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
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
        appchain_minium_validators: u32,
        minium_staking_amount: U128,
        bridge_limit_ratio: u16,
        oct_token_price: U128,
    ) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized");
        assert_self();
        Self {
            version: 0,
            token_contract_id,
            total_staked_balance: 0,
            appchain_minium_validators,
            minium_staking_amount: minium_staking_amount.0,
            appchain_id_list: Vector::new(b"ail".to_vec()),
            appchain_data_founder_id: LookupMap::new(b"afi".to_vec()),
            appchain_data_website_url: LookupMap::new(b"wu".to_vec()),
            appchain_data_github_address: LookupMap::new(b"ga".to_vec()),
            appchain_data_github_release: LookupMap::new(b"gr".to_vec()),
            appchain_data_commit_id: LookupMap::new(b"gcl".to_vec()),
            appchain_data_chain_spec_url: LookupMap::new(b"csu".to_vec()),
            appchain_data_chain_spec_hash: LookupMap::new(b"csh".to_vec()),
            appchain_data_chain_spec_raw_url: LookupMap::new(b"csru".to_vec()),
            appchain_data_chain_spec_raw_hash: LookupMap::new(b"csrh".to_vec()),
            appchain_data_boot_nodes: LookupMap::new(b"bn".to_vec()),
            appchain_data_rpc_endpoint: LookupMap::new(b"re".to_vec()),
            appchain_data_bond_tokens: LookupMap::new(b"bt".to_vec()),
            appchain_data_validator_ids: LookupMap::new(b"vi".to_vec()),
            appchain_data_validators_timestamp: LookupMap::new(b"vt".to_vec()),
            appchain_data_status: LookupMap::new(b"st".to_vec()),
            appchain_data_block_height: LookupMap::new(b"abh".to_vec()),
            appchain_data_staked_balance: LookupMap::new(b"sb".to_vec()),

            appchain_data_validator_sets_len: LookupMap::new(b"vsl".to_vec()),
            appchain_data_validator_set: LookupMap::new(b"vs".to_vec()),

            validator_data_account_id: LookupMap::new(b"ai".to_vec()),
            validator_data_weight: LookupMap::new(b"we".to_vec()),
            validator_data_staked_amount: LookupMap::new(b"sa".to_vec()),
            validator_data_block_height: LookupMap::new(b"vbh".to_vec()),
            validator_data_delegation_ids: LookupMap::new(b"di".to_vec()),

            delegation_data_amount: LookupMap::new(b"dam".to_vec()),
            delegation_data_account_id: LookupMap::new(b"dai".to_vec()),
            delegation_data_block_height: LookupMap::new(b"dbh".to_vec()),

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

            status: LockerStatus::default(),
            locked_appchain_map: LookupMap::new(b"la".to_vec()),
            locked_len_appchain_map: LookupMap::new(b"ll".to_vec()),
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
                assert_eq!(msg_vec.len(), 6, "params length wrong!");
                self.register_appchain(
                    msg_vec.get(1).unwrap().to_string(),
                    msg_vec.get(2).unwrap().to_string(),
                    msg_vec.get(3).unwrap().to_string(),
                    msg_vec.get(4).unwrap().to_string(),
                    msg_vec.get(5).unwrap().to_string(),
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

    fn register_appchain(
        &mut self,
        appchain_id: String,
        website_url: String,
        github_address: String,
        github_release: String,
        commit_id: String,
        bond_tokens: u128,
    ) {
        let founder_id = env::signer_account_id();
        assert!(
            !self.appchain_data_founder_id.contains_key(&appchain_id),
            "Appchain_id is already registered"
        );
        self.appchain_data_founder_id
            .insert(&appchain_id, &founder_id);
        self.appchain_data_website_url
            .insert(&appchain_id, &website_url);
        self.appchain_data_github_address
            .insert(&appchain_id, &github_address);
        self.appchain_data_github_release
            .insert(&appchain_id, &github_release);
        self.appchain_data_commit_id
            .insert(&appchain_id, &commit_id);
        self.appchain_data_bond_tokens
            .insert(&appchain_id, &bond_tokens);
        self.appchain_data_status
            .insert(&appchain_id, &AppchainStatus::Auditing);
        self.appchain_data_validator_ids
            .insert(&appchain_id, &Vec::default());
        self.appchain_data_block_height
            .insert(&appchain_id, &env::block_index());
        self.appchain_data_validator_sets_len
            .insert(&appchain_id, &0);
        self.appchain_id_list.push(&appchain_id);

        log!(
            "Appchain added, appchain_id is {}, bund_tokens is {}.",
            appchain_id,
            u128::from(bond_tokens)
        );
    }

    pub fn remove_appchain(&mut self, appchain_id: AppchainId) {
        self.assert_owner();
        assert_eq!(
            self.appchain_data_status
                .get(&appchain_id)
                .expect("Appchain not found."),
            AppchainStatus::Auditing,
            "appchain can only be removed in auditing status"
        );
        let index = self
            .appchain_id_list
            .to_vec()
            .iter()
            .position(|id| id.clone() == appchain_id)
            .expect("Appchain not exists") as u32;

        let bond_tokens = self
            .appchain_data_bond_tokens
            .get(&appchain_id)
            .expect("Appchain not exists");
        let account_id = self
            .appchain_data_founder_id
            .get(&appchain_id)
            .unwrap()
            .clone();

        ext_token::ft_transfer(
            account_id,
            (bond_tokens / 10).into(),
            None,
            &self.token_contract_id,
            1,
            GAS_FOR_FT_TRANSFER_CALL,
        )
        .then(ext_self::resolve_remove_appchain(
            index,
            appchain_id.clone(),
            &env::current_account_id(),
            NO_DEPOSIT,
            env::prepaid_gas() / 2,
        ));
    }

    pub fn resolve_remove_appchain(&mut self, index: u32, appchain_id: AppchainId) {
        // Update state
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                self.appchain_id_list.swap_remove(index as u64);
                self.appchain_data_founder_id.remove(&appchain_id);
                self.appchain_data_website_url.remove(&appchain_id);
                self.appchain_data_github_address.remove(&appchain_id);
                self.appchain_data_github_release.remove(&appchain_id);
                self.appchain_data_commit_id.remove(&appchain_id);
                self.appchain_data_chain_spec_url.remove(&appchain_id);
                self.appchain_data_chain_spec_hash.remove(&appchain_id);
                self.appchain_data_chain_spec_raw_url.remove(&appchain_id);
                self.appchain_data_chain_spec_raw_hash.remove(&appchain_id);
                self.appchain_data_boot_nodes.remove(&appchain_id);
                self.appchain_data_rpc_endpoint.remove(&appchain_id);
                self.appchain_data_bond_tokens.remove(&appchain_id);
                self.appchain_data_validator_ids.remove(&appchain_id);
                self.appchain_data_validators_timestamp.remove(&appchain_id);
                self.appchain_data_status.remove(&appchain_id);
                self.appchain_data_block_height.remove(&appchain_id);
                self.appchain_data_staked_balance.remove(&appchain_id);
                self.appchain_data_validator_sets_len.remove(&appchain_id);
                self.appchain_data_validator_set.remove(&(appchain_id, 0));
            }
            PromiseResult::Failed => {}
        }
    }

    pub fn list_appchain(&mut self, appchain_id: AppchainId) {
        self.assert_owner();
        let candidate_appchain = self
            .get_appchain(appchain_id.clone())
            .expect("Appchain not found");
        assert_eq!(
            &candidate_appchain.status,
            &AppchainStatus::Auditing,
            "Appchain is not in auditing."
        );
        self.appchain_data_status
            .insert(&appchain_id, &AppchainStatus::Frozen);
    }

    pub fn update_appchain(
        &mut self,
        appchain_id: AppchainId,
        website_url: String,
        github_address: String,
        github_release: String,
        commit_id: String,
        chain_spec_url: String,
        chain_spec_hash: String,
        chain_spec_raw_url: String,
        chain_spec_raw_hash: String,
    ) {
        assert_ne!(
            self.appchain_data_status
                .get(&appchain_id)
                .expect("Appchain not found"),
            AppchainStatus::Auditing,
            "Appchain is in auditing."
        );

        let account_id = env::signer_account_id();
        // Only appchain founder can do this
        assert!(
            account_id == self.appchain_data_founder_id.get(&appchain_id).unwrap(),
            "You aren't the appchain founder!"
        );
        self.appchain_data_website_url
            .insert(&appchain_id, &website_url);
        self.appchain_data_github_address
            .insert(&appchain_id, &github_address);
        self.appchain_data_github_release
            .insert(&appchain_id, &github_release);
        self.appchain_data_commit_id
            .insert(&appchain_id, &commit_id);
        self.appchain_data_chain_spec_url
            .insert(&appchain_id, &chain_spec_url);
        self.appchain_data_chain_spec_hash
            .insert(&appchain_id, &chain_spec_hash);
        self.appchain_data_chain_spec_raw_url
            .insert(&appchain_id, &chain_spec_raw_url);
        self.appchain_data_chain_spec_raw_hash
            .insert(&appchain_id, &chain_spec_raw_hash);
        self.appchain_data_status
            .insert(&appchain_id, &AppchainStatus::Frozen);
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
        self.appchain_id_list.len() as u32
    }

    /// Returns the total staking balance.
    pub fn get_total_staked_balance(&self) -> U128 {
        self.total_staked_balance.into()
    }

    pub fn get_minium_staking_amount(&self) -> U128 {
        self.minium_staking_amount.into()
    }

    pub fn get_appchain(&self, appchain_id: AppchainId) -> Option<Appchain> {
        let appchain_founder_option = self.appchain_data_founder_id.get(&appchain_id);
        if appchain_founder_option.is_some() {
            Some(Appchain {
                id: appchain_id.clone(),
                founder_id: self
                    .appchain_data_founder_id
                    .get(&appchain_id)
                    .unwrap_or(String::from(""))
                    .clone(),
                website_url: self
                    .appchain_data_website_url
                    .get(&appchain_id)
                    .unwrap_or(String::from(""))
                    .clone(),
                github_address: self
                    .appchain_data_github_address
                    .get(&appchain_id)
                    .unwrap_or(String::from(""))
                    .clone(),
                github_release: self
                    .appchain_data_github_release
                    .get(&appchain_id)
                    .unwrap_or(String::from(""))
                    .clone(),
                commit_id: self
                    .appchain_data_commit_id
                    .get(&appchain_id)
                    .unwrap_or(String::from(""))
                    .clone(),
                chain_spec_url: self
                    .appchain_data_chain_spec_url
                    .get(&appchain_id)
                    .unwrap_or(String::from(""))
                    .clone(),
                chain_spec_hash: self
                    .appchain_data_chain_spec_hash
                    .get(&appchain_id)
                    .unwrap_or(String::from(""))
                    .clone(),
                chain_spec_raw_url: self
                    .appchain_data_chain_spec_raw_url
                    .get(&appchain_id)
                    .unwrap_or(String::from(""))
                    .clone(),
                chain_spec_raw_hash: self
                    .appchain_data_chain_spec_raw_hash
                    .get(&appchain_id)
                    .unwrap_or(String::from(""))
                    .clone(),
                boot_nodes: self
                    .appchain_data_boot_nodes
                    .get(&appchain_id)
                    .unwrap_or(String::from(""))
                    .clone(),
                rpc_endpoint: self
                    .appchain_data_rpc_endpoint
                    .get(&appchain_id)
                    .unwrap_or(String::from(""))
                    .clone(),
                bond_tokens: self
                    .appchain_data_bond_tokens
                    .get(&appchain_id)
                    .unwrap()
                    .clone()
                    .into(),
                validators: self.get_validators(appchain_id.clone()).unwrap_or_default(),
                validators_timestamp: self
                    .appchain_data_validators_timestamp
                    .get(&appchain_id)
                    .unwrap_or(0)
                    .clone(),
                status: self.appchain_data_status.get(&appchain_id).unwrap().clone(),
                block_height: self
                    .appchain_data_block_height
                    .get(&appchain_id)
                    .unwrap()
                    .clone(),
                staked_balance: self
                    .appchain_data_staked_balance
                    .get(&appchain_id)
                    .unwrap_or(0)
                    .into(),
            })
        } else {
            None
        }
    }

    pub fn get_version(&self) -> u32 {
        self.version
    }

    pub fn get_appchain_minium_validators(&self) -> u32 {
        self.appchain_minium_validators
    }

    pub fn get_validators(&self, appchain_id: AppchainId) -> Option<Vec<Validator>> {
        self.appchain_data_validator_ids
            .get(&appchain_id)
            .expect("Appchain not found")
            .iter()
            .map(|v| self.get_validator(appchain_id.clone(), v.to_string()))
            .collect()
    }

    pub fn next_validator_set(&self, appchain_id: AppchainId) -> Option<ValidatorSet> {
        let seq_num = self.get_curr_validator_set_len(appchain_id.clone());
        let validators_timestamp_option = self.appchain_data_validators_timestamp.get(&appchain_id);
        if !validators_timestamp_option.is_some() {
            return None;
        }
        let validators_timestamp = validators_timestamp_option.unwrap();
        let validators_from_unix = validators_timestamp / VALIDATOR_SET_CYCLE;
        let today_from_unix = env::block_timestamp() / VALIDATOR_SET_CYCLE;
        if today_from_unix - validators_from_unix > 0 {
            let mut validators: Vec<LiteValidator> = self
                .get_validators(appchain_id)
                .unwrap()
                .iter()
                .map(|v| LiteValidator {
                    id: v.id.clone(),
                    account_id: v.account_id.clone(),
                    weight: v.weight,
                    block_height: v.block_height,
                    delegations: v.delegations.clone(),
                })
                .collect();
            validators.sort_by(|a, b| b.weight.cmp(&a.weight));
            return Some(ValidatorSet {
                seq_num,
                validators,
            });
        } else {
            return None;
        }
    }

    pub fn get_validator(
        &self,
        appchain_id: AppchainId,
        validator_id: String,
    ) -> Option<Validator> {
        let account_id_option = self
            .validator_data_account_id
            .get(&(appchain_id.clone(), validator_id.clone()));
        if account_id_option.is_some() {
            Some(Validator {
                id: validator_id.clone(),
                account_id: account_id_option.unwrap().to_string(),
                weight: self
                    .validator_data_weight
                    .get(&(appchain_id.clone(), validator_id.clone()))
                    .unwrap(),
                staked_amount: self
                    .validator_data_staked_amount
                    .get(&(appchain_id.clone(), validator_id.clone()))
                    .unwrap()
                    .into(),
                block_height: self
                    .validator_data_block_height
                    .get(&(appchain_id.clone(), validator_id.clone()))
                    .unwrap(),
                delegations: self
                    .validator_data_delegation_ids
                    .get(&(appchain_id.clone(), validator_id.clone()))
                    .unwrap()
                    .iter()
                    .map(|d| {
                        self.get_delegation(
                            appchain_id.clone(),
                            validator_id.clone(),
                            d.to_string(),
                        )
                        .unwrap()
                    })
                    .collect(),
            })
        } else {
            None
        }
    }

    pub fn get_delegation(
        &self,
        appchain_id: AppchainId,
        validator_id: ValidatorId,
        delegator_id: DelegatorId,
    ) -> Option<Delegation> {
        let account_id_option = self.delegation_data_account_id.get(&(
            appchain_id.clone(),
            validator_id.clone(),
            delegator_id.clone(),
        ));
        if account_id_option.is_some() {
            Some(Delegation {
                id: delegator_id.clone(),
                account_id: account_id_option.unwrap().to_string(),
                amount: self
                    .delegation_data_amount
                    .get(&(
                        appchain_id.clone(),
                        validator_id.clone(),
                        delegator_id.clone(),
                    ))
                    .unwrap()
                    .into(),
                block_height: self
                    .delegation_data_block_height
                    .get(&(appchain_id, validator_id, delegator_id))
                    .unwrap(),
            })
        } else {
            None
        }
    }

    // Returns the appchain current validator_set index
    pub fn get_curr_validator_set_index(&self, appchain_id: AppchainId) -> u32 {
        self.get_curr_validator_set_len(appchain_id) - 1
    }

    // Returns the appchain current validator_set len
    pub fn get_curr_validator_set_len(&self, appchain_id: AppchainId) -> u32 {
        self.appchain_data_validator_sets_len
            .get(&appchain_id)
            .unwrap()
    }

    pub fn get_validator_set(&self, appchain_id: AppchainId) -> Option<ValidatorSet> {
        let next_validator_set = self.next_validator_set(appchain_id.clone());
        if next_validator_set.is_some() {
            next_validator_set
        } else {
            let validator_set_len = self.get_curr_validator_set_len(appchain_id.clone());
            if validator_set_len == 0 {
                return None;
            }
            self.get_validator_set_by_seq_num(appchain_id.clone(), validator_set_len - 1)
        }
    }

    pub fn get_validator_set_by_seq_num(
        &self,
        appchain_id: AppchainId,
        seq_num: u32,
    ) -> Option<ValidatorSet> {
        if seq_num == self.get_curr_validator_set_len(appchain_id.clone()) {
            return self.next_validator_set(appchain_id);
        } else {
            return self
                .appchain_data_validator_set
                .get(&(appchain_id, seq_num));
        }
    }

    fn stake(&mut self, appchain_id: AppchainId, id: String, amount: u128) {
        assert_ne!(
            self.appchain_data_status
                .get(&appchain_id)
                .expect("Appchain not found"),
            AppchainStatus::Auditing,
            "Appchain is in auditing."
        );
        let account_id = env::signer_account_id();
        // Check amount
        assert!(
            amount >= self.minium_staking_amount,
            "Insufficient staking amount"
        );

        let weight = (amount / OCT_DECIMALS_BASE) as u32;

        let validators = self.get_validators(appchain_id.clone()).unwrap();
        for v in validators {
            assert!(
                v.account_id != account_id,
                "Your account is already staked on the appchain!"
            );
            assert!(
                v.id != id,
                "This validator is already staked on the appchain!"
            );
        }

        self.validator_data_account_id
            .insert(&(appchain_id.clone(), id.clone()), &account_id);
        self.validator_data_weight
            .insert(&(appchain_id.clone(), id.clone()), &weight);
        self.validator_data_staked_amount
            .insert(&(appchain_id.clone(), id.clone()), &amount);
        self.validator_data_block_height
            .insert(&(appchain_id.clone(), id.clone()), &env::block_index());

        self.validator_data_delegation_ids
            .insert(&(appchain_id.clone(), id.clone()), &Vec::default());

        let mut validator_ids: Vec<ValidatorId> = self
            .appchain_data_validator_ids
            .get(&appchain_id)
            .unwrap()
            .clone();

        validator_ids.push(id.clone());
        self.appchain_data_validator_ids
            .insert(&appchain_id, &validator_ids);

        let staked_balance = self
            .appchain_data_staked_balance
            .get(&appchain_id)
            .unwrap_or(0);
        self.appchain_data_staked_balance
            .insert(&appchain_id, &(staked_balance + amount));
        self.total_staked_balance += amount;

        // Check to update validator set
        self.update_validator_set(appchain_id);
    }

    fn stake_more(&mut self, appchain_id: AppchainId, amount: u128) {
        assert_ne!(
            self.appchain_data_status
                .get(&appchain_id)
                .expect("Appchain not found"),
            AppchainStatus::Auditing,
            "Appchain is in auditing."
        );

        let account_id = env::signer_account_id();
        // Check amount
        assert!(
            amount >= self.minium_staking_amount,
            "Insufficient staking amount"
        );

        let weight = (amount / OCT_DECIMALS_BASE) as u32;

        let mut validators = self
            .get_validators(appchain_id.clone())
            .expect("Appchain not found");

        validators
            .iter()
            .find(|v| v.account_id == account_id)
            .expect("You are not staked on the appchain");

        let mut found = false;
        for v in validators.iter_mut() {
            if v.account_id == account_id {
                self.validator_data_staked_amount.insert(
                    &(appchain_id.clone(), v.id.clone()),
                    &(v.staked_amount.0 + amount),
                );
                self.validator_data_weight
                    .insert(&(appchain_id.clone(), v.id.clone()), &(v.weight + weight));
                found = true;
            }
        }

        if !found {
            panic!("You are not staked on the appchain");
        }

        let staked_balance = self
            .appchain_data_staked_balance
            .get(&appchain_id)
            .unwrap_or(0);
        self.appchain_data_staked_balance
            .insert(&appchain_id, &(staked_balance + amount));
        self.total_staked_balance += amount;

        // Check to update validator set
        self.update_validator_set(appchain_id);
    }

    #[payable]
    pub fn unstake(&mut self, appchain_id: AppchainId) {
        assert_ne!(
            self.appchain_data_status
                .get(&appchain_id)
                .expect("Appchain not found"),
            AppchainStatus::Auditing,
            "Appchain is in auditing."
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
        .then(ext_self::resolve_unstake(
            appchain_id,
            account_id,
            validator.staked_amount.into(),
            &env::current_account_id(),
            NO_DEPOSIT,
            env::prepaid_gas() / 2,
        ));
    }

    pub fn resolve_unstake(
        &mut self,
        appchain_id: AppchainId,
        account_id: AccountId,
        amount: U128,
    ) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let amount: u128 = amount.0;
                let mut validator_ids = self
                    .appchain_data_validator_ids
                    .get(&appchain_id)
                    .expect("Appchain not found")
                    .clone();
                // Remove the validator
                validator_ids.retain(|v| {
                    self.validator_data_account_id
                        .get(&(appchain_id.clone(), v.to_string()))
                        .unwrap()
                        != account_id
                });
                // Update state
                self.appchain_data_validator_ids
                    .insert(&appchain_id, &validator_ids);
                let staked_balance = self
                    .appchain_data_staked_balance
                    .get(&appchain_id)
                    .unwrap_or(0);
                self.appchain_data_staked_balance
                    .insert(&appchain_id, &(staked_balance - amount));
                self.total_staked_balance -= amount;
                // // Check to update validator set
                self.update_validator_set(appchain_id);
            }
            PromiseResult::Failed => {}
        }
    }

    pub fn activate_appchain(
        &mut self,
        appchain_id: AppchainId,
        boot_nodes: String,
        rpc_endpoint: String,
        chain_spec_url: String,
        chain_spec_hash: String,
        chain_spec_raw_url: String,
        chain_spec_raw_hash: String,
    ) -> PromiseOrValue<U128> {
        assert_ne!(
            self.appchain_data_status
                .get(&appchain_id)
                .expect("Appchain not found"),
            AppchainStatus::Auditing,
            "Appchain is in auditing."
        );
        // Only admin can do this
        self.assert_owner();
        // Can only activate a frozen appchain
        assert!(
            self.appchain_data_status.get(&appchain_id).unwrap() == AppchainStatus::Frozen,
            "Appchain status incorrect"
        );
        // Check validators
        assert!(
            self.appchain_data_validator_ids
                .get(&appchain_id)
                .unwrap()
                .len() as u32
                >= self.appchain_minium_validators,
            "Insufficient number of appchain validators"
        );

        let account_id = self
            .appchain_data_founder_id
            .get(&appchain_id)
            .unwrap()
            .clone();
        let bond_tokens = self.appchain_data_bond_tokens.get(&appchain_id).unwrap();
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
    }

    pub fn resolve_activate_appchain(
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
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                self.appchain_data_status
                    .insert(&appchain_id, &AppchainStatus::Active);
                self.appchain_data_boot_nodes
                    .insert(&appchain_id, &boot_nodes);
                self.appchain_data_rpc_endpoint
                    .insert(&appchain_id, &rpc_endpoint);
                self.appchain_data_bond_tokens.insert(&appchain_id, &0);

                // Check to update validator set
                self.update_validator_set(appchain_id.clone());
                self.appchain_data_chain_spec_url
                    .insert(&appchain_id, &chain_spec_url);
                self.appchain_data_chain_spec_hash
                    .insert(&appchain_id, &chain_spec_hash);
                self.appchain_data_chain_spec_raw_url
                    .insert(&appchain_id, &chain_spec_raw_url);
                self.appchain_data_chain_spec_raw_hash
                    .insert(&appchain_id, &chain_spec_raw_hash);
                self.appchain_data_status.get(&appchain_id)
            }
            PromiseResult::Failed => self.appchain_data_status.get(&appchain_id),
        }
    }

    pub fn freeze_appchain(&mut self, appchain_id: AppchainId) {
        if !self.appchain_data_founder_id.contains_key(&appchain_id) {
            panic!("Appchain not found");
        }

        self.assert_owner();

        // Check status
        assert!(
            self.appchain_data_status.get(&appchain_id).unwrap() == AppchainStatus::Active,
            "Appchain status incorrect"
        );

        // Update state
        self.appchain_data_status
            .insert(&appchain_id, &AppchainStatus::Frozen);
    }

    fn update_validator_set(&mut self, appchain_id: AppchainId) -> bool {
        let next_validator_set_option = self.next_validator_set(appchain_id.clone());

        self.appchain_data_validators_timestamp
            .insert(&appchain_id, &env::block_timestamp());

        // Check status
        if self.appchain_data_status.get(&appchain_id).unwrap() != AppchainStatus::Active {
            return false;
        }

        if next_validator_set_option.is_some() {
            let next_validator_set = next_validator_set_option.unwrap();
            let seq_num = next_validator_set.seq_num;
            if (self
                .appchain_data_validator_ids
                .get(&appchain_id)
                .unwrap()
                .len() as u32)
                < self.appchain_minium_validators
            {
                self.appchain_data_status
                    .insert(&appchain_id, &AppchainStatus::Frozen);
                self.appchain_data_validator_set.insert(
                    &(appchain_id.clone(), seq_num),
                    &ValidatorSet {
                        seq_num: seq_num,
                        validators: vec![],
                    },
                );
            } else {
                self.appchain_data_validator_set
                    .insert(&(appchain_id.clone(), seq_num), &next_validator_set);
            }
            self.appchain_data_validator_sets_len
                .insert(&appchain_id, &(seq_num + 1));
        }

        true
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

#[near_bindgen]
impl OctopusRelay {
    fn lock_token(
        &mut self,
        appchain_id: AppchainId,
        receiver_id: String,
        token_id: AccountId,
        amount: u128,
    ) -> U128 {
        let allowed_amount: u128 = self
            .get_bridge_allowed_amount(appchain_id.clone(), token_id.clone())
            .into();
        assert!(allowed_amount >= amount.into(), "Bridge not allowed");

        let total_locked: Balance = self
            .token_appchain_total_locked
            .get(&(token_id.clone(), appchain_id.clone()))
            .unwrap_or(0);
        let next_total_locked = total_locked + u128::from(amount);
        self.token_appchain_total_locked.insert(
            &(token_id.clone(), appchain_id.clone()),
            &(next_total_locked),
        );
        let seq_num = self.locked_len_appchain_map.get(&appchain_id).unwrap_or(0);
        self.locked_len_appchain_map
            .insert(&appchain_id, &(seq_num + 1));
        self.locked_appchain_map.insert(
            &(appchain_id.clone(), seq_num),
            &Locked {
                seq_num,
                token_id,
                appchain_id,
                receiver_id,
                amount: amount.into(),
                block_height: env::block_index(),
            },
        );
        amount.into()
    }

    pub fn get_locked_events(
        &self,
        appchain_id: AppchainId,
        start: SeqNum,
        limit: SeqNum,
    ) -> Vec<Locked> {
        let locked_len = self.locked_len_appchain_map.get(&appchain_id).unwrap();
        (start..std::cmp::min(start + limit, locked_len))
            .map(|index| {
                self.locked_appchain_map
                    .get(&(appchain_id.clone(), index))
                    .unwrap()
            })
            .collect()
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
