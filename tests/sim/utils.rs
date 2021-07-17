use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};

use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk_sim::{
    deploy, init_simulator, lazy_static_include, to_yocto, ContractAccount, UserAccount,
    DEFAULT_GAS, STORAGE_AMOUNT,
};

use num_format::{Locale, ToFormattedString};

// Load in contract bytes at runtime
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    OCT_WASM_BYTES => "res/oct_token.wasm",
    RELAY_WASM_BYTES => "res/octopus_relay.wasm",
    PREVIOUS_RELAY_WASM_BYTES => "res/previous_octopus_relay.wasm",
}

const OCT_ID: &str = "oct_token";
const B_TOKEN_ID: &str = "b_token";
const RELAY_ID: &str = "octopus_relay";

// Register the given `user` with oct_token
pub fn register_user(user: &near_sdk_sim::UserAccount) {
    user.call(
        OCT_ID.to_string(),
        "storage_deposit",
        &json!({
            "account_id": user.valid_account_id()
        })
        .to_string()
        .into_bytes(),
        near_sdk_sim::DEFAULT_GAS / 2,
        near_sdk::env::storage_byte_cost() * 125, // attached deposit
    )
    .assert_success();
    user.call(
        B_TOKEN_ID.to_string(),
        "storage_deposit",
        &json!({
            "account_id": user.valid_account_id()
        })
        .to_string()
        .into_bytes(),
        near_sdk_sim::DEFAULT_GAS / 2,
        near_sdk::env::storage_byte_cost() * 125, // attached deposit
    )
    .assert_success();
}

