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
            max_events,
        }
    }

    /// Creates a new SuiEventSource instance using the default Sui Mainnet RPC endpoint
    pub fn new_with_mainnet(interval_ms: u64, max_events: usize) -> Self {
        Self::new(SUI_MAINNET_URL.to_string(), interval_ms, max_events)
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[async_trait]
impl Source<ChainEvent> for SuiEventSource {
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
                StreamError::Runtime(e.to_string())
            })?;

        self.client = Some(client);
        self.initialized = true;
        tracing::info!("SuiEventSource initialized with RPC URL: {}", self.rpc_url);

        Ok(())
    }

    async fn next(&mut self) -> StreamResult<Option<Record<ChainEvent>>> {
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
            .query_events(EventFilter::All([]), None, Some(self.max_events), false)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch events: {}", e);
                StreamError::Runtime(e.to_string())
            })?;

        // Return None if no new events
        if events.data.is_empty() {
            tracing::info!("No new events found");
            return Ok(None);
        }

        // Get latest event
        let latest_event = events
            .data
            .first()
            .ok_or_else(|| StreamError::Runtime("Failed to get first event".to_string()))?;

        // Return None if event already processed
        if let Some(last_id) = &self.last_processed_event_id {
            if last_id == &latest_event.id.tx_digest.to_string() {
                tracing::info!("No new events since last check");
                return Ok(None);
            }
        }

        // Update last processed event ID
        self.last_processed_event_id = Some(latest_event.id.tx_digest.to_string());

        // Convert to chain event
        let event = ChainEvent {
            id: latest_event.id,
            package_id: latest_event.package_id.to_string(),
            module_name: latest_event.transaction_module.to_string(),
            event_type: latest_event.type_.to_string(),
            sender: latest_event.sender.to_string(),
            data: format!("{:?}", latest_event.parsed_json),
            timestamp: latest_event.timestamp_ms.expect("Timestamp not available"),
        };

        tracing::info!(
            "Processed Sui event: {} from package: {}",
            event.id.tx_digest,
            event.package_id
        );

        Ok(Some(Record::new(event)))
    }

    async fn close(&mut self) -> StreamResult<()> {
        self.initialized = false;
        self.client = None;
        tracing::info!("SuiEventSource closed");
        Ok(())
    }
}
