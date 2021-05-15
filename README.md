# Octopus Realy Smart Contract

Octopus Relay is part of the Octopus Network.

This is the smart contract, you can also follow the [Webapp](https://github.com/octopus-network/octopus-relay-webapp.git).

## Building

```bash
./build.sh
```

## Testing

```bash
cargo test --package octopus-relay-contract -- --nocapture
```

## Deploy And Usage

### Deploy & Init

```bash
# Deploy to new account on TestNet, and paste the account id to below
near dev-deploy

# Set contract ID
export CONTRACT_ID=dev_account_in_neardev

# Set token contract ID
export TOKEN_CONTRACT_ID=token_account_id

# Set account ID for signing transactions
export SIGNER=your_account_id

# Initialize contract with given council and parameters (this is for testing, where you stil have access key to the contract).
near call $CONTRACT_ID new '{"token_contract_id": "'$TOKEN_CONTRACT_ID'", "appchain_minium_validators": 1, "minium_staking_amount": "100000000000000000000000000", "bridge_limit_ratio": 3333, "oct_token_price": "2000000" }' --accountId $CONTRACT_ID
```

### Use test contracts initialized by Octopus

```bash
# Set contract Id
export CONTRACT_ID=oct-relay.testnet

# Set token contract Id
export TOKEN_CONTRACT_ID=oct-token.testnet
```

### Usage

```bash
# Storage deposit
near call $TOKEN_CONTRACT_ID storage_deposit  '{"account_id": "'$CONTRACT_ID'"}' --accountId $SIGNER --amount 0.1

# Registry appchain
near call $TOKEN_CONTRACT_ID ft_transfer_call '{"receiver_id": "'$CONTRACT_ID'", "amount": "200000000000000000000000000", "msg": "register_appchain,your_appchain_name,website_url,github_address"}' --accountId $SIGNER --amount 0.000000000000000000000001

# View appchain
near view $CONTRACT_ID get_appchain '{"appchain_id": 0}'

# View number of appchains
near view $CONTRACT_ID get_num_appchains ''

# Staking
near call $TOKEN_CONTRACT_ID ft_transfer_call '{"receiver_id": "'$CONTRACT_ID'", "amount": "200000000000000000000000000", "msg": "staking,0,validator_id0"}' --accountId $SIGNER --amount 0.000000000000000000000001

# View current validators(Not finalized)
near view $CONTRACT_ID get_validators '{"appchain_id": 0}'

# Update appchain
near call $CONTRACT_ID update_appchain '{"appchain_id": 0, "website_url": "website_url", "github_address": "github_address", "chain_spec_url": "chain_spec_url", "chain_spec_hash": "chain_spec_hash"}' --accountId $SIGNER

# Activate appchain
near call $CONTRACT_ID activate_appchain '{"appchain_id": 0, "boot_nodes": "boot_nodes_string", "rpc_endpoint": "rpc_endpoint"}' --accountId $CONTRACT_ID

# Get finalized validator_set
near view $CONTRACT_ID get_validator_set '{"appchain_id": 0}'

# Staking more
near call $TOKEN_CONTRACT_ID ft_transfer_call '{"receiver_id": "'$CONTRACT_ID'", "amount": "200000000000000000000000000", "msg": "staking_more,0"}' --accountId $SIGNER --amount 0.000000000000000000000001

# Get finalized validator_set_index
near view $CONTRACT_ID get_curr_validator_set_index '{"appchain_id": 0}'

# Get finalized validator_set by sequence number
near view $CONTRACT_ID get_validator_set_by_seq_num '{"appchain_id": 0, "seq_num": 0}'

# Register bridge_token, 1000000 means 1.0000000000 usd
near call $CONTRACT_ID register_bridge_token '{"token_id": "test-stable.testnet", "symbol": "TSB", "price": "1000000", "decimals": 12}' --accountId $CONTRACT_ID

# view bridge_token
near view $CONTRACT_ID get_bridge_token '{"token_id": "test-stable.testnet"}'

# call and get bridge_allowed_amount
near view $CONTRACT_ID get_bridge_allowed_limit '{"appchain_id": 0, "token_id": "test-stable.testnet"}'

```
