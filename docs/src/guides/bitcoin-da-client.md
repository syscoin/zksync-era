# Bitcoin Data Availability client (Gateway node)

This section explains how to enable the Bitcoin DA client for zkSync nodes. The implementation leverages the
[Syscoin PoDA service](https://docs.syscoin.org/docs/tech/poda/) to store data off chain. PoDA is not a standard Bitcoin
node: it runs on a Syscoin node that is secured by Bitcoin miners through merged mining.

## Enabling the client (on Gateway / settlement layer)

Set `DA_CLIENT=Bitcoin` in your environment or configuration file. This is intended for the Gateway (Validium) node. For
the External Node the variables should be prefixed with `EN_` (e.g. `EN_DA_CLIENT`).

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

### `smart_config` example

To configure the node with the new layered configuration format, add a fragment similar to:

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

When using Bitcoin DA for Validium, set `state_keeper.max_pubdata_per_batch` to about `750_000` bytes. It can support
2MB but the circuits currently support around 761856 bytes (based on 6 EIP4844 blobs). This will naturally increase
which comes at the cost of additional pubdata slots (at 2mb it will use 200mb RAM for the bootloader). For now we keep
it under the pre-configured circuit, slot and bootloader limits.