pub fn init(
    initial_balance: u128,
    appchain_minimum_validators: u32,
    minimum_staking_amount: u128,
) -> (
    UserAccount,
    UserAccount,
    UserAccount,
    UserAccount,
    UserAccount,
) {
    let root = init_simulator(None);

    let oct = root.deploy(&OCT_WASM_BYTES, OCT_ID.into(), 10 * STORAGE_AMOUNT);
    let b_token = root.deploy(&OCT_WASM_BYTES, B_TOKEN_ID.into(), 10 * STORAGE_AMOUNT);
    let relay = root.deploy(&RELAY_WASM_BYTES, RELAY_ID.into(), 10 * STORAGE_AMOUNT);

    oct.call(
        OCT_ID.into(),
        "new",
        &json!({
            "owner_id": root.valid_account_id(),
            "total_supply": U128::from(initial_balance),
            "metadata": FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "OCTToken".to_string(),
                symbol: "OCT".to_string(),
                icon: None,
                reference: None,
                reference_hash: None,
                decimals: 24,
            }
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        0, // attached deposit
    )
    .assert_success();

    b_token
        .call(
            B_TOKEN_ID.into(),
            "new",
            &json!({
                "owner_id": root.valid_account_id(),
                "total_supply": U128::from(initial_balance),
                "metadata": FungibleTokenMetadata {
                    spec: FT_METADATA_SPEC.to_string(),
                    name: "BridgeToken".to_string(),
                    symbol: "BTK".to_string(),
                    icon: None,
                    reference: None,
                    reference_hash: None,
                    decimals: 12,
                }
            })
            .to_string()
            .into_bytes(),
            DEFAULT_GAS / 2,
            0, // attached deposit
        )
        .assert_success();

    relay
        .call(
            RELAY_ID.into(),
            "new",
            &json!({
                "token_contract_id": oct.valid_account_id(),
                "appchain_minimum_validators": appchain_minimum_validators,
                "minimum_staking_amount": U128::from(minimum_staking_amount),
                "bridge_limit_ratio": 3333,
                "oct_token_price": U128::from(2000000)
            })
            .to_string()
            .into_bytes(),
            DEFAULT_GAS / 2,
            0, // attached deposit
        )
        .assert_success();

    let alice = root.create_user("alice".to_string(), to_yocto("100"));
    register_user(&alice);

    root.call(
        OCT_ID.into(),
        "ft_transfer",
        &json!({
            "receiver_id": alice.valid_account_id(),
            "amount": U128::from(initial_balance / 10),
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        1, // attached deposit
    )
    .assert_success();

    (root, oct, b_token, relay, alice)
}

pub fn init_by_previous(
    initial_balance: u128,
    appchain_minimum_validators: u32,
    minimum_staking_amount: u128,
) -> (
    UserAccount,
    UserAccount,
    UserAccount,
    UserAccount,
    UserAccount,
) {
    let root = init_simulator(None);

    let oct = root.deploy(&OCT_WASM_BYTES, OCT_ID.into(), 10 * STORAGE_AMOUNT);
    let b_token = root.deploy(&OCT_WASM_BYTES, B_TOKEN_ID.into(), 10 * STORAGE_AMOUNT);
    let relay = root.deploy(
        &PREVIOUS_RELAY_WASM_BYTES,
        RELAY_ID.into(),
        10 * STORAGE_AMOUNT,
    );

    let mut result = oct.call(
        OCT_ID.into(),
        "new",
        &json!({
            "owner_id": root.valid_account_id(),
            "total_supply": U128::from(initial_balance),
            "metadata": FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "OCTToken".to_string(),
                symbol: "OCT".to_string(),
                icon: None,
                reference: None,
                reference_hash: None,
                decimals: 24,
            }
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        0, // attached deposit
    );
    println!(
        "Gas burnt of function 'new' of OCT token contract: {}",
        result.gas_burnt().to_formatted_string(&Locale::en)
    );
    result.assert_success();

    result = b_token.call(
        B_TOKEN_ID.into(),
        "new",
        &json!({
            "owner_id": root.valid_account_id(),
            "total_supply": U128::from(initial_balance),
            "metadata": FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "BridgeToken".to_string(),
                symbol: "BTK".to_string(),
                icon: None,
                reference: None,
                reference_hash: None,
                decimals: 12,
            }
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        0, // attached deposit
    );
    println!(
        "Gas burnt of function 'new' of B token contract: {}",
        result.gas_burnt().to_formatted_string(&Locale::en)
    );
    result.assert_success();

    result = relay.call(
        RELAY_ID.into(),
        "new",
        &json!({
            "token_contract_id": oct.valid_account_id(),
            "appchain_minimum_validators": appchain_minimum_validators,
            "minimum_staking_amount": U128::from(minimum_staking_amount),
            "bridge_limit_ratio": 3333,
            "oct_token_price": U128::from(2000000)
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        0, // attached deposit
    );
    println!(
        "Gas burnt of function 'new' of relay contract: {}",
        result.gas_burnt().to_formatted_string(&Locale::en)
    );
    result.assert_success();

    let alice = root.create_user("alice".to_string(), to_yocto("100"));
    register_user(&alice);

    result = root.call(
        OCT_ID.into(),
        "ft_transfer",
        &json!({
            "receiver_id": alice.valid_account_id(),
            "amount": U128::from(initial_balance / 10),
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        1, // attached deposit
    );
    println!(
        "Gas burnt of function 'ft_transfer' of user account 'root': {}",
        result.gas_burnt().to_formatted_string(&Locale::en)
    );
    result.assert_success();

    (root, oct, b_token, relay, alice)
}

pub fn upgrade_contract_code_and_perform_migration(relay: &near_sdk_sim::UserAccount) {
    relay
        .create_transaction(relay.account_id())
        .deploy_contract(RELAY_WASM_BYTES.to_vec())
        .submit()
        .assert_success();
    let result = relay.call(
        RELAY_ID.into(),
        "migrate_state",
        &json!({
            "new_note_of_validator": "migrate to new version",
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        0, // attached deposit
    );
    result.logs().iter().for_each(|l| println!("{}", l));
    println!(
        "Gas burnt of function 'migrate_state' of relay contract: {}",
        result.gas_burnt().to_formatted_string(&Locale::en)
    );
    result.assert_success();
}
