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
near call $CONTRACT_ID new '{"token_contract_id": "'$TOKEN_CONTRACT_ID'", "appchain_minium_validators": 2, "minium_staking_amount": "100000000000000000000000000"}' --accountId $CONTRACT_ID
```

### Use test contracts initialized by Octopus

```bash
# Set contract Id
export CONTRACT_ID=dev-1618441266496-9170291

# Set token contract Id
export TOKEN_CONTRACT_ID=dev-1618322195294-7281987
```

### Usage

```bash
# Storage deposit
near call $TOKEN_CONTRACT_ID storage_deposit  '{"account_id": "'$CONTRACT_ID'"}' --accountId $SIGNER --amount 0.1

# Registry appchain
near call $TOKEN_CONTRACT_ID ft_transfer_call '{"receiver_id": "'$CONTRACT_ID'", "amount": "200000000000000000000000000", "msg": "register_appchain,your_appchain_name,website_url,github_address"}' --accountId $SIGNER --amount 0.000000000000000000000001

# View appchain
near view $CONTRACT_ID get_appchain '{"appchain_id": 0}'

# Staking
near call $TOKEN_CONTRACT_ID ft_transfer_call '{"receiver_id": "'$CONTRACT_ID'", "amount": "200000000000000000000000000", "msg": "staking,0,validator_id0"}' --accountId $SIGNER --amount 0.000000000000000000000001

# View current validators(Not finalized)
near view $CONTRACT_ID get_validators '{"appchain_id": 0}'

# Get finalized validator_set
near view $CONTRACT_ID get_validator_set '{"appchain_id": 0}'

# Get validator_set by sequence number
near view $CONTRACT_ID get_validator_set_by_seq_num '{"appchain_id": 0, "seq_num": 0}'
```
