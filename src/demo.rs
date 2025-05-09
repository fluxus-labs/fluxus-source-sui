use async_trait::async_trait;
use fluxus::sources::Source;
use fluxus::utils::models::{Record, StreamResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DemoEvent {
    pub id: String,
    pub event_type: String,
    pub timestamp: u64,
    pub value: f64,
    pub metadata: String,
}

/// A demo source that generates simulated events
pub struct DemoSource {
    /// Interval between events in milliseconds
    interval: Duration,
    /// Counter for generating unique event IDs
    counter: u64,
    /// Whether the source has been initialized
    initialized: bool,
}

impl DemoSource {
    /// Create a new DemoSource with a specified interval between events
    pub fn new(interval_ms: u64) -> Self {
        Self {
            interval: Duration::from_millis(interval_ms),
            counter: 0,
            initialized: false,
        }
    }

    /// Create a demo event with simulated data
    fn generate_event(&mut self) -> DemoEvent {
        let event_types = ["create", "update", "delete"];
        let event_type = event_types[self.counter as usize % event_types.len()].to_string();

        let event = DemoEvent {
            id: format!("{}", self.counter),
            event_type,
            timestamp: self.counter,
            value: (self.counter as f64).sin() * 100.0,
            metadata: format!("Event metadata {}", self.counter),
        };

        self.counter += 1;
        event
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[async_trait]
impl Source<DemoEvent> for DemoSource {
    async fn init(&mut self) -> StreamResult<()> {
        if self.initialized {
            return Ok(());
        }
        self.initialized = true;
        Ok(())
    }

    async fn next(&mut self) -> StreamResult<Option<Record<DemoEvent>>> {
        // Simulate processing delay
        sleep(self.interval).await;

        // Generate and return a new event
        let event = self.generate_event();
        tracing::info!("Generated event: {:?}", event);

        Ok(if self.counter > 10 {
            None
        } else {
            Some(Record::new(event))
        })
    }

    async fn close(&mut self) -> StreamResult<()> {
        self.initialized = false;
        Ok(())
    }
}
