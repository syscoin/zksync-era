# Bitcoin Data Availability client

This section explains how to enable the Bitcoin DA client for zkSync nodes. The implementation leverages the
[Syscoin PoDA service](https://github.com/syscoin) to store data off chain. PoDA is not a standard Bitcoin node: it runs
on a Syscoin node that is secured by Bitcoin miners through merged mining.

## Enabling the client

Set `DA_CLIENT=Bitcoin` in your environment or configuration file. For the External Node the variables should be
prefixed with `EN_` (e.g. `EN_DA_CLIENT`).

If you use configuration files, the `da_client` section of `general.yaml` should look similar to the following:

```yaml
da_client:
  client: Bitcoin
  api_node_url: https://bitcoin-node.example.com
  poda_url: https://poda.example.com
```

Secrets such as the RPC user and password must be stored in `secrets.yaml` or provided via environment variables.
For the External Node use variables prefixed with `EN_`.

For context on the new configuration format see [Configuration Format Changes](../announcements/config_format_changes.md).

### Required variables

- `BITCOIN_API_NODE_URL` &ndash; URL of the Syscoin node that exposes Bitcoin chain data.
- `BITCOIN_PODA_URL` &ndash; endpoint of the PoDA service used to publish data on the merged-mined Syscoin chain.
- `DA_SECRETS_RPC_USER` &ndash; RPC user name for authenticating with PoDA.
- `DA_SECRETS_RPC_PASSWORD` &ndash; RPC password for PoDA.

Example of setting parameters via environment variables:

```bash
export DA_CLIENT=Bitcoin
export BITCOIN_API_NODE_URL="https://bitcoin-node.example.com"
export BITCOIN_PODA_URL="https://poda.example.com"
export DA_SECRETS_RPC_USER="user"
export DA_SECRETS_RPC_PASSWORD="password"
```

For instructions on running the PoDA service and Syscoin node see the
[Syscoin GitHub repository](https://github.com/syscoin).
