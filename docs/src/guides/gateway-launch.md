# Gateway launch

This guide shows how to launch a gateway chain using ZK Stack.

## Ecosystem setup

Initialize the ecosystem:

```bash
zkstack ecosystem create
# or
zkstack ecosystem init
```

## Chain creation

Create the chain with Bitcoin data availability:

```bash
zkstack chain create --l1-batch-commit-data-generator-mode validium --validium-type bitcoin
```

## Chain initialization

Use the gateway configuration from [`etc/env/ecosystems/gateway/stage_gateway.yaml`](../../etc/env/ecosystems/gateway/stage_gateway.yaml):

```bash
zkstack chain init --gateway-config-path etc/env/ecosystems/gateway/stage_gateway.yaml
```

This generates the chain configs under `chains/<chain_name>/configs`.
Modify `general.yaml` there to adjust Bitcoin DA parameters. You can merge overrides using `smart_config` so the changes persist across updates.

## Run the server

Start the node:

```bash
zkstack server
```
