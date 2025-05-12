use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bitcoin_da_client::{SyscoinClient, MAX_BLOB_SIZE};
use serde::{Deserialize, Serialize};
use zksync_config::configs::da_client::bitcoin::{
    BitcoinConfig as BitcoinServerConfig, BitcoinSecrets,
};
use zksync_da_client::{
    types,
    types::{ClientType, DAError, DispatchResponse, InclusionData},
    DataAvailabilityClient,
};

use crate::utils::{to_non_retriable_da_error, to_retriable_da_error};
use hex::FromHex;

#[derive(Clone, Deserialize, Serialize)]
struct RPCError {
    code: i32,
    message: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct CreateBlobResponse {
    error: Option<RPCError>,
    result: Option<BlobResult>, // Made Option to handle potential errors better
}

#[derive(Clone, Deserialize, Serialize)]
struct BlobResult {
    versionhash: String,
}

// BitcoinDAClient with Arc-wrapped client for thread-safety
pub struct BitcoinDAClient {
    client: Arc<SyscoinClient>,
    // Store these for potential cloning
    api_node_url: String,
    rpc_user: String,
    rpc_password: String,
    poda_url: String,
}

impl BitcoinDAClient {
    pub fn new(config: BitcoinServerConfig, secrets: BitcoinSecrets) -> Result<Self> {
        let client = SyscoinClient::new(
            &config.api_node_url,
            &secrets.rpc_user,
            &secrets.rpc_password,
            &config.poda_url,
            None,
        )
        .map_err(|e| anyhow!("Failed to create SyscoinClient: {}", e))?;

        Ok(Self {
            client: Arc::new(client),
            api_node_url: config.api_node_url.clone(),
            rpc_user: secrets.rpc_user.clone(),
            rpc_password: secrets.rpc_password.clone(),
            poda_url: config.poda_url.clone(),
        })
    }
}

// Manual impl for Debug
impl Debug for BitcoinDAClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitcoinDAClient")
            .field("api_node_url", &self.api_node_url)
            .field("poda_url", &self.poda_url)
            .finish_non_exhaustive()
    }
}

// Now clone is simple because we're using Arc
impl Clone for BitcoinDAClient {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            api_node_url: self.api_node_url.clone(),
            rpc_user: self.rpc_user.clone(),
            rpc_password: self.rpc_password.clone(),
            poda_url: self.poda_url.clone(),
        }
    }
}

#[async_trait]
impl DataAvailabilityClient for BitcoinDAClient {
    async fn dispatch_blob(
        &self,
        _batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, DAError> {
        // Check for non-retriable errors first (client-side validation)
        let size_limit = MAX_BLOB_SIZE;
        if data.is_empty() {
            return Err(to_non_retriable_da_error(anyhow!(
                "Cannot dispatch empty blob"
            )));
        }
        if data.len() > size_limit {
            return Err(to_non_retriable_da_error(anyhow!(
                "Blob size {} exceeds the maximum limit of {} bytes",
                data.len(),
                size_limit
            )));
        }

        // Server-side errors are generally retriable (might be transient)
        match self.client.create_blob(&data).await {
            Ok(blob_id) => Ok(DispatchResponse {
                request_id: blob_id,
            }),
            Err(e) => Err(to_retriable_da_error(anyhow!("{}", e))),
        }
    }

    async fn get_inclusion_data(&self, blob_id: &str) -> Result<Option<InclusionData>, DAError> {
        // Invalid blob_id format would be non-retriable
        let blob_id_clean = blob_id.strip_prefix("0x").unwrap_or(blob_id);
        if blob_id_clean.len() != 64 || !blob_id_clean.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(to_non_retriable_da_error(anyhow!(
                "Invalid blob ID format: expected 32-byte hex string"
            )));
        }

        // We don't need the raw blob here; the L1 validator expects the 32-byte hash itself.
        let bytes: Vec<u8> = Vec::from_hex(blob_id_clean).map_err(|e| {
            to_non_retriable_da_error(anyhow!("Failed to decode blob ID hex: {}", e))
        })?;

        Ok(Some(InclusionData { data: bytes }))
    }

    async fn ensure_finality(
        &self,
        dispatch_request_id: String,
    ) -> Result<Option<types::FinalityResponse>, DAError> {
        match self.client.check_blob_finality(&dispatch_request_id).await {
            Ok(true) => {
                // Blob exists and is final
                Ok(Some(types::FinalityResponse {
                    blob_id: dispatch_request_id,
                }))
            }
            Ok(false) => {
                // Blob exists but not yet final
                Ok(None)
            }
            Err(e) => Err(to_retriable_da_error(anyhow!("Failed to verify finality: {}", e))),
        }
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) -> Option<usize> {
        Some(MAX_BLOB_SIZE)
    }

    fn client_type(&self) -> ClientType {
        ClientType::Bitcoin
    }

    async fn balance(&self) -> Result<u64, DAError> {
        match self.client.get_balance().await {
            Ok(balance) => Ok(balance as u64),
            Err(e) => Err(to_retriable_da_error(anyhow!("{}", e))),
        }
    }
}
