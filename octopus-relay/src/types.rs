use crate::*;

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
    pub id: DelegatorId,
    pub account_id: AccountId,
    pub amount: U128,
    pub block_height: BlockHeight,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Validator {
    pub id: String,
    pub account_id: AccountId,
    pub weight: u32,
    pub staked_amount: U128,
    pub block_height: BlockHeight,
    pub delegations: Vec<Delegation>,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LiteValidator {
    pub id: String,
    pub account_id: AccountId,
    pub weight: u32,
    pub block_height: BlockHeight,
    pub delegations: Vec<Delegation>,
}

impl Default for LiteValidator {
    fn default() -> Self {
        Self {
            id: String::from(""),
            account_id: AccountId::from(""),
            weight: 0 as u32,
            block_height: 0,
            delegations: vec![],
        }
    }
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorSet {
    pub seq_num: u32,
    pub validators: Vec<LiteValidator>,
}

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Appchain {
    pub id: u32,
    pub founder_id: AccountId,
    pub appchain_name: String,
    pub website_url: String,
    pub github_address: String,
    pub chain_spec_url: String,
    pub chain_spec_hash: String,
    pub boot_nodes: String,
    pub rpc_endpoint: String,
    pub bond_tokens: U128,
    pub validators: Vec<Validator>,
    pub status: AppchainStatus,
    pub block_height: BlockHeight,
}
