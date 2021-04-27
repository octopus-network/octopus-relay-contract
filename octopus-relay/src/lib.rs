pub mod types;

// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use crate::types::{Appchain, AppchainStatus, Delegation, LiteValidator, Validator, ValidatorSet};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_self, env, ext_contract, log, near_bindgen, wee_alloc, AccountId, Balance, BlockHeight,
    PromiseOrValue,
};
use std::collections::HashMap;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const NO_DEPOSIT: Balance = 0;
const SINGLE_CALL_GAS: u64 = 5_000_000_000_000;
const DECIMALS_BASE: Balance = 1000_000_000_000_000_000_000_000;

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
    token_contract_id: AccountId,
    appchain_minium_validators: u32,
    minium_staking_amount: u128,
    total_staked_balance: u128,

    // data for Appchain
    appchain_data_founder_id: HashMap<AppchainId, AccountId>,
    appchain_data_name: HashMap<AppchainId, String>,
    appchain_data_website_url: HashMap<AppchainId, String>,
    appchain_data_github_address: HashMap<AppchainId, String>,
    appchain_data_chain_spec_url: HashMap<AppchainId, String>,
    appchain_data_chain_spec_hash: HashMap<AppchainId, String>,
    appchain_data_boot_nodes: HashMap<AppchainId, String>,
    appchain_data_rpc_endpoint: HashMap<AppchainId, String>,
    appchain_data_bond_tokens: HashMap<AppchainId, Balance>,
    appchain_data_validator_ids: HashMap<AppchainId, Vec<ValidatorId>>,
    appchain_data_validators_timestamp: HashMap<AppchainId, u64>,
    appchain_data_status: HashMap<AppchainId, AppchainStatus>,
    appchain_data_block_height: HashMap<AppchainId, BlockHeight>,

    is_appchain_name_registered: HashMap<String, bool>,

    appchain_data_validator_sets_len: HashMap<AppchainId, SeqNum>,
    appchain_data_validator_set: HashMap<(AppchainId, SeqNum), ValidatorSet>,

    // data for Validator
    validator_data_account_id: HashMap<(AppchainId, ValidatorId), AccountId>,
    validator_data_weight: HashMap<(AppchainId, ValidatorId), u32>,
    validator_data_staked_amount: HashMap<(AppchainId, ValidatorId), Balance>,
    validator_data_block_height: HashMap<(AppchainId, ValidatorId), BlockHeight>,
    validator_data_delegation_ids: HashMap<(AppchainId, ValidatorId), Vec<AccountId>>,

    // data for Delegation
    delegation_data_amount: HashMap<(AppchainId, ValidatorId, DelegatorId), Balance>,
    delegation_data_account_id: HashMap<(AppchainId, ValidatorId, DelegatorId), AccountId>,
    delegation_data_block_height: HashMap<(AppchainId, ValidatorId, DelegatorId), BlockHeight>,

    test_data: HashMap<u32, HashMap<u32, u32>>,
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

