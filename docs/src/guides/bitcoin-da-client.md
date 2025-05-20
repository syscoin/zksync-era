# Bitcoin Data Availability client

This section explains how to enable the Bitcoin DA client for zkSync nodes. The implementation leverages the
[Syscoin PoDA service](hhttps://docs.syscoin.org/docs/tech/poda/) to store data off chain. PoDA is not a standard Bitcoin node: it runs
on a Syscoin node that is secured by Bitcoin miners through merged mining.

## Enabling the client

Set `DA_CLIENT=Bitcoin` in your environment or configuration file. For the External Node the variables should be
prefixed with `EN_` (e.g. `EN_DA_CLIENT`).

### Required variables

- `BITCOIN_API_NODE_URL` &ndash; URL of the Syscoin node that exposes Bitcoin chain data.
- `BITCOIN_PODA_URL` &ndash; endpoint of the PoDA service used to publish data on the merged-mined Syscoin chain.
- `DA_SECRETS_RPC_USER` &ndash; RPC user name for authenticating with PoDA.
- `DA_SECRETS_RPC_PASSWORD` &ndash; RPC password for PoDA.

Put secret values into your secrets configuration or export them as environment variables. Example:

```bash
export DA_CLIENT=Bitcoin
export BITCOIN_API_NODE_URL="https://bitcoin-node.example.com"
export BITCOIN_PODA_URL="https://poda.example.com"
export DA_SECRETS_RPC_USER="user"
export DA_SECRETS_RPC_PASSWORD="password"
```

For instructions on running the PoDA service and Syscoin node see the
[Syscoin GitHub repository](https://github.com/syscoin/syscoin).

### `smart_config` example

To configure the node with the new layered configuration format, add a fragment similar to:

```yaml
da_client:
  client: Bitcoin
  api_node_url: https://bitcoin-node.example.com
  poda_url: https://poda.example.com
```

Secrets can be provided via `secrets.yaml` or environment variables:

```yaml
da_client:
  rpc_user: user
  rpc_password: password
```

When using Bitcoin DA for Validium, set `state_keeper.max_pubdata_per_batch` to about `2_000_000` bytes. See [Configuration Format Changes](../announcements/config_format_changes.md) for details.
