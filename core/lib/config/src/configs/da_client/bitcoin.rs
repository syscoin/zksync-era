use serde::Deserialize;
use zksync_basic_types::secrets::PrivateKey;

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct BitcoinDAConfig {
    pub api_node_url: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BitcoinDASecrets {
    pub private_key: PrivateKey,
}
