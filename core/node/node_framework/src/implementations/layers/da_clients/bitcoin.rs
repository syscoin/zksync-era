use zksync_config::{configs::da_client::bitcoin::BitcoinSecrets, BitcoinConfig};
use zksync_da_client::DataAvailabilityClient;
use zksync_da_clients::bitcoin::BitcoinClient;
use zksync_node_framework::{
    implementations::resources::da_client::DAClientResource,
    wiring_layer::{WiringError, WiringLayer},
    IntoContext,
};

#[derive(Debug)] // No longer Default
pub struct BitcoinWiringLayer {
    config: BitcoinServerConfig,
    secrets: BitcoinSecrets,
}

impl BitcoinWiringLayer {
    pub fn new(config: BitcoinServerConfig, secrets: BitcoinSecrets) -> Self {
        Self { config, secrets }
    }
}

#[derive(Debug, IntoContext)]
#[context(crate = crate)] // Assuming node_framework is the current crate context for IntoContext proc macro
pub struct Output {
    pub client: DAClientResource,
}

#[async_trait::async_trait]
impl WiringLayer for BitcoinWiringLayer {
    type Input = (); // Assuming no specific input is required from other layers for now
    type Output = Output;

    fn layer_name(&self) -> &'static str {
        "bitcoin_client_layer"
    }

    async fn wire(self, _input: Self::Input) -> Result<Self::Output, WiringError> {
        // Pass the stored config and secrets to the client's new method
        let client_result = BitcoinClient::new(self.config, self.secrets);
        let client: Box<dyn DataAvailabilityClient> =
            Box::new(client_result.map_err(|e| WiringError::Internal(e.into()))?);

        Ok(Self::Output {
            client: DAClientResource(client),
        })
    }
}
