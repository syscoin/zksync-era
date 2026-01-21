# Launching a chain on ZK Gateway with Bitcoin DA

This tutorial shows how to deploy Gateway contracts, create the first chain using Bitcoin as the data availability
layer, and run the node using the new `smart_config` format.

## Prerequisites

- Install dependencies from `docs/src/guides/setup-dev.md`.
- Clone the repo and init submodules:
  ```bash
  git clone https://github.com/syscoin/zksync-era.git
  cd zksync-era
  git submodule update --init --recursive
  ```
- Build the `zkstack` CLI locally:
  ```bash
  curl -L https://raw.githubusercontent.com/matter-labs/zksync-era/main/zkstack_cli/zkstackup/install | bash
  zkstackup --local
  ```
- Make sure you have a Syscoin node + PoDA service available (or an RPC to one).
- Ensure Postgres is running. If you are using `zkstack containers`, run it before starting the steps below:
  ```bash
  zkstack containers
  ```

### Wallet funding (public Syscoin networks)

Before deploying contracts, fund the wallets listed in:

- `configs/wallets.yaml` (ecosystem wallets)
- `chains/gateway/configs/wallets.yaml`
- `chains/zksys/configs/wallets.yaml`

At a minimum, the `deployer` and `governor` wallets should have enough SYS to cover deployment and migration
transactions. The CLI recommends ~5 SYS for contract deployment on public networks; plan for at least that in the
deployer wallet.

Suggested funding (from the Syscoin deployment README):

- Ecosystem wallets (`configs/wallets.yaml`):
  - deployer: 1 SYS
  - governor: 5 SYS
- Gateway wallets (`chains/gateway/configs/wallets.yaml`):
  - deployer: 1 SYS
  - governor: 0.1 SYS
- zkSYS wallets (`chains/zksys/configs/wallets.yaml`):
  - deployer: 0.2 SYS
  - governor: 2 SYS

1. **Create the Gateway ecosystem (call it 'gateway' and use chain-id 57001) in Validium mode with Bitcoin DA**

   ```bash
   export ZKSYNC_ERA_PATH=/path/to/zksync-era # Path to zksync-era repo
   export GATEWAY_CHAIN_ID=57001 # Gateway chain id
   export GATEWAY_PROVER_MODE=gpu # Gateway prover mode: no-proofs/gpu
   export GATEWAY_COMMIT_MODE=validium # Gateway commit mode: validium/rollup

   zkstack ecosystem create \
     --ecosystem-name gateway \
     --link-to-code ${ZKSYNC_ERA_PATH} \
     --chain-name gateway \
     --chain-id ${GATEWAY_CHAIN_ID} \
     --prover-mode ${GATEWAY_PROVER_MODE} \
     --wallet-creation random \
     --l1-batch-commit-data-generator-mode ${GATEWAY_COMMIT_MODE} \
     --validium-type bitcoin \
     --base-token-address 0x0000000000000000000000000000000000000001 \
     --base-token-price-nominator 1 \
     --base-token-price-denominator 1 \
     --evm-emulator false
   ```

2. **Deploy ecosystem contracts (Syscoin L1: tanenbaum=5700, mainnet=57).**

   ```bash
   export L1_RPC_URL=http://localhost:8545 # Syscoin L1 RPC
   cd gateway
   # Update token_weth_address in configs/initial_deployments.yaml before deploying.
   # Tanenbaum: 0xa66b2E50c2b805F31712beA422D0D9e7D0Fd0F35
   # Mainnet:   0xd3e822f3ef011Ca5f17D82C956D952D8d7C3A1BB
   zkstack dev clean all
   zkstack dev contracts
   FOUNDRY_EVM_VERSION=shanghai FOUNDRY_CHAIN_ID=5700 zkstack ecosystem init \
     --update-submodules true \
     --l1-rpc-url ${L1_RPC_URL} \
     --deploy-ecosystem true \
     --deploy-erc20 false \
     --deploy-paymaster false \
     --ecosystem-only \
     --no-genesis \
     --observability false
   ```

   If prompted about having less than 5 SYS, select "Proceed with the deployment anyway" if your wallet is funded.

   `FOUNDRY_CHAIN_ID` here refers to the Syscoin L1 chain ID, not the Gateway chain ID (which is 57001).

3. **Deploy Gateway chain contracts (Validium + Bitcoin DA)**

   ```bash
   FOUNDRY_EVM_VERSION=shanghai FOUNDRY_CHAIN_ID=5700 zkstack chain init \
     --no-genesis \
     --deploy-paymaster false \
     --l1-rpc-url ${L1_RPC_URL} \
     --chain gateway

   # This will open an interactive input session to provide Bitcoin DA values:
   # - Validium type: Bitcoin
   # - Bitcoin DA RPC URL
   # - PoDA URL (e.g. https://poda.syscoin.org)
   # - Bitcoin DA RPC user / password
   ```

