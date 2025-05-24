use async_trait::async_trait;
use fluxus::sources::Source;
use fluxus::utils::models::{Record, StreamError, StreamResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use sui_sdk::rpc_types::{SuiTransactionBlockDataAPI, SuiTransactionBlockResponseOptions};
use sui_sdk::rpc_types::{SuiTransactionBlockResponse, SuiTransactionBlockResponseQuery};
use sui_sdk::types::base_types::SuiAddress;
use sui_sdk::types::digests::TransactionDigest;
use sui_sdk::types::messages_checkpoint::CheckpointSequenceNumber;
use sui_sdk::{SUI_MAINNET_URL, SuiClient, SuiClientBuilder};
use tokio::time::sleep;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuiEvent {
    /// Transaction ID
    pub transaction_digest: String,
    /// Transaction type
    pub transaction_type: String,
    /// Timestamp
    pub timestamp: u64,
    /// Sender address
    pub sender: String,
    /// Recipient address (if applicable)
    pub recipient: Option<String>,
    /// Transaction amount (if applicable)
    pub amount: Option<u64>,
    /// Transaction metadata
    pub metadata: String,
}

/// Sui blockchain data source for fetching transaction data from the Sui network
pub struct SuiTransactionSource {
    /// Sui RPC endpoint URL
    rpc_url: String,
    /// Polling interval (milliseconds)
    interval: Duration,
    /// Whether initialized
    initialized: bool,
    /// Sui client
    client: Option<SuiClient>,
    /// Last processed transaction digest
    last_processed_digest: Option<String>,
    /// Last processed checkpoint
    last_processed_checkpoint: Option<CheckpointSequenceNumber>,
    /// Transaction query
    query: SuiTransactionBlockResponseQuery,
    /// Cursor for pagination
    cursor: Option<TransactionDigest>,
    /// Whether to fetch transactions in descending order
    descending_order: bool,
    /// Maximum number of transactions to fetch
    max_transactions: usize,
}

impl SuiTransactionSource {
    /// Creates a new SuiTransactionSource instance
    ///
    /// # Parameters
    /// * `rpc_url` - Sui RPC endpoint URL
    /// * `interval_ms` - Polling interval in milliseconds
    /// * `max_transactions` - Maximum number of transactions to fetch per poll
    pub fn new(rpc_url: String, interval_ms: u64, max_transactions: usize) -> Self {
        // Set transaction query options
        let options = SuiTransactionBlockResponseOptions::new()
            .with_input()
            .with_effects()
            .with_events()
            .with_balance_changes();
        let query = SuiTransactionBlockResponseQuery::new(None, Some(options));
        Self {
            rpc_url,
            interval: Duration::from_millis(interval_ms),
            initialized: false,
            client: None,
            last_processed_digest: None,
            last_processed_checkpoint: None,
            cursor: None,
            query,
            descending_order: true,
            max_transactions,
        }
    }

    /// Creates a new SuiTransactionSource instance using the default Sui Devnet RPC endpoint
    pub fn new_with_mainnet(interval_ms: u64, max_transactions: usize) -> Self {
        Self::new(SUI_MAINNET_URL.to_string(), interval_ms, max_transactions)
    }

    /// Sets the cursor for pagination
    pub fn with_cursor(mut self, cursor: TransactionDigest) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Sets the query for fetching transactions
    pub fn with_query(mut self, query: SuiTransactionBlockResponseQuery) -> Self {
        self.query = query;
        self
    }

    /// Sets the descending order flag
    pub fn with_descending_order(mut self, descending_order: bool) -> Self {
        self.descending_order = descending_order;
        self
    }

    /// Converts SuiTransactionBlockResponse to SuiEvent
    fn transaction_to_event(&self, transaction: SuiTransactionBlockResponse) -> SuiEvent {
        let digest = transaction.digest.to_string();
        let timestamp = transaction.timestamp_ms.unwrap_or(0);

        // Determine transaction type
        let transaction_type = if let Some(kind) = transaction
            .transaction
            .as_ref()
            .map(|tx| tx.data.transaction().name())
        {
            kind.to_string()
        } else {
            "unknown".to_string()
        };

        // Get sender address
        let sender = transaction
            .transaction
            .as_ref()
            .map(|tx| tx.data.sender().as_ref())
            .map(|addr| {
                SuiAddress::try_from(addr)
                    .map_err(|_| "Invalid sender address format")
                    .ok()
                    .map(|addr| addr.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        let metadata = transaction
            .transaction
            .as_ref()
            .map(|tx| format!("{:?}", tx.data))
            .unwrap_or_else(|| "unknown".to_string());

        // Try to extract recipient and amount (if applicable)
        let (recipient, amount) = (None, None);

        SuiEvent {
            transaction_digest: digest,
            transaction_type,
            timestamp,
            sender,
            recipient,
            amount,
            metadata,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[async_trait]
impl Source<SuiEvent> for SuiTransactionSource {
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
        tracing::info!(
            "SuiTransactionSource initialized with RPC URL: {}",
            self.rpc_url
        );

        Ok(())
    }

    async fn next(&mut self) -> StreamResult<Option<Record<SuiEvent>>> {
        // Ensure initialized
        if !self.initialized || self.client.is_none() {
            return Err(StreamError::Runtime(
                "SuiTransactionSource not initialized".to_string(),
            ));
        }

        // Polling interval
        sleep(self.interval).await;

        let client = self.client.as_ref().ok_or_else(|| {
            StreamError::Runtime("SuiTransactionSource client not available".to_string())
        })?;

        // Get recent transactions
        let transactions = client
            .read_api()
            .query_transaction_blocks(
                self.query.clone(),
                self.cursor,
                Some(self.max_transactions),
                self.descending_order,
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch transactions: {}", e);
                StreamError::Runtime(format!("Failed to fetch transactions: {}", e))
            })?;

        // Return None if no new transactions
        if transactions.data.is_empty() {
            tracing::info!("No new transactions found");
            return Ok(None);
        }

        // Get latest transaction
        let latest_transaction = transactions
            .data
            .first()
            .ok_or_else(|| StreamError::Runtime("Failed to get first transaction".to_string()))?;
        let latest_digest = latest_transaction.digest.to_string();

        // Return None if transaction already processed
        if let Some(last_digest) = &self.last_processed_digest {
            if last_digest == &latest_digest {
                tracing::info!("No new transactions since last check");
                return Ok(None);
            }
        }

        // Update last processed digest
        self.last_processed_digest = Some(latest_digest.clone());
        self.last_processed_checkpoint = latest_transaction.checkpoint;

        // Convert to event and return
        let event = self.transaction_to_event(latest_transaction.clone());
        tracing::info!(
            "Processed Sui transaction: {} checkpoint: {:?}",
            latest_digest,
            latest_transaction.checkpoint
        );

        Ok(Some(Record::new(event)))
    }

    async fn close(&mut self) -> StreamResult<()> {
        self.initialized = false;
        self.client = None;
        tracing::info!("SuiTransactionSource closed");
        Ok(())
    }
}
