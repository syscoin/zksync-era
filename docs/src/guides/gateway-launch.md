# Launching a zkOS Gateway stack on Syscoin

This guide launches the future Gateway topology from scratch:

- a Gateway settlement chain on Syscoin L1
- Bitcoin DA on the Gateway settlement chain
- a child zkOS chain that settles to Gateway using relayed settlement-layer calldata
- `zksync-os-server` as the runtime
- `zksync-airbender` / `zksync-airbender-prover` as the proving path

## Topology

- Gateway settlement chain:
  - `validium` commit mode in the control plane
  - `Bitcoin` pubdata mode in `zksync-os-server`
- Child chain on Gateway:
  - `rollup` commit mode in the control plane
  - `RelayedL2Calldata` pubdata mode in `zksync-os-server`

## Control plane vs runtime

Right now the control plane and the runtime live in different places:

- `bitcoin+` / `zkstack`:
  - ecosystem creation
  - contract deployment
  - chain initialization
  - gateway conversion / migration
  - the `--zksync-os` switch that tells the control plane to generate a zkOS chain instead of an Era one
- `zksync-os-server`:
  - sequencer / batcher / API runtime
  - settlement-layer discovery
  - Bitcoin DA publication on the Gateway chain
- `zksync-airbender` / `zksync-airbender-prover`:
  - real zkOS proving path

Do not assume the legacy `zkstack server` / `zkstack prover` flow is the same thing as the current zkOS runtime + prover
path.

## Prerequisites

- Run `zkstack dev clean all`.
- Install dependencies from `docs/src/guides/setup-dev.md`.
- Clone the relevant repos:
  - `bitcoin+`
  - `bitcoin+-contracts`
  - `zksync-os`
  - `zksync-os-interface`
  - `zksync-os-server`
  - `zksync-airbender`
  - `zksync-airbender-prover`
  - `bitcoin_da_client`
- Build the `zkstack` CLI locally:
  ```bash
  curl -L https://raw.githubusercontent.com/matter-labs/zksync-era/main/zkstack_cli/zkstackup/install | bash
  zkstackup --local
  ```
- Make sure you have a Syscoin node + PoDA service available (or RPCs to them).
- Ensure Postgres is running. If you are using `zkstack containers`, run it before starting the steps below:
  ```bash
  zkstack containers
  ```

### Deterministic CLI input notes

Pass all required CLI values explicitly to avoid unexpected prompts in automated or non-interactive runs:

- Always set `--l1-network` during `zkstack ecosystem create` (for Syscoin testnet use `tanenbaum`).
- Always set explicit booleans for prompt-backed options such as `--set-as-default` and `--start-containers`.
- Do **not** use `-a` with `zkstack ecosystem create` or `zkstack chain create`; those subcommands do not accept it.

Example:

```bash
zkstack ecosystem create \
  --ecosystem-name gateway \
  --l1-network tanenbaum \
  --link-to-code ${ZKSYNC_ERA_PATH} \
  --chain-name gateway \
  --chain-id ${GATEWAY_CHAIN_ID} \
  --prover-mode ${GATEWAY_PROVER_MODE} \
  --wallet-creation random \
  --l1-batch-commit-data-generator-mode ${GATEWAY_COMMIT_MODE} \
  --base-token-address 0x0000000000000000000000000000000000000001 \
  --base-token-price-nominator 1 \
  --base-token-price-denominator 1 \
  --set-as-default true \
  --evm-emulator false \
  --start-containers false \
  --zksync-os
```

### Network variables

```bash
export ZKSYNC_ERA_PATH=/path/to/bitcoin+
export SYSCOIN_L1_CHAIN_ID=5700   # Tanenbaum; use 57 on mainnet
export L1_RPC_URL=http://localhost:8545

export FOUNDRY_EVM_VERSION=shanghai
export FOUNDRY_CHAIN_ID=${SYSCOIN_L1_CHAIN_ID}
```

### Wallet funding (public Syscoin networks)

Before deploying contracts, fund the wallets listed in:

- `configs/wallets.yaml` (ecosystem wallets)
- `chains/gateway/configs/wallets.yaml`
- `chains/zksys/configs/wallets.yaml` (after the child chain is created)

