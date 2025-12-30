use async_trait::async_trait;
use fluxus::sources::Source;
use fluxus::utils::models::{Record, StreamError, StreamResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use sui_sdk::rpc_types::{SuiObjectData, SuiObjectDataOptions, SuiObjectResponseQuery};
use sui_sdk::types::base_types::{ObjectID, SuiAddress};
use sui_sdk::{SUI_MAINNET_URL, SuiClient, SuiClientBuilder};
use tokio::time::sleep;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainObject {
    /// Object ID
    pub id: String,
    /// Object type
    pub object_type: String,
    /// Owner address
    pub owner: String,
    /// Object version
    pub version: u64,
    /// Object data
    pub data: SuiObjectData,
    /// Last transaction digest
    pub last_transaction_digest: String,
}

/// Sui blockchain data source for fetching object data from the Sui network
pub struct SuiObjectSource {
    /// Sui RPC endpoint URL
    rpc_url: String,
    /// Polling interval (milliseconds)
    interval: Duration,
    /// Whether initialized
    initialized: bool,
    /// Sui client
    client: Option<SuiClient>,
    /// Target address to monitor
    target_address: String,
    /// Last processed object version map (object_id -> version)
    last_processed_versions: HashMap<String, u64>,
    /// Object query
    query: Option<SuiObjectResponseQuery>,
    /// Cursor for pagination
    cursor: Option<ObjectID>,
    /// Maximum number of objects to fetch
    max_objects: usize,
}

impl SuiObjectSource {
    /// Creates a new SuiObjectSource instance
    ///
    /// # Parameters
    /// * `rpc_url` - Sui RPC endpoint URL
    /// * `interval_ms` - Polling interval in milliseconds
    /// * `target_address` - Target address to monitor objects
    /// * `max_objects` - Maximum number of objects to fetch per poll
    pub fn new(
        rpc_url: String,
        interval_ms: u64,
        target_address: String,
        max_objects: usize,
    ) -> Self {
        let query = SuiObjectResponseQuery::new_with_options(SuiObjectDataOptions::full_content());
        Self {
            rpc_url,
            interval: Duration::from_millis(interval_ms),
            initialized: false,
            client: None,
            target_address,
            last_processed_versions: HashMap::new(),
            query: Some(query),
            cursor: None,
            max_objects,
        }
    }

    /// Creates a new SuiObjectSource instance using the default Sui Mainnet RPC endpoint
    pub fn new_with_mainnet(interval_ms: u64, target_address: String, max_objects: usize) -> Self {
        Self::new(
            SUI_MAINNET_URL.to_string(),
            interval_ms,
            target_address,
            max_objects,
        )
    }

    /// Sets the cursor for pagination
    pub fn with_cursor(mut self, cursor: ObjectID) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Sets the query for object data
    pub fn with_query(mut self, query: SuiObjectResponseQuery) -> Self {
        self.query = Some(query);
        self
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[async_trait]
impl Source<Vec<ChainObject>> for SuiObjectSource {
    async fn init(&mut self) -> StreamResult<()> {
        if self.initialized {
            return Ok(());
        }

        // Initialize Sui client
        let client = SuiClientBuilder::default()
            .build(self.rpc_url.as_str())
            .await
            .map_err(|e| {
                tracing::error!("Failed to initialize Sui client: {}", e);
                StreamError::Runtime(format!("Failed to initialize Sui client: {}", e))
            })?;

        self.client = Some(client);
        self.initialized = true;
        tracing::info!("SuiObjectSource initialized with RPC URL: {}", self.rpc_url);

        Ok(())
    }

    async fn next(&mut self) -> StreamResult<Option<Record<Vec<ChainObject>>>> {
        // Ensure initialized
        if !self.initialized || self.client.is_none() {
            return Err(StreamError::Runtime(
                "SuiObjectSource not initialized".to_string(),
            ));
        }

        // Polling interval
        sleep(self.interval).await;

        let client = self.client.as_ref().ok_or_else(|| {
            StreamError::Runtime("SuiObjectSource client not available".to_string())
        })?;

        // Query objects owned by the target address
        let objects = client
            .read_api()
            .get_owned_objects(
                SuiAddress::from_str(&self.target_address).map_err(|e| {
                    tracing::error!("Invalid target address: {}", e);
                    StreamError::Runtime(format!("Invalid target address: {}", e))
                })?,
                self.query.clone(),
                self.cursor,
                Some(self.max_objects),
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch objects: {}", e);
                StreamError::Runtime(format!("Failed to fetch objects: {}", e))
            })?;

        // Return None if no objects found
        if objects.data.is_empty() {
            tracing::info!("No objects found for address: {}", self.target_address);
            return Ok(None);
        }

        // Process objects with new versions
        let mut chain_objects = Vec::new();
        for object in objects.data {
            let object_data = object.data.ok_or_else(|| {
                tracing::error!("Object data is missing");
                StreamError::Runtime("Object data is missing".to_string())
            })?;

            let object_id = object_data.object_id.to_string();
            let current_version = object_data.version.value();

            // Skip if object version hasn't changed
            if let Some(&last_version) = self.last_processed_versions.get(&object_id)
                && last_version >= current_version
            {
                continue;
            }

            // Update last processed version
            self.last_processed_versions
                .insert(object_id.clone(), current_version);

            // Convert to chain object
            let chain_object = ChainObject {
                id: object_id.clone(),
                object_type: object_data
                    .clone()
                    .type_
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
                owner: self.target_address.clone(),
                version: current_version,
                data: object_data.clone(),
                last_transaction_digest: object_data
                    .previous_transaction
                    .map(|t| t.to_string())
                    .unwrap_or_default(),
            };

            tracing::debug!(
                "Processed Sui object: {} version: {} owner: {}",
                chain_object.id,
                chain_object.version,
                chain_object.owner
            );

            chain_objects.push(chain_object);
        }

        // Return None if no new object versions found
        if chain_objects.is_empty() {
            tracing::info!(
                "No new object versions found for address: {}",
                self.target_address
            );
            return Ok(None);
        }

        Ok(Some(Record::new(chain_objects)))
    }

    async fn close(&mut self) -> StreamResult<()> {
        self.initialized = false;
        self.client = None;
        tracing::info!("SuiObjectSource closed");
        Ok(())
    }
}
