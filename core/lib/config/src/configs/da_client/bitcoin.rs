use smart_config::{de::FromSecretString, DescribeConfig, DeserializeConfig};

#[derive(Clone, Debug, PartialEq, DescribeConfig, DeserializeConfig)]
pub struct BitcoinConfig {
    pub api_node_url: String,
    pub poda_url: String,
}

#[derive(Clone, Debug, DescribeConfig, DeserializeConfig)]
pub struct BitcoinSecrets {
    #[config(with = Optional(FromSecretString))]
    pub rpc_user: String,
    #[config(with = Optional(FromSecretString))]
    pub rpc_password: String,
}
