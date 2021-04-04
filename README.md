# Octopus Realy Smart Contract

Octopus relay is part of the Octopus Network.

## Building

```bash

./build.sh

```

## Testing

```bash

cargo test --package octopus-relay -- --nocapture

```

## Deploy And Usage

```bash

# Deploy to new account on TestNet, and paste the account id to below
near dev-deploy

# Set contract Id (fish)
export CONTRACT_ID="dev-1617505091223-4534842"

# Set token contract Id
export TOKEN_CONTRACT_ID="dev-1616962983544-1322706"

# Initialize contract with given council and parameters (this is for testing, where you stil have access key to the contract).
near call $CONTRACT_ID new '{"owner": "your_id", "token_contract_id": "'$TOKEN_CONTRACT_ID'", "appchain_minium_validators": 2, "minium_staking_amount": 100}' --accountId your_id


# Storage deposit
near call $TOKEN_CONTRACT_ID storage_deposit  '{"account_id": "'$CONTRACT_ID'"}' --accountId your_id --amount 0.1

# Registry appchain
near call $TOKEN_CONTRACT_ID ft_transfer_call '{"receiver_id": "'$CONTRACT_ID'", "amount": "1000", "msg": "register_appchain,madchain,http://xasx.com,scsadvdfbfvervdsfvdfs"}' --accountId your_id --amount 0.000000000000000000000001


```
