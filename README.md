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

```bash

# Deploy to new account on TestNet, and paste the account id to below
near dev-deploy

# Set contract Id (fish)
export CONTRACT_ID="dev-1617522041326-1812977"

# Set token contract Id
export TOKEN_CONTRACT_ID="dev-1616962983544-1322706"

# Initialize contract with given council and parameters (this is for testing, where you stil have access key to the contract).
near call $CONTRACT_ID new '{"owner": "your_id", "token_contract_id": "'$TOKEN_CONTRACT_ID'", "appchain_minium_validators": 2, "minium_staking_amount": 100}' --accountId your_id


# Storage deposit
near call $TOKEN_CONTRACT_ID storage_deposit  '{"account_id": "'$CONTRACT_ID'"}' --accountId your_id --amount 0.1

# Registry appchain
near call $TOKEN_CONTRACT_ID ft_transfer_call '{"receiver_id": "'$CONTRACT_ID'", "amount": "1000", "msg": "register_appchain,your_appchain_name"}' --accountId your_id --amount 0.000000000000000000000001


```
