use smart_config::{DescribeConfig, DeserializeConfig};

#[derive(Clone, Debug, PartialEq, DescribeConfig, DeserializeConfig)]
pub struct BitcoinConfig {
    pub api_node_url: String,
    pub poda_url: String,
}

#[derive(Clone, Debug, DescribeConfig, DeserializeConfig)]
pub struct BitcoinSecrets {
    pub rpc_user: String,
    pub rpc_password: String,
}
