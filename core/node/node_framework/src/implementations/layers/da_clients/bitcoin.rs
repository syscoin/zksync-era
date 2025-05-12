use zksync_config::{configs::da_client::bitcoin::BitcoinSecrets, BitcoinConfig};
use zksync_da_client::DataAvailabilityClient;
use zksync_da_clients::bitcoin::BitcoinDAClient;
use zksync_node_framework_derive::FromContext;

use crate::{
    implementations::resources::da_client::DAClientResource,
    wiring_layer::{WiringError, WiringLayer},
    IntoContext,
};

#[derive(Debug)]
pub struct BitcoinWiringLayer {
    config: BitcoinConfig,
    secrets: BitcoinSecrets,
}

impl BitcoinWiringLayer {
    pub fn new(config: BitcoinConfig, secrets: BitcoinSecrets) -> Self {
        Self { config, secrets }
    }
}

#[derive(Debug, FromContext)]
#[context(crate = crate)]
pub struct Input {}

#[derive(Debug, IntoContext)]
#[context(crate = crate)]
pub struct Output {
    pub client: DAClientResource,
}

#[async_trait::async_trait]
impl WiringLayer for BitcoinWiringLayer {
    type Input = Input;
    type Output = Output;

    fn layer_name(&self) -> &'static str {
        "bitcoin_client_layer"
    }

    async fn wire(self, _input: Self::Input) -> Result<Self::Output, WiringError> {
        let client: Box<dyn DataAvailabilityClient> = Box::new(BitcoinDAClient::new(
            self.config,
            self.secrets,
        )?);

        Ok(Self::Output {
            client: DAClientResource(client),
        })
    }
}
