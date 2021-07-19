use crate::utils::{init, init_by_previous, register_user};
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, ExecutionResult, UserAccount, DEFAULT_GAS};
use octopus_relay::types::{
    Appchain, AppchainStatus, BridgeStatus, BridgeToken, Fact, Validator, ValidatorSet,
};
use num_format::{Locale, ToFormattedString};

pub const initial_balance_str: &str = "100000";
pub const appchain_minimum_validators: u32 = 2;
pub const minimum_staking_amount_str: &str = "100";

pub const val_id0: &str = "0xc425bbf59c7bf49e4fcc6547539d84ba8ecd2fb171f5b83cde3571d45d0c8224";
pub const val_id1: &str = "0xd447acbfe7761c0cfba8341e616275caca6401637308ee123b77082a40095331";

pub fn default_init() -> (
    UserAccount,
    UserAccount,
    UserAccount,
    UserAccount,
    UserAccount,
) {
    let (root, oct, b_token, relay, alice) = init(
        to_yocto(initial_balance_str),
        appchain_minimum_validators,
        to_yocto(minimum_staking_amount_str),
    );

    (root, oct, b_token, relay, alice)
}

pub fn default_init_by_previous() -> (
    UserAccount,
    UserAccount,
    UserAccount,
    UserAccount,
    UserAccount,
) {
    let (root, oct, b_token, relay, alice) = init_by_previous(
        to_yocto(initial_balance_str),
        appchain_minimum_validators,
        to_yocto(minimum_staking_amount_str),
    );

    (root, oct, b_token, relay, alice)
}