At a minimum, the `deployer` and `governor` wallets should have enough SYS to cover deployment and migration
transactions. The CLI enforces a 5 SYS minimum-balance check during deployment paths; in practice, fund above 5 SYS to
leave room for gas spent by preceding transactions in the same step.

Suggested funding:

- Ecosystem wallets (`configs/wallets.yaml`):
  - deployer: 6 SYS
  - governor: 6 SYS
  - operator: keep funded (recommended >= 1 SYS)
- Gateway wallets (`chains/gateway/configs/wallets.yaml`):
  - deployer: 6 SYS
  - governor: 6 SYS
  - operator: keep funded (recommended >= 1 SYS)
  - blob_operator: keep funded (recommended >= 0.1 SYS)
  - fee_account: keep funded (recommended >= 0.1 SYS)
  - token_multiplier_setter: keep funded (recommended >= 0.1 SYS)
  - prove_operator: keep funded (recommended >= 0.1 SYS)
  - execute_operator: keep funded (recommended >= 0.1 SYS)
- Child-chain wallets (`chains/zksys/configs/wallets.yaml`):
  - deployer: 6 SYS
  - governor: 6 SYS
  - operator: keep funded (recommended >= 1 SYS)
  - blob_operator: keep funded (recommended >= 0.1 SYS)
  - fee_account: keep funded (recommended >= 0.1 SYS)
  - token_multiplier_setter: keep funded (recommended >= 0.1 SYS)
  - prove_operator: keep funded (recommended >= 0.1 SYS)
  - execute_operator: keep funded (recommended >= 0.1 SYS)

The child-chain operator funding matters for migration: the current `migrate-to-gateway` flow checks a minimum validator
balance before sending the migration transactions.

The same 5 SYS deployer/governor minimum applies to child-chain initialization paths as well. If child `deployer` /
`governor` balances fall under this threshold, `chain init` may trigger an interactive low-balance prompt, which can
panic in non-interactive sessions (`Kind(NotConnected)`).

Preflight check before each major step (`ecosystem init`, `chain init`, `migrate-to-gateway`):

- Re-read the relevant wallets file(s) and confirm every wallet entry is funded: `deployer`, `governor`, `operator`,
  `blob_operator`, `fee_account`, `token_multiplier_setter`, `prove_operator`, and `execute_operator`.
- If any listed wallet is under the expected balance, top it up before continuing.

Important deployment constraint:

- `zkstack ecosystem init` runs a balance guard before deployment. If the deployer wallet is below the minimum
  threshold, the CLI opens an interactive "Proceed with the deployment anyway / Check balance again / Exit" prompt.
- In non-interactive environments, that prompt path can fail with `Kind(NotConnected)` instead of proceeding.
- Ensure deployer / governor balances stay comfortably above the minimum before running `ecosystem init` and CTM
  follow-ups.
- Passing forge args (for example private key or gas price) does not bypass this balance prompt path.

## 1. Create the Gateway ecosystem

Call the ecosystem `gateway` and use chain ID `57001`:

```bash
export GATEWAY_CHAIN_ID=57001
export GATEWAY_PROVER_MODE=gpu
export GATEWAY_COMMIT_MODE=validium

zkstack ecosystem create \
  --ecosystem-name gateway \
  --link-to-code ${ZKSYNC_ERA_PATH} \
  --chain-name gateway \
  --chain-id ${GATEWAY_CHAIN_ID} \
  --prover-mode ${GATEWAY_PROVER_MODE} \
  --wallet-creation random \
  --l1-batch-commit-data-generator-mode ${GATEWAY_COMMIT_MODE} \
  --base-token-address 0x0000000000000000000000000000000000000001 \
  --base-token-price-nominator 1 \
  --base-token-price-denominator 1 \
  --evm-emulator false \
  --zksync-os
```

Important:

- `GATEWAY_COMMIT_MODE` must be `validium`.
- `--zksync-os` is what tells `zkstack` to create a zkOS chain/control-plane config instead of an Era one.
- The Bitcoin DA type is applied during deploy / init steps, not during `ecosystem create`.

## 2. Deploy the ecosystem contracts on Syscoin L1

