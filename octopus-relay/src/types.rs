use crate::*;
use codec::{Decode, Encode, Input};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Vote {
    Yes,
    No,
}

pub type HexAddress = [u8; 32];

/// Describes the status of appchains
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainStatus {
    Auditing,
    Voting,
    Staging,
    Booting,
}

impl Default for AppchainStatus {
    fn default() -> Self {
        AppchainStatus::Auditing
    }
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Delegator {
    pub id: DelegatorId,
    pub account_id: AccountId,
    pub amount: U128,
    pub block_height: BlockHeight,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Validator {
    pub id: ValidatorId,
    pub account_id: AccountId,
    pub staked_amount: U128,
    pub block_height: BlockHeight,
    pub delegators: Vec<Delegator>,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LiteValidator {
    pub id: ValidatorId,
    pub account_id: AccountId,
    pub weight: U128,
    pub block_height: BlockHeight,
    pub delegators: Vec<Delegator>,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorSet {
    pub seq_num: SeqNum,
    pub set_id: u32,
    pub validators: Vec<LiteValidator>,
}

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Appchain {
    pub id: String,
    pub founder_id: AccountId,
    pub website_url: String,
    pub github_address: String,
    pub github_release: String,
    pub commit_id: String,
    pub email: String,
    pub chain_spec_url: String,
    pub chain_spec_hash: String,
    pub chain_spec_raw_url: String,
    pub chain_spec_raw_hash: String,
    pub boot_nodes: String,
    pub rpc_endpoint: String,
    pub bond_tokens: U128,
    pub validators: Vec<Validator>,
    pub validators_timestamp: u64,
    pub status: AppchainStatus,
    pub block_height: BlockHeight,
    pub staked_balance: U128,
    pub subql_url: String,
    pub fact_sets_len: SeqNum,
    pub validator_sets_len: SeqNum,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum BridgeStatus {
    Paused,
    Active,
    Closed,
}

impl Default for BridgeStatus {
    fn default() -> Self {
        BridgeStatus::Active
    }
}

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct BridgeToken {
    pub token_id: AccountId,
    pub symbol: String,
    pub status: BridgeStatus,
    pub price: U128,
    pub decimals: u32,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Locked {
    pub seq_num: SeqNum,
    pub token_id: AccountId,
    pub sender_id: AccountId,
    pub receiver: String,
    pub amount: U128,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Burned {
    pub seq_num: SeqNum,
    pub sender_id: AccountId,
    pub receiver: String,
    pub amount: U128,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Fact {
    UpdateValidatorSet(ValidatorSet),
    LockToken(Locked),
    BurnNativeToken(Burned),
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    pub total: U128,
    pub available: U128,
}

#[derive(
    Encode, Decode, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug,
)]
#[serde(crate = "near_sdk::serde")]
pub enum PayloadType {
    Lock,
    BurnAsset,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct BurnAssetPayload {
    pub token_id: AccountId,
    pub sender: String,
    pub receiver_id: ValidAccountId,
    pub amount: U128,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LockPayload {
    pub sender: String,
    pub receiver_id: ValidAccountId,
    pub amount: U128,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum MessagePayload {
    BurnAsset(BurnAssetPayload),
    Lock(LockPayload),
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Message {
    pub nonce: u64,
    pub payload: MessagePayload,
}
