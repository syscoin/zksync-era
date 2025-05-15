use serde::Deserialize;

// SYSCOIN
use crate::{AvailConfig, BitcoinConfig, CelestiaConfig, EigenConfig, ObjectStoreConfig};

pub mod avail;
pub mod celestia;
pub mod eigen;
// SYSCOIN
pub mod bitcoin;

pub const AVAIL_CLIENT_CONFIG_NAME: &str = "Avail";
pub const CELESTIA_CLIENT_CONFIG_NAME: &str = "Celestia";
pub const EIGEN_CLIENT_CONFIG_NAME: &str = "Eigen";
pub const OBJECT_STORE_CLIENT_CONFIG_NAME: &str = "ObjectStore";
pub const NO_DA_CLIENT_CONFIG_NAME: &str = "NoDA";
// SYSCOIN
pub const BITCOIN_CLIENT_CONFIG_NAME: &str = "Bitcoin";

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum DAClientConfig {
    Avail(AvailConfig),
    Celestia(CelestiaConfig),
    Eigen(EigenConfig),
    // SYSCOIN
    Bitcoin(BitcoinConfig),
    ObjectStore(ObjectStoreConfig),
    NoDA,
}

impl From<AvailConfig> for DAClientConfig {
    fn from(config: AvailConfig) -> Self {
        Self::Avail(config)
    }
}
// SYSCOIN
impl From<BitcoinConfig> for DAClientConfig {
    fn from(config: BitcoinConfig) -> Self {
        Self::Bitcoin(config)
    }
}