4. **Convert the chain to a Gateway settlement layer**

   ```bash
   FOUNDRY_EVM_VERSION=shanghai FOUNDRY_CHAIN_ID=5700 zkstack chain gateway create-tx-filterer --chain gateway
   FOUNDRY_EVM_VERSION=shanghai FOUNDRY_CHAIN_ID=5700 zkstack chain gateway convert-to-gateway --chain gateway

   # Apply gateway override
   zkstack dev config-writer --path ../etc/env/file_based/overrides/gateway.yaml --chain gateway

   # Apply network override (for testnet)
   zkstack dev config-writer --path ../etc/env/file_based/overrides/testnet.yaml --chain gateway

   # OR For mainnet:
   zkstack dev config-writer --path ../etc/env/file_based/overrides/mainnet.yaml --chain gateway
   ```

5. **Create and register a child Rollup chain (zkSYS) on Gateway**

   ```bash
   export ZKSYS_CHAIN_ID=57057 # zkSYS chain id
   export ZKSYS_PROVER_MODE=gpu # zkSYS prover mode: no-proofs/gpu
   export ZKSYS_COMMIT_MODE=rollup # zkSYS commit mode: validium/rollup

   # Create the chain
   zkstack chain create \
       --chain-name zksys \
       --chain-id ${ZKSYS_CHAIN_ID} \
       --prover-mode ${ZKSYS_PROVER_MODE} \
       --wallet-creation random \
       --l1-batch-commit-data-generator-mode ${ZKSYS_COMMIT_MODE} \
       --base-token-address 0x0000000000000000000000000000000000000001 \
       --base-token-price-nominator 1 \
       --base-token-price-denominator 1 \
       --override l3_to_gateway

   # Initialize it against L1. Use L1 RPC when it asks for RPC here as well.
   # This creates a chain without creating priority txs immediately.
   FOUNDRY_EVM_VERSION=shanghai FOUNDRY_CHAIN_ID=5700 zkstack chain init \
     --chain zksys \
     --no-genesis \
     --deploy-paymaster false \
     --skip-priority-txs \
     --l1-rpc-url ${L1_RPC_URL}

   # If your Gateway RPC is not local, update it before migration:
   # (the migration command reads api.web3_json_rpc.http_url from the Gateway general config)
   vi chains/gateway/configs/general.yaml

   # Update the below config
   api:
     web3_json_rpc:
       http_url: <GATEWAY_PUBLIC_RPC_URL>

   # Migrate the chain to gateway. Make sure the Gateway chain is running and publicly available.
   # At this stage PQ is indeed empty before starting the migration process
   # Skips the `set_da_validator_pair_via_gateway` call if the priority txs were skipped. Note that this step is not skipped when a chain migrates back to gateway, as the contract must have already been deployed.
   FOUNDRY_EVM_VERSION=shanghai FOUNDRY_CHAIN_ID=5700 zkstack chain gateway migrate-to-gateway --chain zksys --gateway-chain-name gateway -v

   # Finalize the migration of the chain on gateway that sends the skipped priority txs and later calls `set_da_validator_pair_via_gateway`.
   FOUNDRY_EVM_VERSION=shanghai FOUNDRY_CHAIN_ID=5700 zkstack chain gateway finalize-chain-migration-to-gateway --chain zksys --gateway-chain-name gateway

   # Apply l3_to_gateway override (recommended for L3 chains settling on gateway)
   zkstack dev config-writer --path ../etc/env/file_based/overrides/l3_to_gateway.yaml --chain zksys
   ```

   The commands deploy contracts, register the chain in BridgeHub and link it to Gateway.

   Note: zkSYS (child rollup) does not use the Bitcoin DA client; keep it in rollup mode.

6. **Adjust the Gateway chain configuration**

   Add the DA client configuration for Syscoin PoDA (Gateway only):

   ```yaml
   da_client:
     client: Bitcoin
     api_node_url: http://localhost:8370 # Syscoin NEVM RPC/API node
     poda_url: https://poda.syscoin.org # PoDA endpoint (or your own)
   ```

   Then set the required secrets (credentials and Gateway RPC URL) in `chains/gateway/configs/secrets.yaml`:

   ```yaml
   l1:
     gateway_rpc_url: http://127.0.0.1:3050/ # Gateway chain RPC (your Gateway node)

   da_client:
     rpc_user: YOUR_SYSCOIN_RPC_USER
     rpc_password: YOUR_SYSCOIN_RPC_PASSWORD
   ```

   Or, as environment variables (alternative to editing secrets):

   ```bash
   # Gateway server (settlement layer)
   export L1_GATEWAY_WEB3_URL="http://127.0.0.1:3050/"

   # External node for Gateway
   export EN_GATEWAY_URL="http://127.0.0.1:3050/"
   ```

   Also set `state_keeper.max_pubdata_per_batch` for Bitcoin DA (Gateway only):

   ```yaml
   state_keeper:
     max_pubdata_per_batch: 750000
   ```

   For more details, see the [Bitcoin DA smart_config](./bitcoin-da-client.md#smart_config-example).

7. **Run the nodes**

   ```bash
   # Gateway node (Validium + Bitcoin DA)
   zkstack server --chain gateway &
   zkstack server wait --verbose --chain gateway

   # zkSYS rollup node
   zkstack server --chain zksys &
   zkstack server wait --verbose --chain zksys
   ```

This launches the first zkSYS chain on Gateway with BitcoinDA
