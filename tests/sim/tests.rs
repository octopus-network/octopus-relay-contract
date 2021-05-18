use crate::utils::{init, register_user};
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, ExecutionResult, UserAccount, DEFAULT_GAS};
use octopus_relay::types::{Appchain, AppchainStatus, Validator, ValidatorSet};

const initial_balance_str: &str = "100000";
const appchain_minium_validators: u32 = 3;
const minium_staking_amount_str: &str = "100";

pub fn default_init() -> (UserAccount, UserAccount, UserAccount, UserAccount) {
    let (root, oct, relay, alice) = init(
        to_yocto(initial_balance_str),
        appchain_minium_validators,
        to_yocto(minium_staking_amount_str),
    );

    (root, oct, relay, alice)
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
    (outcome, transfer_amount)
}

pub fn default_staking(
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
            "msg": "staking,0,validator_id",
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        1,
    );
    (outcome, transfer_amount)
}

#[test]
fn simulate_total_supply() {
    let (_, oct, _, _) = default_init();

    let total_supply: U128 = oct
        .view(oct.account_id(), "ft_total_supply", b"")
        .unwrap_json();

    assert_eq!(to_yocto(initial_balance_str), total_supply.0);
}

#[test]
fn simulate_register_appchain() {
    let (root, oct, relay, _) = default_init();
    let (outcome, transfer_amount) = default_register_appchain(&root, &oct, &relay);
    outcome.assert_success();

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
    println!("{:#?}", appchain_option);
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
    let (root, oct, relay, _) = default_init();
    default_register_appchain(&root, &oct, &relay);

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
}

#[test]
fn simulate_staking() {
    let (root, oct, relay, _) = default_init();
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
