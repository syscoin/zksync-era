use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BitcoinConfig {
    pub api_node_url: String,
    pub poda_url: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BitcoinSecrets {
    pub rpc_user: String,
    pub rpc_password: String,
}
