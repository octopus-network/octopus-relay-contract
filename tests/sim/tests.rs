use crate::default::{
    default_activate_appchain, default_init, default_list_appchain, default_register_appchain,
    default_register_bridge_token, default_set_bridge_permitted, default_stake,
    default_update_appchain, lock_token, to_decimals_amount,
};
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, ExecutionResult, UserAccount, DEFAULT_GAS};
use octopus_relay::types::{
    Appchain, AppchainStatus, BridgeStatus, BridgeToken, Validator, ValidatorSet,
};

const initial_balance_str: &str = "100000";
const appchain_minium_validators: u32 = 2;
const minium_staking_amount_str: &str = "100";

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
            "testchain", transfer_amount
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
                "appchain_id": "testchain"
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();

    let appchain = appchain_option.unwrap();
    assert_eq!(appchain.id, "testchain");
    assert_eq!(appchain.founder_id, root.account_id());
    assert_eq!(appchain.chain_spec_url, String::from(""));
    assert_eq!(appchain.chain_spec_hash, String::from(""));
    assert_eq!(appchain.chain_spec_raw_url, String::from(""));
    assert_eq!(appchain.chain_spec_raw_hash, String::from(""));
    assert_eq!(appchain.bond_tokens, U128::from(transfer_amount));
    assert_eq!(appchain.validators.len(), 0);
    assert_eq!(appchain.status, AppchainStatus::Auditing);
}

#[test]
fn simulate_list_appchain() {
    let (root, oct, _, relay, _) = default_init();
    let (_, transfer_amount) = default_list_appchain(&root, &oct, &relay);

    let num_appchains: usize = root
        .view(relay.account_id(), "get_num_appchains", b"")
        .unwrap_json();

    assert_eq!(num_appchains, 1);

    let appchain_option: Option<Appchain> = root
        .view(
            relay.account_id(),
            "get_appchain",
            &json!({
                "appchain_id": "testchain"
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();

    let appchain = appchain_option.unwrap();
    assert_eq!(appchain.id, "testchain");
    assert_eq!(appchain.founder_id, root.account_id());
    assert_eq!(appchain.chain_spec_url, String::from("chain_spec_url"));
    assert_eq!(appchain.chain_spec_hash, String::from("chain_spec_hash"));
    assert_eq!(
        appchain.chain_spec_raw_url,
        String::from("chain_spec_raw_url")
    );
    assert_eq!(
        appchain.chain_spec_raw_hash,
        String::from("chain_spec_raw_hash")
    );
    assert_eq!(appchain.bond_tokens, U128::from(transfer_amount));
    assert_eq!(appchain.validators.len(), 0);
    assert_eq!(appchain.status, AppchainStatus::Frozen);
}

#[test]
fn simulate_update_appchain() {
    let (root, oct, _, relay, _) = default_init();
    default_list_appchain(&root, &oct, &relay);
    default_update_appchain(&root, &relay);
}

#[test]
fn simulate_stake() {
    let (root, oct, _, relay, _) = default_init();
    default_list_appchain(&root, &oct, &relay);
    let (outcome, transfer_amount) = default_stake(&root, &oct, &relay);
    outcome.assert_success();
    let validators: Vec<Validator> = root
        .view(
            relay.account_id(),
            "get_validators",
            &json!({
                "appchain_id": "testchain"
            })
            .to_string()
            .into_bytes(),
        )
        .unwrap_json();
    let validator = validators.get(0).unwrap();
    assert_eq!(validator.id, root.valid_account_id().to_string().as_ref());
    assert_eq!(validator.account_id, "root");
    assert_eq!(validator.weight, 200);
    assert_eq!(validator.staked_amount, U128::from(transfer_amount));
}

#[test]
fn simulate_activate_appchain() {
    let (root, oct, _, relay, alice) = default_init();
    default_list_appchain(&root, &oct, &relay);
    default_stake(&root, &oct, &relay);
    default_stake(&alice, &oct, &relay);
    default_update_appchain(&root, &relay);
    default_activate_appchain(&relay);

    let appchain_option: Option<Appchain> = root
        .view(
            relay.account_id(),
            "get_appchain",
            &json!({
                "appchain_id": "testchain"
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
            "get_bridge_allowed_amount",
            &json!({
                "appchain_id": "testchain",
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

    let locked_events0 = lock_token(&b_token, &root, &relay, 100);
    println!("locked_events0{:#?}", locked_events0);
    let locked_events1 = lock_token(&b_token, &root, &relay, 160);

    assert_eq!(locked_events0.len(), 1);
    assert_eq!(locked_events1.len(), 2);

    let locked0 = &locked_events0[0];
    let locked1 = &locked_events1[1];
    assert_eq!(locked0.seq_num, 0);
    assert_eq!(locked1.seq_num, 1);
    assert_eq!(locked0.amount, U128::from(to_decimals_amount(100, 12)));
    assert_eq!(locked1.amount, U128::from(to_decimals_amount(160, 12)));
}
