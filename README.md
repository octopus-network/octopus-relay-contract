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
export RELAY_CONTRACT_ID=dev_account_in_neardev

# Set OCT token contract ID
export OCT_TOKEN_CONTRACT_ID=oct_token_account_id

# Set bridge token contract ID
export BRIDGE_TOKEN_CONTRACT_ID=bridge_token_account_id

# Set account ID for signing transactions
export SIGNER=your_account_id

# Initialize contract with given council and parameters (this is for testing, where you stil have access key to the contract).
near call $RELAY_CONTRACT_ID new '{"token_contract_id": "'$OCT_TOKEN_CONTRACT_ID'", "appchain_minium_validators": 2, "minium_staking_amount": "100000000000000000000000000", "bridge_limit_ratio": 3333, "oct_token_price": "2000000" }' --accountId $RELAY_CONTRACT_ID
```

### Use test contracts

```bash
# Set contract Id
export RELAY_CONTRACT_ID=oct-relay.testnet

# Set OCT token contract Id
export OCT_TOKEN_CONTRACT_ID=oct-token.testnet

# Set bridge token contract Id
export BRIDGE_TOKEN_CONTRACT_ID=test-stable.testnet
```

### Use dev contract

```bash
# Set contract Id
export RELAY_CONTRACT_ID=dev-oct-relay.testnet

# Set OCT token contract Id, it is the same as testnet
export OCT_TOKEN_CONTRACT_ID=oct-token.testnet

# Set bridge token contract Id, it is the same as testnet
export BRIDGE_TOKEN_CONTRACT_ID=test-stable.testnet
```

### Usage

```bash
# Storage deposit
near call $OCT_TOKEN_CONTRACT_ID storage_deposit  '{"account_id": "'$RELAY_CONTRACT_ID'"}' --accountId $SIGNER --amount 0.1

near call $BRIDGE_TOKEN_CONTRACT_ID storage_deposit  '{"account_id": "'$RELAY_CONTRACT_ID'"}' --accountId $SIGNER --amount 0.1

# Register appchain
near call $OCT_TOKEN_CONTRACT_ID ft_transfer_call '{"receiver_id": "'$RELAY_CONTRACT_ID'", "amount": "200000000000000000000000000", "msg": "register_appchain,testchain,website_url_string,github_address_string,chain_spec_url,chain_spec_hash"}' --accountId $SIGNER --amount 0.000000000000000000000001


# Pass appchain
near call $RELAY_CONTRACT_ID pass_appchain '{"appchain_id": "testchain"}' --accountId $RELAY_CONTRACT_ID --gas 300000000000000

# Appchain go staging
near call $RELAY_CONTRACT_ID appchain_go_staging '{"appchain_id": "testchain"}' --accountId $RELAY_CONTRACT_ID --gas 300000000000000

# View appchain
near view $RELAY_CONTRACT_ID get_appchain '{"appchain_id": "testchain"}'

# View number of appchains
near view $RELAY_CONTRACT_ID get_num_appchains ''

# Stake
near call $OCT_TOKEN_CONTRACT_ID ft_transfer_call '{"receiver_id": "'$RELAY_CONTRACT_ID'", "amount": "200000000000000000000000000", "msg": "stake,testchain,validator_id0"}' --accountId $SIGNER --amount 0.000000000000000000000001

# View current validators(Not finalized)
near view $RELAY_CONTRACT_ID get_validators '{"appchain_id": "testchain"}'

# Remove appchain
near call $RELAY_CONTRACT_ID remove_appchain '{"appchain_id": "testchain"}' --accountId $RELAY_CONTRACT_ID --gas 300000000000000

# Activate appchain
near call $RELAY_CONTRACT_ID activate_appchain '{"appchain_id": "testchain", "boot_nodes": "[\"/ip4/3.113.45.140/tcp/30333/p2p/12D3KooWAxYKgdmTczLioD1jkzMyaDuV2Q5VHBsJxPr5zEmHr8nY\",   \"/ip4/18.179.183.182/tcp/30333/p2p/12D3KooWSmLVShww4w9PVW17cCAS5C1JnXBU4NbY7FcGGjMyUGiq\",   \"/ip4/54.168.14.201/tcp/30333/p2p/12D3KooWT2umkS7F8GzUTLrfUzVBJPKn6YwCcuv6LBFQ27UPoo2Y\",   \"/ip4/35.74.18.116/tcp/30333/p2p/12D3KooWHNf9JxUZKHoF7rrsmorv86gonXSb2ZU44CbMsnBNFSAJ\", ]", "rpc_endpoint": "wss://easydeal-dev.rpc.testnet.oct.network:9944", "chain_spec_url": "chain_spec_url", "chain_spec_hash": "chain_spec_hash", "chain_spec_raw_url": "chain_spec_raw_url", "chain_spec_raw_hash": "chain_spec_raw_hash"}' --accountId $RELAY_CONTRACT_ID --gas 300000000000000

# Update appchain
near call $RELAY_CONTRACT_ID update_appchain '{"appchain_id": "testchain", "website_url": "website_url", "github_address": "github_address", "github_release": "github_release", "commit_id": "commit_id", "chain_spec_url": "chain_spec_url", "chain_spec_hash": "chain_spec_hash", "chain_spec_raw_url": "chain_spec_raw_url", "chain_spec_raw_hash": "chain_spec_raw_hash"}' --accountId $SIGNER

# Get finalized validator_set
near view $RELAY_CONTRACT_ID get_validator_set '{"appchain_id": "testchain"}'

# Stake more
near call $OCT_TOKEN_CONTRACT_ID ft_transfer_call '{"receiver_id": "'$RELAY_CONTRACT_ID'", "amount": "200000000000000000000000000", "msg": "stake_more,testchain"}' --accountId $SIGNER --amount 0.000000000000000000000001

# Get finalized validator_set_index
near view $RELAY_CONTRACT_ID get_curr_validator_set_index '{"appchain_id": "testchain"}'

# Get finalized validator_set by sequence number
near view $RELAY_CONTRACT_ID get_validator_set_by_seq_num '{"appchain_id": "testchain", "seq_num": 0}'

# Register bridge_token, 1000000 means 1.0000000000 usd
near call $RELAY_CONTRACT_ID register_bridge_token '{"token_id": "'$BRIDGE_TOKEN_CONTRACT_ID'", "symbol": "TSB", "price": "1000000", "decimals": 12}' --accountId $RELAY_CONTRACT_ID

# set token bridge permitted for appchain
near call $RELAY_CONTRACT_ID set_bridge_permitted '{"token_id": "'$BRIDGE_TOKEN_CONTRACT_ID'", "appchain_id": "testchain", "permitted": true}' --accountId $RELAY_CONTRACT_ID

# view bridge_token
near view $RELAY_CONTRACT_ID get_bridge_token '{"token_id": "'$BRIDGE_TOKEN_CONTRACT_ID'"}'

# get get_bridge_allowed_amount
near view $RELAY_CONTRACT_ID get_bridge_allowed_amount '{"appchain_id": "testchain", "token_id": "'$BRIDGE_TOKEN_CONTRACT_ID'"}'

# lock token
near call $BRIDGE_TOKEN_CONTRACT_ID ft_transfer_call '{"receiver_id": "'$RELAY_CONTRACT_ID'", "amount": "100000000000000", "msg": "lock_token,testchain,receiver_id"}' --accountId $SIGNER --amount 0.000000000000000000000001

# get_locked_events
near view $RELAY_CONTRACT_ID get_locked_events '{"appchain_id": "testchain", "start": 0, "limit": 100}'
```
