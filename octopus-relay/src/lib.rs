// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::{
    env, log, near_bindgen, wee_alloc, AccountId, Balance, BlockHeight, PromiseOrValue,
    PromiseResult,
};
use std::collections::HashMap;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const NO_DEPOSIT: Balance = 0;
const SINGLE_CALL_GAS: u64 = 50_000_000_000_000;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Vote {
    Yes,
    No,
}

/// Describes the status of appchains
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainStatus {
    InProgress,
    Frozen,
    Active,
}

impl Default for AppchainStatus {
    fn default() -> Self {
        AppchainStatus::Frozen
    }
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Delegation {
    account_id: String,
    amount: u64,
    block_height: BlockHeight,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Validator {
    account_id: String,
    id: String,
    weight: u128,
    staked_amount: u128,
    block_height: BlockHeight,
    delegations: Vec<Delegation>,
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            account_id: String::from(""),
            id: String::from(""),
            weight: 0,
            staked_amount: 0,
            block_height: 0,
            delegations: vec![],
        }
    }
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorSet {
    pub sequence_number: u32,
    pub validators: Vec<Validator>,
}

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Appchain {
    pub id: u32,
    pub founder_id: AccountId,
    pub appchain_name: String,
    pub chain_spec_url: String,
    pub chain_spec_hash: String,
    pub bond_tokens: u128,
    pub validator_set: HashMap<u32, ValidatorSet>,
    pub validators: Vec<Validator>,
    pub status: AppchainStatus,
    pub block_height: BlockHeight,
}

// Structs in Rust are similar to other languages, and may include impl keyword as shown below
// Note: the names of the structs are not important when calling the smart contract, but the function names are
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct OctopusRelay {
    owner: AccountId,
    token_contract_id: AccountId,
    appchains: HashMap<u32, Appchain>,
    appchain_minium_validators: u32,
    minium_staking_amount: u128,
    total_staked_balance: u128,
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
        owner: AccountId,
        token_contract_id: AccountId,
        appchain_minium_validators: u32,
        minium_staking_amount: U128,
    ) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner,
            token_contract_id,
            appchains: HashMap::default(),
            total_staked_balance: 0,
            appchain_minium_validators,
            minium_staking_amount: minium_staking_amount.0,
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
                assert_eq!(msg_vec.len(), 2, "params length wrong!");
                self.register_appchain(msg_vec.get(1).unwrap().to_string(), amount.0);
                PromiseOrValue::Value(U128::from(0))
            }
            "staking" => {
                assert_eq!(msg_vec.len(), 3, "params length wrong!");
                self.staking(
                    msg_vec.get(1).unwrap().parse::<u32>().unwrap(),
                    msg_vec.get(2).unwrap().to_string(),
                    amount.0,
                );
                PromiseOrValue::Value(U128::from(0))
            }
            "staking_more" => {
                assert_eq!(msg_vec.len(), 2, "params length wrong!");
                self.staking_more(msg_vec.get(1).unwrap().parse::<u32>().unwrap(), amount.0);
                PromiseOrValue::Value(U128::from(0))
            }
            _ => {
                log!("Function name not matched, msg = {}", msg);
                PromiseOrValue::Value(amount)
            }
        }
    }

    fn register_appchain(&mut self, appchain_name: String, bond_tokens: u128) {
        let account_id = env::signer_account_id();
        let appchain_id = self.appchains.len() as u32;

        // Default validator set
        let mut validator_hash_map = HashMap::new();
        validator_hash_map.insert(
            0,
            ValidatorSet {
                sequence_number: 0,
                validators: vec![],
            },
        );

        let appchain = Appchain {
            id: appchain_id,
            founder_id: account_id.clone(),
            appchain_name: appchain_name.clone(),
            chain_spec_url: String::from(""),
            chain_spec_hash: String::from(""),
            bond_tokens,
            validator_set: validator_hash_map,
            validators: Vec::default(),
            status: AppchainStatus::InProgress,
            block_height: env::block_index(),
        };

        self.appchains.insert(appchain_id, appchain);
        log!(
            "Appchain added, appchain_id is {}, bund_tokens is {}.",
            appchain_id,
            bond_tokens
        );
    }

    pub fn update_appchain(
        &mut self,
        appchain_id: u32,
        chain_spec_url: String,
        chain_spec_hash: String,
    ) {
        let mut appchain = self
            .appchains
            .get(&appchain_id)
            .cloned()
            .expect("Appchain not found");

        let account_id = env::signer_account_id();

        // Only appchain founder can do this
        assert!(
            account_id == appchain.founder_id,
            "You aren't the appchain founder!"
        );
        appchain.chain_spec_url = chain_spec_url;
        appchain.chain_spec_hash = chain_spec_hash;
        appchain.status = AppchainStatus::Frozen;
        self.appchains.insert(appchain_id, appchain);
        log!(
            "appchain updated with chain_spec_url={}, chain_spec_hash={}.",
            self.appchains.get(&appchain_id).unwrap().chain_spec_url,
            self.appchains.get(&appchain_id).unwrap().chain_spec_hash
        );
    }

    pub fn get_appchains(&self, from_index: u32, limit: u32) -> Vec<&Appchain> {
        (from_index..std::cmp::min(from_index + limit, self.appchains.len() as u32))
            .map(|index| self.appchains.get(&index).unwrap())
            .collect()
    }

    pub fn get_num_appchains(&self) -> usize {
        self.appchains.len()
    }

    /// Returns the total staking balance.
    pub fn get_total_staked_balance(&self) -> U128 {
        U128::from(self.total_staked_balance)
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn get_minium_staking_amount(&self) -> U128 {
        U128::from(self.minium_staking_amount)
    }

    pub fn get_appchain_minium_validators(&self) -> u32 {
        self.appchain_minium_validators
    }

    pub fn get_appchain(&self, appchain_id: u32) -> Option<Appchain> {
        self.appchains.get(&appchain_id).cloned()
    }

    pub fn get_validator_set(&self, appchain_id: u32, seq_num: u32) -> Option<ValidatorSet> {
        let appchain = self
            .appchains
            .get(&appchain_id)
            .expect("Appchain not found");

        appchain.validator_set.get(&seq_num).cloned()
    }

    // Returns the appchain current validator_set index
    pub fn get_curr_validator_set_index(&self, appchain_id: u32) -> u32 {
        let appchain = self
            .appchains
            .get(&appchain_id)
            .expect("Appchain not found");
        appchain.validator_set.len() as u32 - 1
    }

    fn staking(&mut self, appchain_id: u32, id: String, amount: u128) {
        let account_id = env::signer_account_id();

        // Check amount
        assert!(
            amount >= self.minium_staking_amount,
            "Insufficient staking amount"
        );

        if !self.appchains.contains_key(&appchain_id) {
            panic!("Appchain not found");
        }

        let mut appchain = self
            .appchains
            .get(&appchain_id)
            .cloned()
            .expect("Appchain not found");
        for v in appchain.validators.iter() {
            assert!(
                v.account_id != account_id,
                "You are already staked on the appchain!"
            );
        }

        appchain.validators.push(Validator {
            account_id: account_id.clone(),
            id,
            weight: amount,
            block_height: env::block_index(),
            staked_amount: amount,
            delegations: Vec::default(),
        });

        // Update state
        self.appchains.insert(appchain_id, appchain);
        self.total_staked_balance += amount;

        // Check to update validator set
        self.update_validator_set(appchain_id);
    }

    fn staking_more(&mut self, appchain_id: u32, amount: u128) {
        let account_id = env::signer_account_id();

        // Check amount
        assert!(
            amount >= self.minium_staking_amount,
            "Insufficient staking amount"
        );

        let appchain = self
            .appchains
            .get(&appchain_id)
            .cloned()
            .expect("Appchain not found");
        appchain
            .validators
            .iter()
            .find(|v| v.account_id == account_id)
            .expect("You are not staked on the appchain");

        let mut appchain = self
            .appchains
            .get(&appchain_id)
            .cloned()
            .expect("Appchain not found");
        let mut found = false;
        for v in appchain.validators.iter_mut() {
            if v.account_id == account_id {
                v.staked_amount += amount;
                v.weight += amount;
                found = true;
            }
        }

        if !found {
            panic!("You are not staked on the appchain");
        }

        // Update state
        self.appchains.insert(appchain_id, appchain);
        self.total_staked_balance += amount;

        // Check to update validator set
        self.update_validator_set(appchain_id);
    }

    #[payable]
    pub fn unstaking(&mut self, appchain_id: u32) {
        let account_id = env::signer_account_id();
        let appchain = self
            .appchains
            .get(&appchain_id)
            .cloned()
            .expect("Appchain not found");

        let validator = appchain
            .validators
            .iter()
            .find(|v| v.account_id == account_id)
            .expect("You are not staked on the appchain");

        // Cross-contract call to transfer OCT token
        let promise_transfer = env::promise_create(
            self.token_contract_id.to_string(),
            b"ft_transfer",
            json!({
                "receiver_id": account_id,
                "amount": validator.staked_amount.to_string(),
            })
            .to_string()
            .as_bytes(),
            1,
            SINGLE_CALL_GAS,
        );

        // Check transfer token result and unstaking
        let promise_staking = env::promise_then(
            promise_transfer,
            env::current_account_id(),
            b"check_transfer_and_unstaking",
            json!({
                "appchain_id": appchain_id,
                "account_id": account_id,
                "amount": validator.staked_amount.to_string(),
            })
            .to_string()
            .as_bytes(),
            NO_DEPOSIT,
            SINGLE_CALL_GAS,
        );

        env::promise_return(promise_staking);
    }

    pub fn check_transfer_and_unstaking(
        &mut self,
        appchain_id: u32,
        account_id: AccountId,
        amount: U128,
    ) {
        let amount: u128 = amount.0;
        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut appchain = self
                    .appchains
                    .get(&appchain_id)
                    .cloned()
                    .expect("Appchain not found");

                // Remove the validator
                appchain.validators.retain(|v| v.account_id != account_id);

                // Update state
                self.appchains.insert(appchain_id, appchain);
                self.total_staked_balance -= amount;

                // Check to update validator set
                self.update_validator_set(appchain_id);
            }
            _ => panic!("Transfer token failed"),
        };
    }

    pub fn active_appchain(&mut self, appchain_id: u32) {
        let mut appchain = self
            .appchains
            .get(&appchain_id)
            .cloned()
            .expect("Appchain not found");
        let account_id = env::signer_account_id();

        // Only admin can do this
        assert!(account_id == self.owner, "You're not the relay admin");

        // Can only active a frozen appchain
        assert!(
            appchain.status == AppchainStatus::Frozen,
            "Appchain status incorrect"
        );
        // Check validators
        assert!(
            appchain.validators.len() as u32 >= self.appchain_minium_validators,
            "Insufficient number of appchain validators"
        );

        appchain.status = AppchainStatus::Active;

        // Update state
        self.appchains.insert(appchain_id, appchain);

        // Check to update validator set
        self.update_validator_set(appchain_id);
    }

    pub fn freeze_appchain(&mut self, appchain_id: u32) {
        let mut appchain = self
            .appchains
            .get(&appchain_id)
            .cloned()
            .expect("Appchain not found");

        let account_id = env::signer_account_id();

        // Only admin can do this
        assert!(account_id == self.owner, "You're not the relay admin");

        // Check status
        assert!(
            appchain.status == AppchainStatus::Active,
            "Appchain status incorrect"
        );

        appchain.status = AppchainStatus::Frozen;

        // Update state
        self.appchains.insert(appchain_id, appchain);
    }

    /*
        Update validator set, is called after the appchain validators or status updated
    */
    fn update_validator_set(&mut self, appchain_id: u32) -> bool {
        let mut appchain = self.appchains.get(&appchain_id).cloned().unwrap();

        let appchain_curr_validator_set_idx = self.get_curr_validator_set_index(appchain_id);
        let mut validator_set = appchain
            .validator_set
            .get(&appchain_curr_validator_set_idx)
            .unwrap()
            .clone();

        // Check status
        if appchain.status != AppchainStatus::Active {
            return false;
        }

        let mut changed = false;
        let validators_len = appchain.validators.len() as u32;
        if validators_len < self.appchain_minium_validators {
            appchain.status = AppchainStatus::Frozen;
            validator_set.validators = vec![];
            changed = true;
        } else {
            appchain.validators.sort_by(|a, b| b.weight.cmp(&a.weight));
        }

        // Compare sorted array
        if !changed {
            let max_index = appchain
                .validators
                .len()
                .max(validator_set.validators.len());
            let default_validator = Validator::default();
            for i in 0..max_index {
                let v = validator_set
                    .validators
                    .get(i)
                    .unwrap_or(&default_validator);
                let av = appchain.validators.get(i).unwrap_or(&default_validator);
                if av.account_id != v.account_id {
                    changed = true;
                    validator_set.validators = appchain.validators.clone();
                    break;
                }
            }
        }

        // Update state
        if changed {
            validator_set.sequence_number += 1;

            appchain
                .validator_set
                .insert(appchain_curr_validator_set_idx + 1, validator_set);
            self.appchains.insert(appchain_id, appchain);
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
