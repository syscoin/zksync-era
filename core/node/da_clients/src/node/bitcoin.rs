use zksync_config::configs::da_client::bitcoin::{BitcoinConfig, BitcoinSecrets};
use zksync_da_client::DataAvailabilityClient;
use zksync_node_framework::{
    wiring_layer::{WiringError, WiringLayer},
    FromContext,
};

use crate::bitcoin::BitcoinDAClient;

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
pub struct Input {}

#[async_trait::async_trait]
impl WiringLayer for BitcoinWiringLayer {
    type Input = Input;
    type Output = Box<dyn DataAvailabilityClient>;

    fn layer_name(&self) -> &'static str {
        "bitcoin_client_layer"
    }

    async fn wire(self, _input: Self::Input) -> Result<Self::Output, WiringError> {
        let client: Box<dyn DataAvailabilityClient> =
            Box::new(BitcoinDAClient::new(self.config, self.secrets)?);
        Ok(client)
    }
}
