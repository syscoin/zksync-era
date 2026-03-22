# Bitcoin Data Availability client (settlement node)

This section explains how to enable the Bitcoin DA client for zkSync nodes. The implementation leverages the
[Syscoin PoDA service](https://docs.syscoin.org/docs/tech/poda/) to store data off chain. PoDA is not a standard Bitcoin
node: it runs on a Syscoin node that is secured by Bitcoin miners through merged mining.

## Enabling the client (on the settlement layer)

The old `DA_CLIENT=Bitcoin` / `da_client.*` layout belongs to the legacy settlement-node / control-plane configuration
surface.

For `zksync-os-server`, the current integration uses:

- `l1_sender.pubdata_mode=Bitcoin`
- `batcher.bitcoin_da_*`

Do not assume the old `da_client.*` example by itself is sufficient for the current zkOS runtime.

### Required variables

- `BITCOIN_API_NODE_URL` &ndash;local RPC URL of the Syscoin node that exposes BitcoinDA.
- `BITCOIN_PODA_URL` &ndash; endpoint of the PoDA service used to publish data on the merged-mined Syscoin chain.
- `DA_SECRETS_RPC_USER` &ndash; RPC user name for authenticating with PoDA.
- `DA_SECRETS_RPC_PASSWORD` &ndash; RPC password for PoDA.

Put secret values into your secrets configuration or export them as environment variables. Example:

```bash
export DA_CLIENT=Bitcoin
export BITCOIN_API_NODE_URL="http://localhost:8369"
export BITCOIN_PODA_URL="https://poda.syscoin.org"
export DA_SECRETS_RPC_USER="user"
export DA_SECRETS_RPC_PASSWORD="password"
```

External Node variant (env variables are prefixed with `EN_`):

```bash
export EN_DA_CLIENT=Bitcoin
export EN_BITCOIN_API_NODE_URL="http://localhost:8369"
export EN_BITCOIN_PODA_URL="https://poda.syscoin.org"
export EN_DA_SECRETS_RPC_USER="user"
export EN_DA_SECRETS_RPC_PASSWORD="password"
```

Note: zkSYS (child rollup) does not use the Bitcoin DA client; keep it in rollup mode. For instructions on running the
PoDA service and Syscoin node see the [Syscoin GitHub repository](https://github.com/syscoin/syscoin).

### Legacy / control-plane example

If you are configuring the older control-plane / settlement-node stack, the legacy layout looked like this:

```yaml
da_client:
  client: Bitcoin
  api_node_url: http://localhost:8369
  poda_url: https://poda.syscoin.org
```

Secrets can be provided via `secrets.yaml` or environment variables:

```yaml
da_client:
  rpc_user: user
  rpc_password: password
```

The old Era-side recommendation about `state_keeper.max_pubdata_per_batch` does not carry over 1:1 to
`zksync-os-server`.

For zkOS:

- there is no direct `state_keeper.max_pubdata_per_batch` runtime key
- the closest runtime knobs live under `sequencer` and `batcher`
- in particular, `sequencer.block_pubdata_limit_bytes` is currently reused by the node as both a per-block limit and the
  batch pubdata seal limit

So for the current zkOS runtime path, prefer the `l1_sender` / `batcher` configuration below and do not blindly reuse
the old `750_000`-byte Era assumption.

### `zksync-os-server` batcher settings

For the current zkOS runtime path, configure Bitcoin DA on the batcher:

```yaml
l1_sender:
  pubdata_mode: Bitcoin

batcher:
  bitcoin_da_rpc_url: <SYSCOIN_NODE_RPC_URL>
  bitcoin_da_rpc_user: <RPC_USER_OR___cookie__>
  bitcoin_da_rpc_password: <RPC_PASSWORD_OR_COOKIE_SECRET>
  bitcoin_da_poda_url: <PODA_URL>
  bitcoin_da_wallet_name: zksync-os
  bitcoin_da_address_label: zksync-os-batcher
  bitcoin_da_request_timeout: 60s
  bitcoin_da_finality_poll_interval: 15s
  bitcoin_da_finality_mode: confirmations
  bitcoin_da_finality_confirmations: 5
  bitcoin_da_finality_timeout: 45m
```

Use finality modes as follows:

- testnet: `bitcoin_da_finality_mode: confirmations`
- mainnet: `bitcoin_da_finality_mode: chainlock`
