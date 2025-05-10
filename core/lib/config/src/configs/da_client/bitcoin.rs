use serde::Deserialize;
use zksync_basic_types::secrets::PrivateKey;

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct BitcoinConfig {
    pub api_node_url: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BitcoinSecrets {
    pub private_key: PrivateKey,
}
