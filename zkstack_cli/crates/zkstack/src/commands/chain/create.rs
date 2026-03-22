use std::{cell::OnceCell, fs};

use anyhow::Context;
use serde_json::json;
use xshell::Shell;
use zkstack_cli_common::{logger, spinner::Spinner};
use zkstack_cli_config::{
    create_local_configs_dir, create_wallets, traits::SaveConfigWithBasePath, ChainConfig,
    EcosystemConfig, GenesisConfig, SourceFiles, ZkStackConfig,
};
use zksync_basic_types::L2ChainId;

use crate::{
    commands::chain::args::create::{ChainCreateArgs, ChainCreateArgsFinal},
    messages::{
        MSG_ARGS_VALIDATOR_ERR, MSG_CHAIN_CREATED, MSG_CREATING_CHAIN,
        MSG_CREATING_CHAIN_CONFIGURATIONS_SPINNER, MSG_EVM_EMULATOR_HASH_MISSING_ERR,
        MSG_SELECTED_CONFIG,
    },
};

pub async fn run(args: ChainCreateArgs, shell: &Shell) -> anyhow::Result<()> {
    // TODO support creating without ecosystem
    let mut ecosystem_config = ZkStackConfig::ecosystem(shell)?;
    create(args, &mut ecosystem_config, shell).await
}

pub async fn create(
    args: ChainCreateArgs,
    ecosystem_config: &mut EcosystemConfig,
    shell: &Shell,
) -> anyhow::Result<()> {
    let tokens = ecosystem_config.get_erc20_tokens();
    let args = args
        .fill_values_with_prompt(
            ecosystem_config.list_of_chains().len() as u32,
            &ecosystem_config.l1_network,
            tokens,
        )
        .context(MSG_ARGS_VALIDATOR_ERR)?;

    logger::note(MSG_SELECTED_CONFIG, logger::object_to_string(&args));
    logger::info(MSG_CREATING_CHAIN);

    let spinner = Spinner::new(MSG_CREATING_CHAIN_CONFIGURATIONS_SPINNER);
    let name = args.chain_name.clone();
    let set_as_default = args.set_as_default;
    create_chain_inner(args, ecosystem_config, shell).await?;
    if set_as_default {
        ecosystem_config.set_default_chain(name);
        ecosystem_config.save_with_base_path(shell, ".")?;
    }
    spinner.finish();

    logger::success(MSG_CHAIN_CREATED);

    Ok(())
}

