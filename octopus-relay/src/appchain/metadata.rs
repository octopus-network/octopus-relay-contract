use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, Balance, BlockHeight};

use crate::AppchainId;

/// Metadata of an appchain of Octopus Network
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainMetadata {
    /// Appchain id
    pub id: AppchainId,
    /// AccountId of founder
    pub founder_id: AccountId,
    /// Website url of the appchain
    pub website_url: String,
    /// Github address (url) of the appchain
    pub github_address: String,
    /// Github release version of the appchain
    pub github_release: String,
    /// Github commit it of the appchain
    pub commit_id: String,
    /// Contact email of the appchain
    pub email: String,
    ///
    pub chain_spec_url: String,
    ///
    pub chain_spec_hash: String,
    ///
    pub chain_spec_raw_url: String,
    ///
    pub chain_spec_raw_hash: String,
    ///
    pub boot_nodes: String,
    /// Endpoint of RPC service provided by Octopus Network
    pub rpc_endpoint: String,
    /// The balance of OCT token received at appchain registration
    pub bond_tokens: Balance,
    /// Block height when the founder registered the appchain
    pub block_height: BlockHeight,
    ///
    pub subql_url: String,
}

impl AppchainMetadata {
    /// Return a new instance of AppchainMetadata with the given data
    pub fn new(
        appchain_id: AppchainId,
        founder_id: String,
        website_url: String,
        github_address: String,
        github_release: String,
        commit_id: String,
        email: String,
        bond_tokens: u128,
    ) -> Self {
        Self {
            id: appchain_id,
            founder_id,
            website_url,
            github_address,
            github_release,
            commit_id,
            email,
            chain_spec_url: String::new(),
            chain_spec_hash: String::new(),
            chain_spec_raw_url: String::new(),
            chain_spec_raw_hash: String::new(),
            bond_tokens,
            boot_nodes: String::new(),
            rpc_endpoint: String::new(),
            block_height: env::block_index(),
            subql_url: String::new(),
        }
    }
    /// Update basic info of metadata content of current appchain
    pub fn update_basic_info(
        &mut self,
        website_url: String,
        github_address: String,
        github_release: String,
        commit_id: String,
        email: String,
        rpc_endpoint: String,
    ) {
        self.website_url.clear();
        self.website_url.push_str(website_url.as_str());
        self.github_address.clear();
        self.github_address.push_str(github_address.as_str());
        self.github_release.clear();
        self.github_release.push_str(github_release.as_str());
        self.commit_id.clear();
        self.commit_id.push_str(commit_id.as_str());
        self.email.clear();
        self.email.push_str(email.as_str());
        self.rpc_endpoint.clear();
        self.rpc_endpoint.push_str(rpc_endpoint.as_str());
    }
    /// Update booting info of metadata content of current appchain
    pub fn update_booting_info(
        &mut self,
        boot_nodes: String,
        rpc_endpoint: String,
        chain_spec_url: String,
        chain_spec_hash: String,
        chain_spec_raw_url: String,
        chain_spec_raw_hash: String,
    ) {
        self.boot_nodes.clear();
        self.boot_nodes.push_str(boot_nodes.as_str());
        self.rpc_endpoint.clear();
        self.rpc_endpoint.push_str(rpc_endpoint.as_str());
        self.chain_spec_url.clear();
        self.chain_spec_url.push_str(chain_spec_url.as_str());
        self.chain_spec_hash.clear();
        self.chain_spec_hash.push_str(chain_spec_hash.as_str());
        self.chain_spec_raw_url.clear();
        self.chain_spec_raw_url
            .push_str(chain_spec_raw_url.as_str());
        self.chain_spec_raw_hash.clear();
        self.chain_spec_raw_hash
            .push_str(chain_spec_raw_hash.as_str());
    }
    /// Update subql info of metadata of current appchain
    pub fn update_subql(&mut self, subql: String) {
        self.subql_url.clear();
        self.subql_url.push_str(subql.as_str());
    }
}
