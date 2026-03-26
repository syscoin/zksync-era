use std::sync::Arc;

use zksync_config::GenesisConfig;
use zksync_dal::node::{MasterPool, PoolResource};
use zksync_node_framework::{
    wiring_layer::{WiringError, WiringLayer},
    FromContext,
};
/* When you use zkstack chain init (with --no-genesis), genesis is skipped as part of init, before migration to Gateway. At that moment:
 * After you migrate to Gateway and then run zkstack chain genesis server, it now hits the bug.
 * The chain is now on Gateway, but here it needs to call L1 contracts to be able to fetch genesis data.
 */
 // The same layer runs during normal server startup — but it's a no-op after genesis is done.
 // So once genesis is complete (DB has L1 batch #0), every subsequent server start skips all contract calls entirely. The change has no runtime impact on a running chain.
// use zksync_shared_resources::contracts::SettlementLayerContractsResource;
use zksync_shared_resources::contracts::L1ChainContractsResource;
use zksync_web3_decl::client::{DynClient, L1};

use crate::{main_node::MainNodeGenesis, NodeInitializationStrategy};

/// Wiring layer for main node initialization strategy.
#[derive(Debug)]
pub struct MainNodeInitStrategyLayer {
    pub genesis: GenesisConfig,
    pub event_expiration_blocks: u64,
}

#[derive(Debug, FromContext)]
pub struct Input {
    master_pool: PoolResource<MasterPool>,
    l1_client: Box<DynClient<L1>>,
    contracts: L1ChainContractsResource,
}

#[async_trait::async_trait]
impl WiringLayer for MainNodeInitStrategyLayer {
    type Input = Input;
    type Output = NodeInitializationStrategy;

    fn layer_name(&self) -> &'static str {
        "main_node_role_layer"
    }

    async fn wire(self, input: Self::Input) -> Result<Self::Output, WiringError> {
        let pool = input.master_pool.get().await?;
        let genesis = Arc::new(MainNodeGenesis {
            contracts: input.contracts.0,
            genesis: self.genesis,
            event_expiration_blocks: self.event_expiration_blocks,
            l1_client: input.l1_client,
            pool,
        });

        Ok(NodeInitializationStrategy {
            genesis,
            snapshot_recovery: None,
            block_reverter: None,
        })
    }
}