#[ext_contract(ext_oct_token)]
pub trait ExtOCTContract {
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
    ) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized");
        assert_self();
        Self {
            token_contract_id,
            total_staked_balance: 0,
            appchain_minium_validators,
            minium_staking_amount: minium_staking_amount.0,

            appchain_data_founder_id: HashMap::default(),
            appchain_data_name: HashMap::default(),
            appchain_data_website_url: HashMap::default(),
            appchain_data_github_address: HashMap::default(),
            appchain_data_chain_spec_url: HashMap::default(),
            appchain_data_chain_spec_hash: HashMap::default(),
            appchain_data_boot_nodes: HashMap::default(),
            appchain_data_rpc_endpoint: HashMap::default(),
            appchain_data_bond_tokens: HashMap::default(),
            appchain_data_validator_ids: HashMap::default(),
            appchain_data_validators_timestamp: HashMap::default(),
            appchain_data_status: HashMap::default(),
            appchain_data_block_height: HashMap::default(),

            is_appchain_name_registered: HashMap::default(),

            appchain_data_validator_sets_len: HashMap::default(),
            appchain_data_validator_set: HashMap::default(),

            validator_data_account_id: HashMap::default(),
            validator_data_weight: HashMap::default(),
            validator_data_staked_amount: HashMap::default(),
            validator_data_block_height: HashMap::default(),
            validator_data_delegation_ids: HashMap::default(),

            delegation_data_amount: HashMap::default(),
            delegation_data_account_id: HashMap::default(),
            delegation_data_block_height: HashMap::default(),

            test_data: HashMap::default(),
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
        let appchain_id = self.appchain_data_name.len() as u32;

        assert!(
            !self
                .is_appchain_name_registered
                .contains_key(&appchain_name),
            "Appchain_name is already registered"
        );

        // Default validator set
        self.appchain_data_founder_id
            .insert(appchain_id, account_id);
        self.appchain_data_name
            .insert(appchain_id, appchain_name.clone());
        self.appchain_data_website_url
            .insert(appchain_id, website_url);
        self.appchain_data_github_address
            .insert(appchain_id, github_address);
        self.appchain_data_chain_spec_url
            .insert(appchain_id, String::from(""));
        self.appchain_data_chain_spec_hash
            .insert(appchain_id, String::from(""));
        self.appchain_data_boot_nodes
            .insert(appchain_id, String::from(""));
        self.appchain_data_rpc_endpoint
            .insert(appchain_id, String::from(""));
        self.appchain_data_bond_tokens
            .insert(appchain_id, bond_tokens);
        self.appchain_data_validator_ids
            .insert(appchain_id, Vec::default());
        self.appchain_data_status
            .insert(appchain_id, AppchainStatus::InProgress);
        self.appchain_data_block_height
            .insert(appchain_id, env::block_index());
        self.is_appchain_name_registered.insert(appchain_name, true);
        self.appchain_data_validator_sets_len.insert(appchain_id, 0);
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
            account_id == self.appchain_data_founder_id[&appchain_id],
            "You aren't the appchain founder!"
        );

        self.appchain_data_chain_spec_url
            .insert(appchain_id, chain_spec_url);
        self.appchain_data_chain_spec_hash
            .insert(appchain_id, chain_spec_hash);
        self.appchain_data_website_url
            .insert(appchain_id, website_url);
        self.appchain_data_github_address
            .insert(appchain_id, github_address);
        self.appchain_data_status
            .insert(appchain_id, AppchainStatus::Frozen);
    }

    pub fn get_appchains(&self, from_index: u32, limit: u32) -> Vec<Appchain> {
        (from_index..std::cmp::min(from_index + limit, self.appchain_data_name.len() as u32))
            .map(|index| self.get_appchain(index).unwrap())
            .collect()
    }

    pub fn get_num_appchains(&self) -> usize {
        self.appchain_data_name.len()
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
                    .unwrap_or(&String::from(""))
                    .clone(),
                appchain_name: appchain_name_option.unwrap().clone(),
                website_url: self
                    .appchain_data_website_url
                    .get(&appchain_id)
                    .unwrap_or(&String::from(""))
                    .clone(),
                github_address: self
                    .appchain_data_github_address
                    .get(&appchain_id)
                    .unwrap_or(&String::from(""))
                    .clone(),
                chain_spec_url: self
                    .appchain_data_chain_spec_url
                    .get(&appchain_id)
                    .unwrap_or(&String::from(""))
                    .clone(),
                chain_spec_hash: self
                    .appchain_data_chain_spec_hash
                    .get(&appchain_id)
                    .unwrap_or(&String::from(""))
                    .clone(),
                boot_nodes: self
                    .appchain_data_boot_nodes
                    .get(&appchain_id)
                    .unwrap_or(&String::from(""))
                    .clone(),
                rpc_endpoint: self
                    .appchain_data_rpc_endpoint
                    .get(&appchain_id)
                    .unwrap_or(&String::from(""))
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
                    .unwrap_or(&0)
                    .clone(),
                status: self.appchain_data_status.get(&appchain_id).unwrap().clone(),
                block_height: self
                    .appchain_data_block_height
                    .get(&appchain_id)
                    .unwrap()
                    .clone(),
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

    pub fn next_validator_set(
        &self,
        appchain_id: AppchainId,
        seq_num: SeqNum,
    ) -> Option<ValidatorSet> {
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
                weight: self.validator_data_weight[&(appchain_id, validator_id.clone())],
                staked_amount: self.validator_data_staked_amount
                    [&(appchain_id, validator_id.clone())]
                    .into(),
                block_height: self.validator_data_block_height
                    [&(appchain_id, validator_id.clone())],
                delegations: self.validator_data_delegation_ids
                    [&(appchain_id, validator_id.clone())]
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
                amount: self.delegation_data_amount
                    [&(appchain_id, validator_id.clone(), delegator_id.clone())]
                    .into(),
                block_height: self.delegation_data_block_height
                    [&(appchain_id, validator_id, delegator_id)],
            })
        } else {
            None
        }
    }

    // Returns the appchain current validator_set index
    pub fn get_curr_validator_set_index(&self, appchain_id: AppchainId) -> u32 {
        self.appchain_data_validator_sets_len[&appchain_id] - 1
    }

    pub fn get_validator_set(&self, appchain_id: AppchainId) -> Option<ValidatorSet> {
        let seq_num = self.get_curr_validator_set_index(appchain_id);
        let next_validator_set = self.next_validator_set(appchain_id, seq_num + 1);
        if next_validator_set.is_some() {
            next_validator_set
        } else {
            self.get_validator_set_by_seq_num(appchain_id, seq_num)
        }
    }

    pub fn get_validator_set_by_seq_num(
        &self,
        appchain_id: AppchainId,
        seq_num: u32,
    ) -> Option<ValidatorSet> {
        let validator_set_option = self
            .appchain_data_validator_set
            .get(&(appchain_id, seq_num));
        if validator_set_option.is_some() {
            Some(validator_set_option.unwrap().clone())
        } else {
            None
        }
    }

    fn staking(&mut self, appchain_id: AppchainId, id: String, amount: u128) {
        let account_id = env::signer_account_id();

        // Check amount
        assert!(
            amount >= self.minium_staking_amount,
            "Insufficient staking amount"
        );

        let weight = (amount / DECIMALS_BASE) as u32;

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
            .insert((appchain_id, id.clone()), account_id);
        self.validator_data_weight
            .insert((appchain_id, id.clone()), weight);
        self.validator_data_staked_amount
            .insert((appchain_id, id.clone()), amount);
        self.validator_data_block_height
            .insert((appchain_id, id.clone()), env::block_index());

        self.validator_data_delegation_ids
            .insert((appchain_id, id.clone()), Vec::default());

        let mut validator_ids: Vec<ValidatorId> = self
            .appchain_data_validator_ids
            .get(&appchain_id)
            .unwrap()
            .clone();

        validator_ids.push(id.clone());
        self.appchain_data_validator_ids
            .insert(appchain_id, validator_ids);
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

        let weight = (amount / DECIMALS_BASE) as u32;

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
                    .insert((appchain_id, v.id.clone()), v.staked_amount.0 + amount);
                self.validator_data_weight
                    .insert((appchain_id, v.id.clone()), v.weight + weight);
                found = true;
            }
        }

        if !found {
            panic!("You are not staked on the appchain");
        }

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

        ext_oct_token::ft_transfer(
            account_id.clone(),
            validator.staked_amount.into(),
            None,
            &self.token_contract_id,
            1,
            SINGLE_CALL_GAS,
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
            self.validator_data_account_id[&(appchain_id, v.to_string())] != account_id
        });

        // Update state
        self.appchain_data_validator_ids
            .insert(appchain_id, validator_ids);
        self.total_staked_balance -= amount;

        // // Check to update validator set
        self.update_validator_set(appchain_id);
    }

    pub fn activate_appchain(
        &mut self,
        appchain_id: AppchainId,
        boot_nodes: String,
        rpc_endpoint: String,
    ) {
        if !self.appchain_data_name.contains_key(&appchain_id) {
            panic!("Appchain not found");
        }
        // Only admin can do this
        assert_self();
        // Can only activate a frozen appchain
        assert!(
            self.appchain_data_status[&appchain_id] == AppchainStatus::Frozen,
            "Appchain status incorrect"
        );
        // Check validators
        assert!(
            self.appchain_data_validator_ids[&appchain_id].len() as u32
                >= self.appchain_minium_validators,
            "Insufficient number of appchain validators"
        );

        let account_id = self.appchain_data_founder_id[&appchain_id].clone();
        let bond_tokens = self.appchain_data_bond_tokens[&appchain_id];
        ext_oct_token::ft_transfer(
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
        ));
    }

    pub fn resolve_activate_appchain(
        &mut self,
        appchain_id: AppchainId,
        boot_nodes: String,
        rpc_endpoint: String,
    ) {
        // Update state
        self.appchain_data_status
            .insert(appchain_id, AppchainStatus::Active);
        self.appchain_data_boot_nodes
            .insert(appchain_id, boot_nodes);
        self.appchain_data_rpc_endpoint
            .insert(appchain_id, rpc_endpoint);

        // Check to update validator set
        self.update_validator_set(appchain_id);
    }

    pub fn freeze_appchain(&mut self, appchain_id: AppchainId) {
        if !self.appchain_data_name.contains_key(&appchain_id) {
            panic!("Appchain not found");
        }

        assert_self();

        // Check status
        assert!(
            self.appchain_data_status[&appchain_id] == AppchainStatus::Active,
            "Appchain status incorrect"
        );

        // Update state
        self.appchain_data_status
            .insert(appchain_id, AppchainStatus::Frozen);
    }

    fn update_validator_set(&mut self, appchain_id: AppchainId) -> bool {
        let seq_num = self.get_curr_validator_set_index(appchain_id);
        let next_seq_num = seq_num + 1;
        let next_validator_set = self.next_validator_set(appchain_id, next_seq_num);

        self.appchain_data_validators_timestamp
            .insert(appchain_id, env::block_timestamp());

        // Check status
        if self.appchain_data_status[&appchain_id] != AppchainStatus::Active {
            return false;
        }

        if next_validator_set.is_some() {
            if (self.appchain_data_validator_ids[&appchain_id].len() as u32)
                < self.appchain_minium_validators
            {
                self.appchain_data_status
                    .insert(appchain_id, AppchainStatus::Frozen);
                self.appchain_data_validator_set.insert(
                    (appchain_id, next_seq_num),
                    ValidatorSet {
                        seq_num: next_seq_num,
                        validators: vec![],
                    },
                );
            } else {
                self.appchain_data_validator_set
                    .insert((appchain_id, next_seq_num), next_validator_set.unwrap());
            }
            self.appchain_data_validator_sets_len
                .insert(appchain_id, next_seq_num + 1);
        }

        true
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
