use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use hex::{decode, encode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use zksync_config::configs::da_client::bitcoin::{
    BitcoinConfig as BitcoinServerConfig, BitcoinSecrets,
};
use zksync_da_client::{
    types,
    types::{ClientType, DAError, DispatchResponse, InclusionData},
    DataAvailabilityClient,
};

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

#[derive(Clone)]
pub struct BitcoinClient {
    http_client: Arc<Client>,
    rpc_url: String,
    rpc_user: String,
    rpc_password: String,
}

impl BitcoinClient {
    const MAX_BLOB_SIZE: usize = 2 * 1024 * 1024;

    pub fn new(config: BitcoinServerConfig, secrets: BitcoinSecrets) -> anyhow::Result<Self> {
        Ok(Self {
            http_client: Arc::new(Client::new()),
            rpc_url: config.api_node_url,
            rpc_user: "u".to_string(),
            rpc_password: "p".to_string(),
        })
    }

    async fn call_rpc<T: for<'a> Deserialize<'a>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let body = json!({
            "method": method,
            "params": params,
            "id": "1",
            "jsonrpc": "2.0"
        });

        let response = self
            .http_client
            .post(&self.rpc_url)
            .basic_auth(&self.rpc_user, Some(&self.rpc_password))
            .json(&body)
            .send()
            .await?;

        // Check for HTTP errors first
        response.error_for_status_ref()?;

        let parsed: T = response.json().await?;
        Ok(parsed)
    }
}

#[async_trait]
impl DataAvailabilityClient for BitcoinClient {
    async fn dispatch_blob(
        &self,
        _batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, DAError> {
        if data.len() > BitcoinClient::MAX_BLOB_SIZE {
            return Err(DAError {
                error: anyhow!("Blob size exceeds the maximum limit"),
                is_retriable: false,
            });
        }

        let data_hex = encode(&data);
        let params = json!({ "data": data_hex });

        // Specific error handling for syscoincreatenevmblob
        let rpc_response: CreateBlobResponse = self
            .call_rpc("syscoincreatenevmblob", params)
            .await
            .map_err(|e_anyhow: anyhow::Error| {
                let mut is_retriable = false;
                // Check if the cause was a reqwest error
                if let Some(reqwest_err) = e_anyhow.downcast_ref::<reqwest::Error>() {
                    is_retriable = reqwest_err.is_connect() || reqwest_err.is_timeout();
                }
                // You could add more checks here for other error sources if needed
                // e.g., if e_anyhow.downcast_ref::<MyCustomRetriableError>().is_some() { is_retriable = true; }
                DAError {
                    error: anyhow!("RPC call to syscoincreatenevmblob failed: {}", e_anyhow), // Keep original anyhow error for context
                    is_retriable,
                }
            })?;

        if let Some(err_info) = rpc_response.error {
            return Err(DAError {
                error: anyhow!(
                    "RPC error from syscoincreatenevmblob: code {}, message: {}",
                    err_info.code,
                    err_info.message
                ),
                is_retriable: false,
            });
        }

        match rpc_response.result {
            Some(blob_result) => Ok(DispatchResponse {
                request_id: blob_result.versionhash,
            }),
            None => Err(DAError {
                error: anyhow!("Missing result in syscoincreatenevmblob response"),
                is_retriable: false,
            }),
        }
    }

    async fn get_inclusion_data(
        &self,
        blob_id: &str,
    ) -> anyhow::Result<Option<InclusionData>, DAError> {
        let actual_blob_id = if blob_id.starts_with("0x") {
            &blob_id[2..]
        } else {
            blob_id
        };
        let params = json!({ "versionhash_or_txid": actual_blob_id, "getdata": true });

        let response: Value =
            self.call_rpc("getnevmblobdata", params)
                .await
                .map_err(|e_anyhow: anyhow::Error| {
                    let mut is_retriable = false;
                    // Check if the cause was a reqwest error
                    if let Some(reqwest_err) = e_anyhow.downcast_ref::<reqwest::Error>() {
                        is_retriable = reqwest_err.is_connect() || reqwest_err.is_timeout();
                    }
                    // You could add more checks here for other error sources if needed
                    // e.g., if e_anyhow.downcast_ref::<MyCustomRetriableError>().is_some() { is_retriable = true; }
                    DAError {
                        error: anyhow!("RPC call to getnevmblobdata failed: {}", e_anyhow), // Keep original anyhow error for context
                        is_retriable,
                    }
                })?;

        if let Some(error_val) = response.get("error") {
            if !error_val.is_null() {
                // Check if error object is present and not null
                let code = error_val.get("code").and_then(Value::as_i64).unwrap_or(0);
                let message = error_val
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Unknown RPC error");
                // Handle specific Syscoin error: "versionhash not found or not a NEVM PoDA blob"
                if message.contains("Could not find blob information for versionhash") {
                    return Ok(None); // Treat as blob not found, not an error to retry indefinitely
                }
                return Err(DAError {
                    error: anyhow!(
                        "RPC error from getnevmblobdata: code {}, message: {}",
                        code,
                        message
                    ),
                    is_retriable: false,
                });
            }
        }

        let data_string = response
            .get("result")
            .and_then(|r| r.get("data"))
            .and_then(Value::as_str);
        match data_string {
            Some(s) => {
                let bytes = decode(s).map_err(|e| DAError {
                    error: anyhow!("Failed to decode blob data from hex: {}", e),
                    is_retriable: false,
                })?;
                Ok(Some(InclusionData { data: bytes }))
            }
            None => Ok(None), // If data is not found or result is missing, treat as blob not found (or not yet available)
        }
    }

    async fn ensure_finality(
        &self,
        dispatch_request_id: String,
    ) -> Result<Option<types::FinalityResponse>, types::DAError> {
        // TODO: Implement actual finality check with Bitcoin/Syscoin
        tracing::info!(
            "Simulating ensure_finality for Bitcoin: request_id = {}",
            dispatch_request_id
        );
        Ok(Some(types::FinalityResponse {
            blob_id: dispatch_request_id,
        }))
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) -> Option<usize> {
        Some(BitcoinClient::MAX_BLOB_SIZE)
    }

    fn client_type(&self) -> ClientType {
        ClientType::Bitcoin
    }

    async fn balance(&self) -> Result<u64, types::DAError> {
        // TODO: Implement balance check if applicable for Bitcoin operator
        tracing::info!("Simulating balance check for Bitcoin. Returning 0.");
        Ok(0)
    }
}

impl Debug for BitcoinClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitcoinClient")
            .field("rpc_url", &self.rpc_url)
            .field("rpc_user", &self.rpc_user)
            .field("rpc_password", &self.rpc_password)
            .finish_non_exhaustive()
    }
}