```bash
cd gateway

# Update token_weth_address in configs/initial_deployments.yaml before deploying.
# Tanenbaum: 0xa66b2E50c2b805F31712beA422D0D9e7D0Fd0F35
# Mainnet:   0xd3e822f3ef011Ca5f17D82C956D952D8d7C3A1BB

zkstack dev contracts

zkstack ecosystem init \
  --zksync-os \
  --update-submodules true \
  --l1-rpc-url ${L1_RPC_URL} \
  --deploy-ecosystem true \
  --deploy-erc20 false \
  --deploy-paymaster false \
  --ecosystem-only \
  --validium-type bitcoin \
  --no-genesis \
  --observability false
```

If prompted about having less than 5 SYS, select "Proceed with the deployment anyway" if your wallet is funded.

`FOUNDRY_CHAIN_ID` here refers to the Syscoin L1 chain ID, not the Gateway chain ID.

## 3. Initialize the Gateway chain directly on L1

Start with the Gateway chain as a direct-to-L1 Bitcoin validium:

```bash
zkstack chain init \
  --chain gateway \
  --validium-type bitcoin \
  --no-genesis \
  --deploy-paymaster false \
  --l1-rpc-url ${L1_RPC_URL}
```

If you omit `--validium-type bitcoin`, the init flow should collect the Bitcoin DA values interactively:

- Bitcoin DA RPC URL
- PoDA URL
- Bitcoin DA RPC user
- Bitcoin DA RPC password

## 4. Convert the direct-to-L1 chain into the Gateway settlement layer

```bash
zkstack chain gateway create-tx-filterer --chain gateway
zkstack chain gateway convert-to-gateway --chain gateway

# Apply network override first.
zkstack dev config-writer --path ../etc/env/file_based/overrides/testnet.yaml --chain gateway

# Or for mainnet:
# zkstack dev config-writer --path ../etc/env/file_based/overrides/mainnet.yaml --chain gateway

# Re-apply gateway override last so gateway-specific timing and precommit settings win.
zkstack dev config-writer --path ../etc/env/file_based/overrides/gateway.yaml --chain gateway
```

This is the intended shape: the Gateway settlement chain starts on L1, then is converted into the Gateway topology.
After conversion it is still the Gateway settlement chain and should continue to use Bitcoin DA.

## 5. Create the child zkOS rollup chain on top of Gateway

```bash
export CHILD_CHAIN_NAME=zksys
export CHILD_CHAIN_ID=57057
export CHILD_PROVER_MODE=gpu
export CHILD_COMMIT_MODE=rollup

zkstack chain create \
  --chain-name ${CHILD_CHAIN_NAME} \
  --chain-id ${CHILD_CHAIN_ID} \
  --prover-mode ${CHILD_PROVER_MODE} \
  --wallet-creation random \
  --l1-batch-commit-data-generator-mode ${CHILD_COMMIT_MODE} \
  --base-token-address 0x0000000000000000000000000000000000000001 \
  --base-token-price-nominator 1 \
  --base-token-price-denominator 1 \
  --set-as-default false \
  --evm-emulator false \
  --zksync-os
```

Important:

- `CHILD_COMMIT_MODE` must be `rollup`.
- Apply `etc/env/file_based/overrides/l3_to_gateway.yaml` later with `zkstack dev config-writer`.
- The child chain should later run `RelayedL2Calldata`, not `Bitcoin`.

## 6. Initialize the child chain on L1 and migrate it to Gateway

Initialize the child chain without sending the priority transactions immediately:

```bash
zkstack chain init \
  --chain ${CHILD_CHAIN_NAME} \
  --no-genesis \
  --deploy-paymaster false \
  --skip-priority-txs \
  --l1-rpc-url ${L1_RPC_URL}
```

If your Gateway RPC is not local, update it before migration. The migration command reads `api.web3_json_rpc.http_url`
from the Gateway chain general config:

```yaml
api:
  web3_json_rpc:
    http_url: <GATEWAY_PUBLIC_RPC_URL>
```

This setting is for the `zkstack` migration flow. Later, when you run `zksync-os-server`, the runtime-side setting is
`general.gateway_rpc_url`.

Then migrate the child chain to Gateway:

```bash
zkstack chain gateway migrate-to-gateway \
  --chain ${CHILD_CHAIN_NAME} \
  --gateway-chain-name gateway \
  -v

zkstack chain gateway finalize-chain-migration-to-gateway \
  --chain ${CHILD_CHAIN_NAME} \
  --gateway-chain-name gateway
```

Finally, apply the child-chain overrides:

```bash
zkstack dev config-writer --path ../etc/env/file_based/overrides/testnet.yaml --chain ${CHILD_CHAIN_NAME}

# Or for mainnet:
# zkstack dev config-writer --path ../etc/env/file_based/overrides/mainnet.yaml --chain ${CHILD_CHAIN_NAME}

# Re-apply the Gateway child override last.
zkstack dev config-writer --path ../etc/env/file_based/overrides/l3_to_gateway.yaml --chain ${CHILD_CHAIN_NAME}
```

### Migration gas-price and funding gotchas

The current Gateway migration flow has two important hardcoded assumptions:

- `zkstack chain gateway migrate-to-gateway` currently hardcodes
  `DEFAULT_MAX_L1_GAS_PRICE_FOR_PRIORITY_TXS = 50_000_000_000` (50 gwei) in
  `zkstack_cli/crates/zkstack/src/commands/chain/gateway/constants.rs`.
- The same flow checks that the child-chain operator / validator holds at least 1 SYS before migration.

If the network conditions on your target Syscoin environment require a higher gas cap, patch that constant and rebuild
`zkstack` before running the migration commands. There is no CLI flag for this today.

## 7. Sanity-check the DA modes before runtime

Before you try to run `zksync-os-server`, make sure the deployed settlement-layer pricing mode matches the runtime
pubdata mode you are about to configure:

- Gateway settlement chain:
  - deployed pricing mode must be `Validium`
  - runtime `l1_sender.pubdata_mode` must be `Bitcoin`
- Child chain on Gateway:
  - deployed pricing mode must be `Rollup`
  - runtime `l1_sender.pubdata_mode` must be `RelayedL2Calldata`

If the deployed state and runtime config do not agree, `zksync-os-server` will refuse to start with an error like:

```text
Pubdata mode doesn't correspond to pricing mode from the l1.
L1 mode: Rollup, configured pubdata mode: Bitcoin
```

Treat that as a deploy / config mismatch, not as a runtime bug.

## 8. Configure `zksync-os-server`

Use the `zkstack`-generated chain configs as the base input, then add the zkOS runtime-specific overrides below.

### Gateway settlement chain

```yaml
general:
  l1_rpc_url: <SYSCOIN_L1_RPC_URL>
  gateway_rpc_url: http://127.0.0.1:3050/

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

Notes:

- Keep `general.gateway_rpc_url` set in the Gateway topology. Runtime settlement-layer discovery handles the
  self-settlement case and avoids trying to connect to itself when the node is the settlement layer.
- The built-in zkOS defaults are more aggressive than the old Era public-network presets:
  - `sequencer.block_time` defaults to `250ms`
  - `batcher.batch_timeout` defaults to `1s`
  - `l1_sender.poll_interval` defaults to `100ms`
  - `l1_watcher.poll_interval` defaults to `100ms` Those are acceptable for local bring-up, but they are not the same
    operational profile as the old Syscoin/Gateway Era overrides.
- There is no ready-made zkOS `testnet.yaml` / `mainnet.yaml` preset bundle equivalent to the old Era runtime override
  stack. `zksync-os-server` uses code defaults plus whatever YAML files you pass via `--config`.
- On Syscoin testnet, confirmation-based Bitcoin DA finality is required because ChainLocks are not currently available.
- If you use `batcher.bitcoin_da_rpc_user: __cookie__`, set `batcher.bitcoin_da_rpc_password` to the actual cookie
  secret value from your Syscoin node cookie file. A placeholder value such as `dummy` will fail with HTTP 401.
- The Era override key `state_keeper.max_pubdata_per_batch` does not map 1:1 to a zkOS runtime setting. On the OS side,
  the relevant knobs live under `sequencer` and `batcher`, especially `sequencer.block_pubdata_limit_bytes`,
  `batcher.blocks_per_batch_limit`, and `batcher.batch_timeout`.
- On mainnet, switch the Gateway chain to:

  ```yaml
  batcher:
    bitcoin_da_finality_mode: chainlock
  ```

### Child chain on Gateway

```yaml
general:
  l1_rpc_url: <SYSCOIN_L1_RPC_URL>
  gateway_rpc_url: http://<GATEWAY_PUBLIC_RPC_URL>/