pub fn default_register_appchain(
    root: &UserAccount,
    oct: &UserAccount,
    relay: &UserAccount,
) -> (ExecutionResult, u128) {
    register_user(&relay);
    let transfer_amount = to_yocto("200");
    let outcome = root.call(
        oct.account_id(),
        "ft_transfer_call",
        &json!({
            "receiver_id": relay.valid_account_id(),
            "amount": transfer_amount.to_string(),
            "msg": "register_appchain,testchain,website_url_string,github_address_string,github_release_string,commit_id,email_string",
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        1,
    );
    println!("Gas burnt of function 'ft_transfer_call': {}", outcome.gas_burnt().to_formatted_string(&Locale::en));
    outcome.assert_success();
    (outcome, transfer_amount)
}

pub fn default_pass_appchain(
    root: &UserAccount,
    oct: &UserAccount,
    relay: &UserAccount,
) -> (ExecutionResult, u128) {
    let (_, transfer_amount) = default_register_appchain(&root, &oct, &relay);
    let outcome = relay.call(
        relay.account_id(),
        "pass_appchain",
        &json!({
            "appchain_id": "testchain",
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    println!("Gas burnt of function 'pass_appchain': {}", outcome.gas_burnt().to_formatted_string(&Locale::en));
    outcome.assert_success();
    (outcome, transfer_amount)
}

pub fn default_appchain_go_staging(
    root: &UserAccount,
    oct: &UserAccount,
    relay: &UserAccount,
) -> (ExecutionResult, u128) {
    let (_, transfer_amount) = default_pass_appchain(&root, &oct, &relay);
    let outcome = relay.call(
        relay.account_id(),
        "appchain_go_staging",
        &json!({
            "appchain_id": "testchain",
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    println!("Gas burnt of function 'appchain_go_staging': {}", outcome.gas_burnt().to_formatted_string(&Locale::en));
    outcome.assert_success();
    (outcome, transfer_amount)
}

pub fn default_stake(
    user: &UserAccount,
    oct: &UserAccount,
    relay: &UserAccount,
    validator_id: &str,
) -> (ExecutionResult, u128) {
    register_user(&relay);
    let transfer_amount = to_yocto("200");
    let mut msg = "stake,testchain,".to_owned();
    msg.push_str(validator_id.to_string().as_ref());

    let outcome = user.call(
        oct.account_id(),
        "ft_transfer_call",
        &json!({
            "receiver_id": relay.valid_account_id(),
            "amount": transfer_amount.to_string(),
            "msg": msg,
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        1,
    );
    println!("Gas burnt of function 'ft_transfer_call': {}", outcome.gas_burnt().to_formatted_string(&Locale::en));
    outcome.assert_success();
    (outcome, transfer_amount)
}

pub fn default_update_appchain(root: &UserAccount, relay: &UserAccount) -> ExecutionResult {
    let chain_spec_url: &str = "https://xxxxxx.xom";
    let chain_spec_hash: &str = "chain_spec_hash";
    let chain_spec_raw_url: &str = "https://xxxxxx_raw.xom";
    let chain_spec_raw_hash: &str = "chain_spec_raw_hash";
    let outcome = root.call(
        relay.account_id(),
        "update_appchain",
        &json!({
            "appchain_id": "testchain",
            "website_url": String::from("website_url_string"),
            "github_address": String::from("github_address_url"),
            "github_release": String::from("github_release"),
            "commit_id": String::from("commit_id"),
            "email": String::from("email_string1"),
            "chain_spec_url": chain_spec_url,
            "chain_spec_hash": chain_spec_hash,
            "chain_spec_raw_url": chain_spec_raw_url,
            "chain_spec_raw_hash": chain_spec_raw_hash
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    println!("Gas burnt of function 'update_appchain': {}", outcome.gas_burnt().to_formatted_string(&Locale::en));
    outcome.assert_success();
    outcome
}

pub fn to_decimals_amount(amount: u128, decimals: u32) -> u128 {
    let bt_decimals_base = (10 as u128).pow(decimals);
    amount * bt_decimals_base
}

pub fn default_activate_appchain(relay: &UserAccount) -> ExecutionResult {
    let outcome = relay.call(
        relay.account_id(),
        "activate_appchain",
        &json!({
            "appchain_id": "testchain",
            "boot_nodes": "[\"/ip4/13.230.75.107/tcp/30333/p2p/12D3KooWAxYKgdmTczLioD1jkzMyaDuV2Q5VHBsJxPr5zEmHr8nY\", \"/ip4/13.113.159.178/tcp/30333/p2p/12D3KooWSmLVShww4w9PVW17cCAS5C1JnXBU4NbY7FcGGjMyUGiq\",   \"/ip4/35.74.91.128/tcp/30333/p2p/12D3KooWT2umkS7F8GzUTLrfUzVBJPKn6YwCcuv6LBFQ27UPoo2Y\", \"/ip4/35.73.129.159/tcp/30333/p2p/12D3KooWHNf9JxUZKHoF7rrsmorv86gonXSb2ZU44CbMsnBNFSAJ\", ]",
            "rpc_endpoint": "wss://barnacle.rpc.testnet.oct.network:9944",
            "chain_spec_url": "chain_spec_url",
            "chain_spec_hash": "chain_spec_hash",
            "chain_spec_raw_url": "chain_spec_raw_url",
            "chain_spec_raw_hash": "chain_spec_raw_hash",
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    println!("Gas burnt of function 'activate_appchain': {}", outcome.gas_burnt().to_formatted_string(&Locale::en));
    outcome.assert_success();
    outcome
}

pub fn default_register_bridge_token(
    root: &UserAccount,
    oct: &UserAccount,
    b_token: &UserAccount,
    relay: &UserAccount,
    alice: &UserAccount,
) -> ExecutionResult {
    default_appchain_go_staging(&root, &oct, &relay);
    default_stake(&root, &oct, &relay, val_id0);
    default_stake(&alice, &oct, &relay, val_id1);
    default_activate_appchain(&relay);

    let outcome = relay.call(
        relay.account_id(),
        "register_bridge_token",
        &json!({
            "token_id": b_token.valid_account_id(),
            "symbol": "BTK",
            "price": U128::from(1000000),
            "decimals": 12,
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    println!("Gas burnt of function 'register_bridge_token': {}", outcome.gas_burnt().to_formatted_string(&Locale::en));
    outcome.assert_success();
    outcome
}

pub fn default_set_bridge_permitted(
    b_token: &UserAccount,
    relay: &UserAccount,
    permitted: bool,
) -> ExecutionResult {
    let outcome = relay.call(
        relay.account_id(),
        "set_bridge_permitted",
        &json!({
            "token_id": b_token.valid_account_id(),
            "appchain_id": "testchain",
            "permitted": permitted
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    println!("Gas burnt of function 'set_bridge_permitted': {}", outcome.gas_burnt().to_formatted_string(&Locale::en));
    outcome.assert_success();
    outcome
}

pub fn get_facts(root: &UserAccount, relay: &UserAccount) -> Vec<Fact> {
    let facts: Vec<Fact> = root
        .view(
            relay.account_id(),
            "get_facts",
            &json!({
                "appchain_id": "testchain",
                "start": 0,
                "limit": 100
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();
    // ignore the fact of initial validator_set
    facts[1..facts.len()].to_vec()
}

pub fn lock_token(
    b_token: &UserAccount,
    root: &UserAccount,
    relay: &UserAccount,
    transfer_amount_str: u128,
) -> Vec<Fact> {
    register_user(&relay);
    let outcome = root.call(
        b_token.account_id(),
        "ft_transfer_call",
        &json!({
            "receiver_id": relay.valid_account_id(),
            "amount": U128::from(to_decimals_amount(transfer_amount_str, 12)),
            "msg": "lock_token,testchain,receiver",
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        1,
    );
    println!("Gas burnt of function 'ft_transfer_call': {}", outcome.gas_burnt().to_formatted_string(&Locale::en));
    outcome.assert_success();

    get_facts(&root, &relay)
}
