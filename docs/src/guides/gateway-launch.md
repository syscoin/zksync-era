# Launching a chain on ZK Gateway with Bitcoin DA

This tutorial shows how to deploy Gateway contracts, create the first chain using Bitcoin as the data availability layer, and run the node using the new `smart_config` format.

1. **Create the ecosystem and initialize contracts**

   ```bash
   zkstack ecosystem create
   zkstack ecosystem init
   ```

2. **Create the Gateway chain in Validium mode**

   ```bash
   zkstack chain create \
       --chain-name gateway \
       --chain-id 580 \
       --l1-batch-commit-data-generator-mode validium
   ```

3. **Init the Gateway chain with Bitcoin DA**
  Use the gateway configuration from [`etc/env/ecosystems/gateway/stage_gateway.yaml`](../../etc/env/ecosystems/gateway/stage_gateway.yaml):
   ```bash
   zkstack chain init \
       --chain gateway \
       --validium-type bitcoin \
       --gateway-config-path ./etc/env/ecosystems/gateway/stage_gateway.yaml
   ```

4. **Convert the chain to a Gateway settlement layer**

   ```bash
   zkstack chain gateway convert-to-gateway --chain gateway
   ```

5. **Create and register a child Rollup chain (zkSYS) on Gateway**

   ```bash
   # Create the chain
   zkstack chain create \
       --chain-name zksys \
       --chain-id 581 \
       --l1-batch-commit-data-generator-mode rollup

   # Initialize it against Gateway
   zkstack chain init \
       --chain zksys \
       --gateway-config-path ./etc/env/ecosystems/gateway/stage_gateway.yaml
   ```

   The commands deploy contracts, register the chain in BridgeHub and link it to Gateway.

6. **Adjust the Gateway chain configuration**

   Edit `chains/<chain_name>/configs/general.yaml` and set

   ```yaml
   state_keeper:
     max_pubdata_per_batch: 2_000_000
   ```

   Add the [Bitcoin DA smart_config](./bitcoin-da-client.md#smart_config-example) snippet for the DA client.

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
