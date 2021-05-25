use crate::utils::{init, register_user};
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, ExecutionResult, UserAccount, DEFAULT_GAS};
use octopus_relay::types::{
    Appchain, AppchainStatus, BridgeStatus, BridgeToken, Validator, ValidatorSet,
};

const initial_balance_str: &str = "100000";
const appchain_minium_validators: u32 = 2;
const minium_staking_amount_str: &str = "100";

pub fn default_init() -> (
    UserAccount,
    UserAccount,
    UserAccount,
    UserAccount,
    UserAccount,
) {
    let (root, oct, b_token, relay, alice) = init(
        to_yocto(initial_balance_str),
        appchain_minium_validators,
        to_yocto(minium_staking_amount_str),
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
            "msg": "register_appchain,testchain,website_url_string,github_address_string",
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        1,
    );
    outcome.assert_success();
    (outcome, transfer_amount)
}

pub fn default_staking(
    user: &UserAccount,
    oct: &UserAccount,
    relay: &UserAccount,
) -> (ExecutionResult, u128) {
    register_user(&relay);
    let transfer_amount = to_yocto("200");
    let outcome = user.call(
        oct.account_id(),
        "ft_transfer_call",
        &json!({
            "receiver_id": relay.valid_account_id(),
            "amount": transfer_amount.to_string(),
            "msg": "staking,0,validator_id",
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        1,
    );
    outcome.assert_success();
    (outcome, transfer_amount)
}

pub fn default_update_appchain(root: &UserAccount, relay: &UserAccount) -> ExecutionResult {
    let chain_spec_url: &str = "https://xxxxxx.xom";
    let chain_spec_hash: &str = "chain_spec_hash";
    let outcome = root.call(
        relay.account_id(),
        "update_appchain",
        &json!({
            "appchain_id": 0,
            "website_url": String::from("website_url_string"),
            "github_address": String::from("github_address_url"),
            "chain_spec_url": chain_spec_url,
            "chain_spec_hash": chain_spec_hash
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        0,
    );

    outcome.assert_success();
    outcome
}

pub fn default_activate_appchain(relay: &UserAccount) -> ExecutionResult {
    let outcome = relay.call(
        relay.account_id(),
        "activate_appchain",
        &json!({
            "appchain_id": 0,
            "boot_nodes": "[\"/ip4/13.230.75.107/tcp/30333/p2p/12D3KooWAxYKgdmTczLioD1jkzMyaDuV2Q5VHBsJxPr5zEmHr8nY\", \"/ip4/13.113.159.178/tcp/30333/p2p/12D3KooWSmLVShww4w9PVW17cCAS5C1JnXBU4NbY7FcGGjMyUGiq\",   \"/ip4/35.74.91.128/tcp/30333/p2p/12D3KooWT2umkS7F8GzUTLrfUzVBJPKn6YwCcuv6LBFQ27UPoo2Y\", \"/ip4/35.73.129.159/tcp/30333/p2p/12D3KooWHNf9JxUZKHoF7rrsmorv86gonXSb2ZU44CbMsnBNFSAJ\", ]",
            "rpc_endpoint": "wss://barnacle.rpc.testnet.oct.network:9944",
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        0,
    );
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
    default_register_appchain(&root, &oct, &relay);
    default_staking(&root, &oct, &relay);
    default_staking(&alice, &oct, &relay);
    default_update_appchain(&root, &relay);
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
            "appchain_id": 0,
            "permitted": permitted
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    outcome.assert_success();
    outcome
}

pub fn lock_token(
    root: &UserAccount,
    b_token: &UserAccount,
    relay: &UserAccount,
    actual_amount: u128,
) -> (ExecutionResult, u128) {
    let outcome = relay.call(
        relay.account_id(),
        "after_token_lock",
        &json!({
            "token_id": b_token.valid_account_id(),
            "appchain_id": 0,
            "amount": U128::from(actual_amount * (10 as u128).pow(12))
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    outcome.assert_success();
    let bridge_allowed: U128 = root
        .view(
            relay.account_id(),
            "get_bridge_allowed",
            &json!({
                "appchain_id": 0,
                "token_id": b_token.valid_account_id()
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();
    (outcome, bridge_allowed.into())
}

#[test]
fn simulate_total_supply() {
    let (_, oct, _, _, _) = default_init();

    let total_supply: U128 = oct
        .view(oct.account_id(), "ft_total_supply", b"")
        .unwrap_json();

    assert_eq!(to_yocto(initial_balance_str), total_supply.0);
}

#[test]
fn simulate_register_appchain() {
    let (root, oct, _, relay, _) = default_init();
    let (outcome, transfer_amount) = default_register_appchain(&root, &oct, &relay);

    let results = outcome.promise_results();
    let logs = results[2].as_ref().unwrap().logs();
    println!(
        "{:#?}",
        outcome.promise_results()[2].as_ref().unwrap().logs()
    );

    assert_eq!(
        logs[1],
        format!(
            "Appchain added, appchain_id is {}, bund_tokens is {}.",
            0, transfer_amount
        )
    );

    let num_appchains: usize = root
        .view(relay.account_id(), "get_num_appchains", b"")
        .unwrap_json();

    assert_eq!(num_appchains, 1);

    let appchain_option: Option<Appchain> = root
        .view(
            relay.account_id(),
            "get_appchain",
            &json!({
                "appchain_id": 0
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();

    let appchain = appchain_option.unwrap();
    assert_eq!(appchain.id, 0);
    assert_eq!(appchain.founder_id, root.account_id());
    assert_eq!(appchain.appchain_name, String::from("testchain"));
    assert_eq!(appchain.chain_spec_url, String::from(""));
    assert_eq!(appchain.chain_spec_hash, String::from(""));
    assert_eq!(appchain.bond_tokens, U128::from(transfer_amount));
    assert_eq!(appchain.validators.len(), 0);
    assert_eq!(appchain.status, AppchainStatus::InProgress);
}

#[test]
fn simulate_update_appchain() {
    let (root, oct, _, relay, _) = default_init();
    default_register_appchain(&root, &oct, &relay);
    default_update_appchain(&root, &relay);
}

#[test]
fn simulate_staking() {
    let (root, oct, _, relay, _) = default_init();
    default_register_appchain(&root, &oct, &relay);
    let (outcome, transfer_amount) = default_staking(&root, &oct, &relay);
    outcome.assert_success();
    let validators: Vec<Validator> = root
        .view(
            relay.account_id(),
            "get_validators",
            &json!({
                "appchain_id": 0
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();
    let validator = validators.get(0).unwrap();
    assert_eq!(validator.id, "validator_id");
    assert_eq!(validator.account_id, "root");
    assert_eq!(validator.weight, 200);
    assert_eq!(validator.staked_amount, U128::from(transfer_amount));
}

#[test]
fn simulate_activate_appchain() {
    let (root, oct, _, relay, alice) = default_init();
    default_register_appchain(&root, &oct, &relay);
    default_staking(&root, &oct, &relay);
    default_staking(&alice, &oct, &relay);
    default_update_appchain(&root, &relay);
    default_activate_appchain(&relay);

    let appchain_option: Option<Appchain> = root
        .view(
            relay.account_id(),
            "get_appchain",
            &json!({
                "appchain_id": 0
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();

    assert_eq!(appchain_option.unwrap().status, AppchainStatus::Active);
}

#[test]
fn simulate_register_bridge_token() {
    let (root, oct, b_token, relay, alice) = default_init();
    default_register_bridge_token(&root, &oct, &b_token, &relay, &alice);
    let bridge_token_option: Option<BridgeToken> = root
        .view(
            relay.account_id(),
            "get_bridge_token",
            &json!({
                "token_id": b_token.valid_account_id()
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();

    let bridge_token = bridge_token_option.unwrap();
    assert_eq!(bridge_token.token_id, "b_token");
    assert_eq!(bridge_token.symbol, "BTK");
    assert_eq!(bridge_token.status, BridgeStatus::Active);
    assert_eq!(bridge_token.price, U128::from(1000000));
    assert_eq!(bridge_token.decimals, 12);
}

#[test]
fn simulate_set_bridge_permitted() {
    let (root, oct, b_token, relay, alice) = default_init();
    default_register_bridge_token(&root, &oct, &b_token, &relay, &alice);
    default_set_bridge_permitted(&b_token, &relay, true);

    let bridge_allowed: U128 = root
        .view(
            relay.account_id(),
            "get_bridge_allowed",
            &json!({
                "appchain_id": 0,
                "token_id": b_token.valid_account_id()
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();
    assert_eq!(
        bridge_allowed,
        U128::from(2666400 * (10 as u128).pow(12) / 10000)
    );
}

#[test]
fn simulate_lock_token() {
    let (root, oct, b_token, relay, alice) = default_init();
    default_register_bridge_token(&root, &oct, &b_token, &relay, &alice);
    default_set_bridge_permitted(&b_token, &relay, true);

    let (_, bridge_allowed) = lock_token(&root, &b_token, &relay, 120);
    assert_eq!(
        bridge_allowed,
        (2666400 - 120 * 10000) * (10 as u128).pow(12) / 10000
    );

    let (_, bridge_allowed) = lock_token(&root, &b_token, &relay, 130);
    assert_eq!(
        bridge_allowed,
        (2666400 - 250 * 10000) * (10 as u128).pow(12) / 10000
    );
}