l1_sender:
  pubdata_mode: RelayedL2Calldata
```

Notes:

- Do not configure `bitcoin_da_*` on the child chain. The Gateway settlement chain is the one that publishes to Bitcoin
  DA.
- Keep the rollup-specific overrides from `l3_to_gateway.yaml` on the child chain.

### Recommended first-pass zkOS runtime translations

These are the Era-to-zkOS translations that are clear enough to carry over directly:

- Era `gateway.yaml`:
  - `state_keeper.miniblock_commit_deadline_ms: 10000` -> zkOS `sequencer.block_time: 10s`
  - `service block pacing` -> zkOS `sequencer.service_block_delay: 30s`
  - `state_keeper.block_commit_deadline_ms: 2400000` -> zkOS `batcher.batch_timeout: 40m`
- Era `validium.yaml`:
  - `eth.sender.pubdata_sending_mode: CUSTOM` -> zkOS `l1_sender.pubdata_mode: Bitcoin`
- Era `l3_to_gateway.yaml`:
  - `eth.sender.pubdata_sending_mode: CALLDATA` -> zkOS `l1_sender.pubdata_mode: RelayedL2Calldata`

Recommended Gateway-chain timing additions:

```yaml
sequencer:
  block_time: 10s
  service_block_delay: 30s

batcher:
  batch_timeout: 40m
```

You can change these later by editing the runtime config and restarting the node.

Recommended child-chain timing additions for the first bring-up:

- leave them unset and use zkOS defaults, or
- pin the defaults explicitly if you want the values written down in config

```yaml
sequencer:
  block_time: 250ms
  service_block_delay: 750ms

batcher:
  batch_timeout: 1s
```

For the child chain, keep the defaults initially unless you have measured a reason to change them. In particular, do not
immediately carry over Era's `max_pubdata_per_batch: 750000` assumption into zkOS runtime config. Start with zkOS
defaults and raise pubdata limits only after measuring proving latency and batch publication behavior.

## 9. Run the zkOS runtime

Use `zksync-os-server` directly, not `zkstack server`.

The node supports multiple `--config` flags or a `:`-delimited list, so you can keep base configs and local overrides
separate.

Gateway chain:

```bash
cd /path/to/zksync-os-server
cargo build --release

cargo run --release -- \
  --config /path/to/gateway/base.yaml \
  --config /path/to/gateway/local-overrides.yaml
```

Child chain:

```bash
cd /path/to/zksync-os-server

cargo run --release -- \
  --config /path/to/zksys/base.yaml \
  --config /path/to/zksys/local-overrides.yaml
```

If you prefer a single file per chain, pre-merge the YAMLs and pass just one `--config` value.

## 10. Run the proving stack

For smoke tests:

- fake provers are acceptable

For real proofs:

- use `zksync-airbender-prover`
- run separate runtime / prover stacks for the Gateway chain and the child chain
- do not assume the legacy `zkstack prover` flow is the right prover path for zkOS Gateway bring-up

At the topology level, the target shape is:

- Gateway chain: `validium` pricing + `Bitcoin`
- Child chain: `rollup` pricing + `RelayedL2Calldata`

## Troubleshooting from real deployments

- `invalid value 'cpu' for '--prover-mode'`:
  - Current accepted values are `no-proofs` and `gpu`.
- `unexpected argument '-a'` on `ecosystem create` or `chain create`:
  - Remove `-a` from those commands.
- Panic in `prompt/select.rs` with `Kind(NotConnected)` during non-interactive runs:
  - This is caused by an unresolved interactive prompt in a non-TTY session.
  - Most commonly this is the low-balance deployer/governor prompt in `ecosystem init` / CTM admin steps; fund above
    minimum first.
- Build failure `error[E0603]: module common is private` in `zkstack_cli`:
  - This indicates a branch mismatch / temporary regression. Pull the latest fix in your branch before deployment.
