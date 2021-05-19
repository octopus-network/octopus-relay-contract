pub mod types;

// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use crate::types::{
    Appchain, AppchainStatus, BridgeToken, Delegation, LiteValidator, Validator, ValidatorSet,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_self, env, ext_contract, log, near_bindgen, wee_alloc, AccountId, Balance, BlockHeight,
    PromiseOrValue,
};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const NO_DEPOSIT: Balance = 0;
const GAS_FOR_FT_TRANSFER_CALL: u64 = 30_000_000_000_000;
const SINGLE_CALL_GAS: u64 = 10_000_000_000_000;
const OCT_DECIMALS_BASE: Balance = 1000_000_000_000_000_000_000_000;

const VALIDATOR_SET_CYCLE: u64 = 60000000000;
// const VALIDATOR_SET_CYCLE: u64 = 86400000000000;

pub type AppchainId = u32;
pub type ValidatorId = String;
pub type DelegatorId = String;
pub type SeqNum = u32;

// Structs in Rust are similar to other languages, and may include impl keyword as shown below
// Note: the names of the structs are not important when calling the smart contract, but the function names are
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct OctopusRelay {
    pub token_contract_id: AccountId,
    pub appchain_minium_validators: u32,
    pub minium_staking_amount: Balance,
    pub total_staked_balance: Balance,
    pub appchain_len: u32,

    // data for Appchain
    pub appchain_data_founder_id: LookupMap<AppchainId, AccountId>,
    pub appchain_data_name: LookupMap<AppchainId, String>,
    pub appchain_data_website_url: LookupMap<AppchainId, String>,
    pub appchain_data_github_address: LookupMap<AppchainId, String>,
    pub appchain_data_chain_spec_url: LookupMap<AppchainId, String>,
    pub appchain_data_chain_spec_hash: LookupMap<AppchainId, String>,
    pub appchain_data_boot_nodes: LookupMap<AppchainId, String>,
    pub appchain_data_rpc_endpoint: LookupMap<AppchainId, String>,
    pub appchain_data_bond_tokens: LookupMap<AppchainId, Balance>,
    pub appchain_data_validator_ids: LookupMap<AppchainId, Vec<ValidatorId>>,
    pub appchain_data_validators_timestamp: LookupMap<AppchainId, u64>,
    pub appchain_data_status: LookupMap<AppchainId, AppchainStatus>,
    pub appchain_data_block_height: LookupMap<AppchainId, BlockHeight>,
    pub appchain_data_staked_balance: LookupMap<AppchainId, Balance>,

    pub is_appchain_name_registered: LookupMap<String, bool>,

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
    pub bridge_token_data_is_active: LookupMap<AccountId, bool>,
    pub bridge_token_data_price: LookupMap<AccountId, Balance>,
    pub bridge_token_data_decimals: LookupMap<AccountId, u32>,
    pub bridge_limit_ratio: u16, // 100 as 1%
    pub owner: AccountId,
    pub bridge_is_active: bool,
    pub oct_token_price: u128, // 1_000_000 as 1usd

    pub token_appchain_total_locked: LookupMap<(AccountId, AppchainId), Balance>,
}

#[ext_contract(ext_self)]
pub trait ExtOctopusRelay {
    fn resolve_unstaking(&mut self, appchain_id: AppchainId, account_id: AccountId, amount: U128);
    fn resolve_activate_appchain(
        &mut self,
        appchain_id: AppchainId,
        boot_nodes: String,
        rpc_endpoint: String,
    );
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
            token_contract_id,
            total_staked_balance: 0,
            appchain_minium_validators,
            minium_staking_amount: minium_staking_amount.0,
            appchain_len: 0,

            appchain_data_founder_id: LookupMap::new(b"afi".to_vec()),
            appchain_data_name: LookupMap::new(b"an".to_vec()),
            appchain_data_website_url: LookupMap::new(b"wu".to_vec()),
            appchain_data_github_address: LookupMap::new(b"ga".to_vec()),
            appchain_data_chain_spec_url: LookupMap::new(b"csu".to_vec()),
            appchain_data_chain_spec_hash: LookupMap::new(b"csh".to_vec()),
            appchain_data_boot_nodes: LookupMap::new(b"bn".to_vec()),
            appchain_data_rpc_endpoint: LookupMap::new(b"re".to_vec()),
            appchain_data_bond_tokens: LookupMap::new(b"bt".to_vec()),
            appchain_data_validator_ids: LookupMap::new(b"vi".to_vec()),
            appchain_data_validators_timestamp: LookupMap::new(b"vt".to_vec()),
            appchain_data_status: LookupMap::new(b"st".to_vec()),
            appchain_data_block_height: LookupMap::new(b"abh".to_vec()),
            appchain_data_staked_balance: LookupMap::new(b"sb".to_vec()),

