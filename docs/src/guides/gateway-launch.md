# Launching a chain on ZK Gateway with Bitcoin DA

This tutorial shows how to deploy Gateway contracts, create the first chain using Bitcoin as the data availability layer, and run the node using the new `smart_config` format.

1. **Create the ecosystem and initialize contracts**

   ```bash
   zkstack ecosystem create
   zkstack ecosystem init
   ```

2. **Create the chain using Bitcoin Validium**

   ```bash
   zkstack chain create \
       --l1-batch-commit-data-generator-mode validium \
       --validium-type bitcoin
   ```

3. **Deploy the chain and register it on Gateway**

  Use the gateway configuration from [`etc/env/ecosystems/gateway/stage_gateway.yaml`](../../etc/env/ecosystems/gateway/stage_gateway.yaml):

   ```bash
   zkstack chain init --gateway-config-path ./etc/env/ecosystems/gateway/stage_gateway.yaml
   ```

   The command deploys the contracts, registers the chain in the BridgeHub, and uses the Gateway configuration file.

4. **Adjust the chain configuration**

   Edit `chains/<chain_name>/configs/general.yaml` and set

   ```yaml
   state_keeper:
     max_pubdata_per_batch: 2_000_000
   ```

   Add the [Bitcoin DA smart_config](./bitcoin-da-client.md#smart_config-example) snippet for the DA client.

5. **Run the node**

   ```bash
   zkstack server
   ```

This launches the first zkSYS chain on Gateway with Bitcoin providing off-chain data availability
