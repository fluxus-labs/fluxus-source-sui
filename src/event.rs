use async_trait::async_trait;
use fluxus::sources::Source;
use fluxus::utils::models::{Record, StreamError, StreamResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use sui_sdk::rpc_types::EventFilter;
use sui_sdk::types::event::EventID;
use sui_sdk::{SUI_MAINNET_URL, SuiClient, SuiClientBuilder};
use tokio::time::sleep;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainEvent {
    /// Event ID
    pub id: EventID,
    /// Package ID
    pub package_id: String,
    /// Module name
    pub module_name: String,
    /// Event type
    pub event_type: String,
    /// Sender address
    pub sender: String,
    /// Event data
    pub data: String,
    /// Timestamp
    pub timestamp: u64,
}

/// Sui blockchain data source for fetching event data from the Sui network
pub struct SuiEventSource {
    /// Sui RPC endpoint URL
    rpc_url: String,
    /// Polling interval (milliseconds)
    interval: Duration,
    /// Whether initialized
    initialized: bool,
    /// Sui client
    client: Option<SuiClient>,
    /// Last processed event ID
    last_processed_event_id: Option<String>,
    /// Event query filter
    query: EventFilter,
    /// Cursor for pagination
    cursor: Option<EventID>,
    /// Whether to fetch transactions in descending order
    descending_order: bool,
    /// Maximum number of events to fetch
    max_events: usize,
}

impl SuiEventSource {
    /// Creates a new SuiEventSource instance
    ///
    /// # Parameters
    /// * `rpc_url` - Sui RPC endpoint URL
    /// * `interval_ms` - Polling interval in milliseconds
    /// * `max_events` - Maximum number of events to fetch per poll
    pub fn new(rpc_url: String, interval_ms: u64, max_events: usize) -> Self {
        Self {
            rpc_url,
            interval: Duration::from_millis(interval_ms),
            initialized: false,
            client: None,
            last_processed_event_id: None,
            query: EventFilter::All([]),
            cursor: None,
            descending_order: true,
            max_events,
        }
    }

    /// Creates a new SuiEventSource instance using the default Sui Mainnet RPC endpoint
    pub fn new_with_mainnet(interval_ms: u64, max_events: usize) -> Self {
        Self::new(SUI_MAINNET_URL.to_string(), interval_ms, max_events)
    }

    /// Sets the event query filter
    pub fn with_query(mut self, query: EventFilter) -> Self {
        self.query = query;
        self
    }

    /// Sets the cursor for pagination
    pub fn with_cursor(mut self, cursor: EventID) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[async_trait]
impl Source<Vec<ChainEvent>> for SuiEventSource {
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
        tracing::info!("SuiEventSource initialized with RPC URL: {}", self.rpc_url);

        Ok(())
    }

    async fn next(&mut self) -> StreamResult<Option<Record<Vec<ChainEvent>>>> {
        // Ensure initialized
        if !self.initialized || self.client.is_none() {
            return Err(StreamError::Runtime(
                "SuiEventSource not initialized".to_string(),
            ));
        }

        // Polling interval
        sleep(self.interval).await;

        let client = self.client.as_ref().ok_or_else(|| {
            StreamError::Runtime("SuiEventSource client not available".to_string())
        })?;

        // Query events
        let events = client
            .event_api()
            .query_events(
                self.query.clone(),
                self.cursor,
                Some(self.max_events),
                self.descending_order,
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch events: {}", e);
                StreamError::Runtime(format!("Failed to fetch events: {}", e))
            })?;

        // Return None if no new events
        if events.data.is_empty() {
            tracing::info!("No new events found");
            return Ok(None);
        }

        // Get latest event ID
        let latest_event = events
            .data
            .last()
            .ok_or_else(|| StreamError::Runtime("Failed to get latest event".to_string()))?;
        let latest_event_id = latest_event.id.tx_digest.to_string();

        // Return None if event already processed
        if let Some(last_id) = &self.last_processed_event_id {
            if last_id == &latest_event_id {
                tracing::info!("No new events since last check");
                return Ok(None);
            }
        }

        // Update last processed event ID
        self.last_processed_event_id = Some(latest_event_id);

        // Convert to chain events
        let chain_events: Vec<ChainEvent> = events
            .data
            .into_iter()
            .map(|event| {
                let chain_event = ChainEvent {
                    id: event.id,
                    package_id: event.package_id.to_string(),
                    module_name: event.transaction_module.to_string(),
                    event_type: event.type_.to_string(),
                    sender: event.sender.to_string(),
                    data: format!("{:?}", event.parsed_json),
                    timestamp: event.timestamp_ms.expect("Timestamp not available"),
                };
                tracing::debug!(
                    "Processed Sui event: {} from package: {}",
                    chain_event.id.tx_digest,
                    chain_event.package_id
                );
                chain_event
            })
            .collect();

        Ok(Some(Record::new(chain_events)))
    }

    async fn close(&mut self) -> StreamResult<()> {
        self.initialized = false;
        self.client = None;
        tracing::info!("SuiEventSource closed");
        Ok(())
    }
}