pub(crate) async fn create_chain_inner(
    args: ChainCreateArgsFinal,
    ecosystem_config: &EcosystemConfig,
    shell: &Shell,
) -> anyhow::Result<()> {
    let vm_option = args.vm_option;
    if args.legacy_bridge {
        logger::warn("WARNING!!! You are creating a chain with legacy bridge, use it only for testing compatibility")
    }
    let default_chain_name = args.chain_name.clone();
    println!(
        "ecosystem_config.list_of_chains() before: {:?}",
        ecosystem_config.list_of_chains()
    );
    let internal_id = if ecosystem_config.list_of_chains().contains(&args.chain_name) {
        ecosystem_config
            .list_of_chains()
            .iter()
            .position(|x| *x == args.chain_name)
            .unwrap() as u32
            + 1
    } else {
        ecosystem_config.list_of_chains().len() as u32 + 1
    };
    println!("internal_id: {}", internal_id);
    let chain_path = ecosystem_config.chains.join(&default_chain_name);
    let chain_configs_path = create_local_configs_dir(shell, &chain_path)?;
    let (chain_id, legacy_bridge) = if args.legacy_bridge {
        // Legacy bridge is distinguished by using the same chain id as ecosystem
        (ecosystem_config.era_chain_id, Some(true))
    } else {
        (L2ChainId::from(args.chain_id), None)
    };
    println!(
        "ecosystem_config.list_of_chains() after: {:?}",
        ecosystem_config.list_of_chains()
    );
    let zksync_os_genesis_template = if vm_option.is_zksync_os() {
        Some(read_zksync_os_genesis_template(shell, ecosystem_config)?)
    } else {
        None
    };
    let has_evm_emulation_support = if let Some(template) = &zksync_os_genesis_template {
        template.get("evm_emulator_hash").is_some()
    } else {
        let genesis_config_path = ecosystem_config.default_genesis_path(vm_option);
        let default_genesis_config = GenesisConfig::read(shell, &genesis_config_path).await?;
        default_genesis_config.evm_emulator_hash()?.is_some()
    };
    if args.evm_emulator && !has_evm_emulation_support {
        anyhow::bail!(MSG_EVM_EMULATOR_HASH_MISSING_ERR);
    }

    let chain_config = ChainConfig::new(
        internal_id,
        default_chain_name.clone(),
        chain_id,
        args.prover_version,
        ecosystem_config.l1_network,
        chain_path.clone(),
        ecosystem_config.link_to_code(),
        ecosystem_config.get_chain_rocks_db_path(&default_chain_name),
        ecosystem_config.get_chain_artifacts_path(&default_chain_name),
        chain_configs_path.clone(),
        None,
        args.l1_batch_commit_data_generator_mode,
        args.base_token,
        args.wallet_creation,
        OnceCell::from(shell.clone()),
        legacy_bridge,
        args.evm_emulator,
        args.tight_ports,
        vm_option,
        Some(SourceFiles {
            contracts_path: ecosystem_config.contracts_path_for_ctm(args.vm_option),
            default_configs_path: ecosystem_config.default_configs_path_for_ctm(args.vm_option),
        }),
    );

    create_wallets(
        shell,
        &chain_config.configs,
        &ecosystem_config.link_to_code(),
        internal_id,
        args.wallet_creation,
        args.wallet_path,
    )?;

    if let Some(template) = zksync_os_genesis_template {
        let genesis_root = template
            .get("genesis_root")
            .cloned()
            .context("zkOS genesis template is missing `genesis_root`")?;
        let mut genesis = serde_json::Map::new();
        genesis.insert("genesis_root".to_string(), genesis_root);
        genesis.insert(
            "l1_chain_id".to_string(),
            json!(ecosystem_config.l1_network.chain_id()),
        );
        genesis.insert(
            "l2_chain_id".to_string(),
            json!(chain_config.chain_id.as_u64()),
        );
        genesis.insert(
            "l1_batch_commit_data_generator_mode".to_string(),
            serde_json::to_value(args.l1_batch_commit_data_generator_mode)?,
        );
        genesis.insert(
            "custom_genesis_state_path".to_string(),
            serde_json::Value::Null,
        );
        if let Some(hash) = template.get("evm_emulator_hash").cloned() {
            genesis.insert("evm_emulator_hash".to_string(), hash);
        }
        fs::write(
            chain_config.path_to_genesis_config(),
            serde_json::to_string_pretty(&serde_json::Value::Object(genesis))?,
        )?;
    }

    chain_config.save_with_base_path(shell, chain_path)?;
    Ok(())
}

fn read_zksync_os_genesis_template(
    shell: &Shell,
    ecosystem_config: &EcosystemConfig,
) -> anyhow::Result<serde_json::Value> {
    let primary_path =
        ecosystem_config.default_genesis_path(zkstack_cli_types::VMOption::ZKSyncOsVM);
    let fallback_path = ecosystem_config
        .link_to_code()
        .parent()
        .context("link_to_code must have a parent directory")?
        .join("zksync-os-server")
        .join("local-chains")
        .join("v30.2")
        .join("genesis.json");
    let path = if shell.path_exists(&primary_path) {
        primary_path
    } else {
        fallback_path
    };
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed reading zkOS genesis template at `{path:?}`"))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed deserializing zkOS genesis template at `{path:?}`"))
}
