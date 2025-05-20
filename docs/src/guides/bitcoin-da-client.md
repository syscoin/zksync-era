# Bitcoin Data Availability client

This section explains how to enable the Bitcoin DA client for zkSync nodes.

## Enabling the client

Set `DA_CLIENT=Bitcoin` in your environment or configuration file. For the External Node the variables should be prefixed with `EN_` (e.g. `EN_DA_CLIENT`).

### Required variables

- `BITCOIN_API_NODE_URL` &ndash; URL of the Bitcoin node that provides block data.
- `BITCOIN_PODA_URL` &ndash; endpoint of the PoDA service used to publish data on Bitcoin.
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