            is_appchain_name_registered: LookupMap::new(b"ir".to_vec()),

            appchain_data_validator_sets_len: LookupMap::new(b"ir".to_vec()),
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
            bridge_symbol_to_token: LookupMap::new(b"st".to_vec()),
            bridge_token_data_is_active: LookupMap::new(b"ta".to_vec()),
            bridge_token_data_price: LookupMap::new(b"tp".to_vec()),
            bridge_token_data_decimals: LookupMap::new(b"td".to_vec()),

            owner: env::current_account_id(),
            bridge_is_active: true,
            bridge_limit_ratio,
            oct_token_price: oct_token_price.into(),

            token_appchain_total_locked: LookupMap::new(b"tab".to_vec()),
        }
    }

    pub fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // Verifying that we were called by fungible token contract that we expect.
        assert_eq!(
            &env::predecessor_account_id(),
            &self.token_contract_id,
            "Only supports the OCT token contract"
        );
        log!(
            "in {} tokens from @{} ft_on_transfer, msg = {}",
            amount.0,
            sender_id.as_ref(),
            msg
        );

        let msg_vec: Vec<String> = msg.split(",").map(|s| s.to_string()).collect();

        match msg_vec.get(0).unwrap().as_str() {
            "register_appchain" => {
                assert_eq!(msg_vec.len(), 4, "params length wrong!");
                self.register_appchain(
                    msg_vec.get(1).unwrap().to_string(),
                    msg_vec.get(2).unwrap().to_string(),
                    msg_vec.get(3).unwrap().to_string(),
                    amount.0,
                );
                PromiseOrValue::Value(0.into())
            }
            "staking" => {
                assert_eq!(msg_vec.len(), 3, "params length wrong!");
                self.staking(
                    msg_vec.get(1).unwrap().parse::<u32>().unwrap(),
                    msg_vec.get(2).unwrap().to_string(),
                    amount.0,
                );
                PromiseOrValue::Value(0.into())
            }
            "staking_more" => {
                assert_eq!(msg_vec.len(), 2, "params length wrong!");
                self.staking_more(msg_vec.get(1).unwrap().parse::<u32>().unwrap(), amount.0);
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
        appchain_name: String,
        website_url: String,
        github_address: String,
        bond_tokens: u128,
    ) {
        let account_id = env::signer_account_id();
        let appchain_id = self.appchain_len;

        assert!(
            !self
                .is_appchain_name_registered
                .contains_key(&appchain_name),
            "Appchain_name is already registered"
        );

        // Default validator set
        self.appchain_data_founder_id
            .insert(&appchain_id, &account_id);
        self.appchain_data_name
            .insert(&appchain_id, &appchain_name.clone());
        self.appchain_data_website_url
            .insert(&appchain_id, &website_url);
        self.appchain_data_github_address
            .insert(&appchain_id, &github_address);
        self.appchain_data_chain_spec_url
            .insert(&appchain_id, &String::from(""));
        self.appchain_data_chain_spec_hash
            .insert(&appchain_id, &String::from(""));
        self.appchain_data_boot_nodes
            .insert(&appchain_id, &String::from(""));
        self.appchain_data_rpc_endpoint
            .insert(&appchain_id, &String::from(""));
        self.appchain_data_bond_tokens
            .insert(&appchain_id, &bond_tokens);
        self.appchain_data_validator_ids
            .insert(&appchain_id, &Vec::default());
        self.appchain_data_status
            .insert(&appchain_id, &AppchainStatus::InProgress);
        self.appchain_data_block_height
            .insert(&appchain_id, &env::block_index());
        self.is_appchain_name_registered
            .insert(&appchain_name, &true);
        self.appchain_data_validator_sets_len
            .insert(&appchain_id, &0);
        self.appchain_len += 1;
        log!(
            "Appchain added, appchain_id is {}, bund_tokens is {}.",
            appchain_id,
            bond_tokens
        );
    }

    pub fn update_appchain(
        &mut self,
        appchain_id: AppchainId,
        website_url: String,
        github_address: String,
        chain_spec_url: String,
        chain_spec_hash: String,
    ) {
        let account_id = env::signer_account_id();

        // Only appchain founder can do this
        assert!(
            account_id == self.appchain_data_founder_id.get(&appchain_id).unwrap(),
            "You aren't the appchain founder!"
        );

        self.appchain_data_chain_spec_url
            .insert(&appchain_id, &chain_spec_url);
        self.appchain_data_chain_spec_hash
            .insert(&appchain_id, &chain_spec_hash);
        self.appchain_data_website_url
            .insert(&appchain_id, &website_url);
        self.appchain_data_github_address
            .insert(&appchain_id, &github_address);
        self.appchain_data_status
            .insert(&appchain_id, &AppchainStatus::Frozen);
    }

    pub fn get_appchains(&self, from_index: u32, limit: u32) -> Vec<Appchain> {
        (from_index..std::cmp::min(from_index + limit, self.appchain_len as u32))
            .map(|index| self.get_appchain(index).unwrap())
            .collect()
    }

    pub fn get_num_appchains(&self) -> u32 {
        self.appchain_len
    }

    /// Returns the total staking balance.
    pub fn get_total_staked_balance(&self) -> U128 {
        self.total_staked_balance.into()
    }

    pub fn get_minium_staking_amount(&self) -> U128 {
        self.minium_staking_amount.into()
    }

    pub fn get_appchain(&self, appchain_id: AppchainId) -> Option<Appchain> {
        let appchain_name_option = self.appchain_data_name.get(&appchain_id);
        if appchain_name_option.is_some() {
            Some(Appchain {
                id: appchain_id,
                founder_id: self
                    .appchain_data_founder_id
                    .get(&appchain_id)
                    .unwrap_or(String::from(""))
                    .clone(),
                appchain_name: appchain_name_option.unwrap().clone(),
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
                validators: self.get_validators(appchain_id).unwrap_or_default(),
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

    pub fn get_appchain_minium_validators(&self) -> u32 {
        self.appchain_minium_validators
    }

    pub fn get_validators(&self, appchain_id: AppchainId) -> Option<Vec<Validator>> {
        self.appchain_data_validator_ids
            .get(&appchain_id)
            .expect("Appchain not found")
            .iter()
            .map(|v| self.get_validator(appchain_id, v.to_string()))
            .collect()
    }

    pub fn next_validator_set(&self, appchain_id: AppchainId) -> Option<ValidatorSet> {
        let seq_num = self.get_curr_validator_set_len(appchain_id);
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
            .get(&(appchain_id, validator_id.clone()));
        if account_id_option.is_some() {
            Some(Validator {
                id: validator_id.clone(),
                account_id: account_id_option.unwrap().to_string(),
                weight: self
                    .validator_data_weight
                    .get(&(appchain_id, validator_id.clone()))
                    .unwrap(),
                staked_amount: self
                    .validator_data_staked_amount
                    .get(&(appchain_id, validator_id.clone()))
                    .unwrap()
                    .into(),
                block_height: self
                    .validator_data_block_height
                    .get(&(appchain_id, validator_id.clone()))
                    .unwrap(),
                delegations: self
                    .validator_data_delegation_ids
                    .get(&(appchain_id, validator_id.clone()))
                    .unwrap()
                    .iter()
                    .map(|d| {
                        self.get_delegation(appchain_id, validator_id.clone(), d.to_string())
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
            appchain_id,
            validator_id.clone(),
            delegator_id.clone(),
        ));
        if account_id_option.is_some() {
            Some(Delegation {
                id: delegator_id.clone(),
                account_id: account_id_option.unwrap().to_string(),
                amount: self
                    .delegation_data_amount
                    .get(&(appchain_id, validator_id.clone(), delegator_id.clone()))
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
        let next_validator_set = self.next_validator_set(appchain_id);
        if next_validator_set.is_some() {
            next_validator_set
        } else {
            let validator_set_len = self.get_curr_validator_set_len(appchain_id);
            if validator_set_len == 0 {
                return None;
            }
            self.get_validator_set_by_seq_num(appchain_id, validator_set_len - 1)
        }
    }

    pub fn get_validator_set_by_seq_num(
        &self,
        appchain_id: AppchainId,
        seq_num: u32,
    ) -> Option<ValidatorSet> {
        if seq_num == self.get_curr_validator_set_len(appchain_id) {
            return self.next_validator_set(appchain_id);
        } else {
            return self
                .appchain_data_validator_set
                .get(&(appchain_id, seq_num));
        }
    }

    fn staking(&mut self, appchain_id: AppchainId, id: String, amount: u128) {
        let account_id = env::signer_account_id();

        // Check amount
        assert!(
            amount >= self.minium_staking_amount,
            "Insufficient staking amount"
        );

        let weight = (amount / OCT_DECIMALS_BASE) as u32;

        if !self.appchain_data_name.contains_key(&appchain_id) {
            panic!("Appchain not found");
        }

        let validators = self.get_validators(appchain_id).unwrap();
        for v in validators {
            assert!(
                v.account_id != account_id,
                "You are already staked on the appchain!"
            );
        }

        self.validator_data_account_id
            .insert(&(appchain_id, id.clone()), &account_id);
        self.validator_data_weight
            .insert(&(appchain_id, id.clone()), &weight);
        self.validator_data_staked_amount
            .insert(&(appchain_id, id.clone()), &amount);
        self.validator_data_block_height
            .insert(&(appchain_id, id.clone()), &env::block_index());

        self.validator_data_delegation_ids
            .insert(&(appchain_id, id.clone()), &Vec::default());

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

    fn staking_more(&mut self, appchain_id: AppchainId, amount: u128) {
        let account_id = env::signer_account_id();

        // Check amount
        assert!(
            amount >= self.minium_staking_amount,
            "Insufficient staking amount"
        );

        let weight = (amount / OCT_DECIMALS_BASE) as u32;

        let mut validators = self
            .get_validators(appchain_id)
            .expect("Appchain not found");

        validators
            .iter()
            .find(|v| v.account_id == account_id)
            .expect("You are not staked on the appchain");

        let mut found = false;
        for v in validators.iter_mut() {
            if v.account_id == account_id {
                self.validator_data_staked_amount
                    .insert(&(appchain_id, v.id.clone()), &(v.staked_amount.0 + amount));
                self.validator_data_weight
                    .insert(&(appchain_id, v.id.clone()), &(v.weight + weight));
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
    pub fn unstaking(&mut self, appchain_id: AppchainId) {
        let account_id = env::signer_account_id();
        let validators = self.get_validators(appchain_id).unwrap();

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
        .then(ext_self::resolve_unstaking(
            appchain_id,
            account_id,
            validator.staked_amount.into(),
            &env::current_account_id(),
            NO_DEPOSIT,
            SINGLE_CALL_GAS,
        ));
    }

    pub fn resolve_unstaking(
        &mut self,
        appchain_id: AppchainId,
        account_id: AccountId,
        amount: U128,
    ) {
        let amount: u128 = amount.0;
        let mut validator_ids = self
            .appchain_data_validator_ids
            .get(&appchain_id)
            .expect("Appchain not found")
            .clone();

        // Remove the validator
        validator_ids.retain(|v| {
            self.validator_data_account_id
                .get(&(appchain_id, v.to_string()))
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

    pub fn activate_appchain(
        &mut self,
        appchain_id: AppchainId,
        boot_nodes: String,
        rpc_endpoint: String,
    ) -> PromiseOrValue<Option<AppchainStatus>> {
        if !self.appchain_data_name.contains_key(&appchain_id) {
            panic!("Appchain not found");
        }
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
            SINGLE_CALL_GAS,
        )
        .then(ext_self::resolve_activate_appchain(
            appchain_id,
            boot_nodes,
            rpc_endpoint,
            &env::current_account_id(),
            NO_DEPOSIT,
            SINGLE_CALL_GAS,
        ))
        .into()
    }

    pub fn resolve_activate_appchain(
        &mut self,
        appchain_id: AppchainId,
        boot_nodes: String,
        rpc_endpoint: String,
    ) -> Option<AppchainStatus> {
        // Update state
        self.appchain_data_status
            .insert(&appchain_id, &AppchainStatus::Active);
        self.appchain_data_boot_nodes
            .insert(&appchain_id, &boot_nodes);
        self.appchain_data_rpc_endpoint
            .insert(&appchain_id, &rpc_endpoint);
        self.appchain_data_bond_tokens.insert(&appchain_id, &0);

        // Check to update validator set
        self.update_validator_set(appchain_id);
        self.appchain_data_status.get(&appchain_id)
    }

    pub fn freeze_appchain(&mut self, appchain_id: AppchainId) {
        if !self.appchain_data_name.contains_key(&appchain_id) {
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
        let next_validator_set_option = self.next_validator_set(appchain_id);

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
                    &(appchain_id, seq_num),
                    &ValidatorSet {
                        seq_num: seq_num,
                        validators: vec![],
                    },
                );
            } else {
                self.appchain_data_validator_set
                    .insert(&(appchain_id, seq_num), &next_validator_set);
            }
            self.appchain_data_validator_sets_len
                .insert(&appchain_id, &(seq_num + 1));
        }

        true
    }

    pub fn pause_bridge(&mut self) {
        assert!(self.bridge_is_active, "The bridge is already paused!");
        self.assert_owner();
        self.bridge_is_active = false;
    }

    pub fn resume_bridge(&mut self) {
        assert!(!self.bridge_is_active, "The bridge is active!");
        self.assert_owner();
        self.bridge_is_active = true;
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
        self.bridge_token_data_is_active.insert(&token_id, &true);
        self.bridge_token_data_price
            .insert(&token_id, &price.into());
        self.bridge_token_data_decimals.insert(&token_id, &decimals);
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

    pub fn after_token_lock(
        &mut self,
        token_id: AccountId,
        appchain_id: AppchainId,
        amount: U128,
    ) -> U128 {
        let total_locked: Balance = self
            .token_appchain_total_locked
            .get(&(token_id.clone(), appchain_id))
            .unwrap_or(0);
        let next_total_locked = total_locked + u128::from(amount);
        self.token_appchain_total_locked
            .insert(&(token_id, appchain_id), &(next_total_locked));
        next_total_locked.into()
    }

    pub fn get_bridge_token(&self, token_id: AccountId) -> Option<BridgeToken> {
        let bridge_token_symbol_option = self.bridge_token_data_symbol.get(&token_id);
        if bridge_token_symbol_option.is_some() {
            Some(BridgeToken {
                symbol: bridge_token_symbol_option.unwrap(),
                is_active: self.bridge_token_data_is_active.get(&token_id).unwrap(),
                price: self.bridge_token_data_price.get(&token_id).unwrap().into(),
                decimals: self.bridge_token_data_decimals.get(&token_id).unwrap(),
                token_id,
            })
        } else {
            None
        }
    }

    pub fn get_bridge_allowed(&self, appchain_id: AppchainId, token_id: AccountId) -> U128 {
        let is_active = self.bridge_is_active
            && self
                .bridge_token_data_is_active
                .get(&token_id)
                .unwrap_or(false);
        assert!(is_active, "The bridge is paused or does not exist");

        let staked_balance = self
            .appchain_data_staked_balance
            .get(&appchain_id)
            .unwrap_or(0);
        let token_price = self.bridge_token_data_price.get(&token_id).unwrap();
        let decimals = self.bridge_token_data_decimals.get(&token_id).unwrap();
        let bt_decimals_base = (10 as u128).pow(decimals);

        let limit_val = staked_balance / OCT_DECIMALS_BASE
            * self.oct_token_price
            * (self.bridge_limit_ratio as u128)
            / 10000;

        let mut total_used_val: Balance = 0;
        self.bridge_token_data_symbol.iter().for_each(|(bt_id, _)| {
            let bt_price = self.bridge_token_data_price.get(&bt_id).unwrap();
            let bt_locked = self
                .token_appchain_total_locked
                .get(&(bt_id, appchain_id))
                .unwrap_or(0);
            let used_val: Balance = bt_locked * bt_price / bt_decimals_base;
            log!("bt_price = {}", bt_price);
            log!("bt_locked = {}", bt_locked);
            log!("used_val = {}", used_val);
            total_used_val += used_val;
        });

        let rest_val = limit_val - total_used_val;

        let allowed_amount = rest_val * bt_decimals_base / token_price;
        allowed_amount.into()
    }
}

pub trait Ownable {
    fn assert_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.get_owner());
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
