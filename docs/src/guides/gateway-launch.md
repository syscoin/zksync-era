# Launching a chain on ZK Gateway with Bitcoin DA

This tutorial shows how to deploy Gateway contracts, create the first chain using Bitcoin as the data availability
layer, and run the node using the new `smart_config` format.

1. **Create the Gateway ecosystem (call it 'gateway' and use chain-id 57057) in Validium mode**

   ```bash
   zkstack ecosystem create
   ```

2. **Init the Gateway ecosystem with Bitcoin DA (tanenbaum=5700, mainnet=57). Use L1 RPC when it asks.**

   ```bash
   cd gateway
   zkstack dev clean contracts-cache
   FOUNDRY_EVM_VERSION=shanghai FOUNDRY_CHAIN_ID=5700 zkstack ecosystem init
   ```

3. **Convert the chain to a Gateway settlement layer**

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

4. **Create and register a child Rollup chain (zkSYS) on Gateway**

   ```bash
   # Create the chain
   zkstack chain create \
       --chain-name zksys \
       --chain-id 57001 \
       --l1-batch-commit-data-generator-mode rollup

   # Initialize it against Gateway (uses addresses generated in `chains/gateway/configs/gateway.yaml`). Use L1 RPC when it asks for RPC here as well.
   FOUNDRY_EVM_VERSION=shanghai FOUNDRY_CHAIN_ID=5700 zkstack chain init --chain zksys

   # Apply l3_to_gateway override (recommended for L3 chains settling on gateway)
   zkstack dev config-writer --path ../etc/env/file_based/overrides/l3_to_gateway.yaml --chain zksys

   # Migrate the chain to gateway
   FOUNDRY_EVM_VERSION=shanghai FOUNDRY_CHAIN_ID=5700 zkstack chain gateway migrate-to-gateway --chain zksys --gateway-chain-name gateway
   ```

   The commands deploy contracts, register the chain in BridgeHub and link it to Gateway.

5. **Adjust the Gateway/zkSYS chain configuration**

   Add the DA client configuration for Syscoin PoDA:

   ```yaml
   da_client:
     client: Bitcoin
     api_node_url: http://localhost:8369 # Syscoin NEVM RPC/API node
     poda_url: https://poda.syscoin.org # PoDA endpoint (or your own)
   ```

   Then set the required secrets (credentials and Gateway RPC URL) in `chains/gateway/configs/secrets.yaml`:

   ```yaml
   l1:
     gateway_rpc_url: http://127.0.0.1:4050/ # Gateway chain RPC (your Gateway node)

   da_client:
     rpc_user: YOUR_SYSCOIN_RPC_USER
     rpc_password: YOUR_SYSCOIN_RPC_PASSWORD
   ```

   Or, as environment variables (alternative to editing secrets):

   ```bash
   # Gateway server (settlement layer)
   export L1_GATEWAY_WEB3_URL="http://127.0.0.1:4050/"

   # External node for Gateway
   export EN_GATEWAY_URL="http://127.0.0.1:4050/"
   ```

   For more details, see the [Bitcoin DA smart_config](./bitcoin-da-client.md#smart_config-example).

6. **Run the nodes**

   ```bash
   # Gateway node (Validium + Bitcoin DA)
   zkstack server --chain gateway &
   zkstack server wait --verbose --chain gateway

   # zkSYS rollup node
   zkstack server --chain zksys &
   zkstack server wait --verbose --chain zksys
   ```

This launches the first zkSYS chain on Gateway with BitcoinDA
