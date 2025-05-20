use async_trait::async_trait;
use fluxus::sources::Source;
use fluxus::utils::models::{Record, StreamResult};
use sui_types::event::Event;
use sui_types::full_checkpoint_content::CheckpointData;
use sui_types::messages_checkpoint::CheckpointSequenceNumber;

pub struct EventSource {
    uri: http::Uri,
    checkpoint: Option<CheckpointSequenceNumber>,
    events: Vec<Event>,
    current_index: usize,
}

impl EventSource {
    pub fn new<T>(uri: T) -> Option<Self>
    where
        T: TryInto<http::Uri>,
    {
        let uri = uri.try_into().ok()?;
        Some(Self {
            uri,
            checkpoint: None,
            events: Vec::new(),
            current_index: 0,
        })
    }

    pub fn set_checkpoint(&mut self, checkpoint: CheckpointSequenceNumber) -> StreamResult<()> {
        self.checkpoint = Some(checkpoint);
        Ok(())
    }
}

async fn get_events(
    sui_client: sui_rpc_api::Client,
    checkpoint: Option<CheckpointSequenceNumber>,
) -> StreamResult<Vec<Event>> {
    let cp = get_full_checkpoint(sui_client, checkpoint).await?;
    let events = get_events_from_checkpoint(cp);
    tracing::info!("Retrieved {} events from checkpoint", events.len());
    Ok(events)
}

async fn get_full_checkpoint(
    sui_client: sui_rpc_api::Client,
    checkpoint: Option<CheckpointSequenceNumber>,
) -> StreamResult<CheckpointData> {
    let checkpoint_number = if let Some(cp) = checkpoint {
        cp
    } else {
        let latest_checkpoint = sui_client
            .get_latest_checkpoint()
            .await
            .map_err(|e| {
                fluxus::utils::models::StreamError::Io(std::io::Error::other(format!(
                    "Failed to get latest checkpoint: {}",
                    e
                )))
            })?
            .into_data();

        latest_checkpoint.sequence_number
    };

    tracing::info!("Fetching checkpoint: {}", checkpoint_number);

    let checkpoint = sui_client
        .get_full_checkpoint(checkpoint_number)
        .await
        .map_err(|e| {
            fluxus::utils::models::StreamError::Io(std::io::Error::other(format!(
                "Failed to get full checkpoint: {}",
                e
            )))
        })?;

    Ok(checkpoint)
}

fn get_events_from_checkpoint(checkpoint: CheckpointData) -> Vec<Event> {
    let mut es = Vec::new();
    for txs in checkpoint.transactions {
        if let Some(mut events) = txs.events {
            es.append(&mut events.data);
        }
    }

    es
}

#[async_trait]
impl Source<Event> for EventSource {
    async fn init(&mut self) -> StreamResult<()> {
        let sui_client = sui_rpc_api::Client::new(self.uri.clone()).map_err(|e| {
            fluxus::utils::models::StreamError::Io(std::io::Error::other(format!(
                "Failed to create Sui client: {}",
                e
            )))
        })?;

        self.events = get_events(sui_client.clone(), self.checkpoint).await?;
        self.current_index = 0;

        Ok(())
    }

    async fn next(&mut self) -> StreamResult<Option<Record<Event>>> {
        if self.current_index >= self.events.len() {
            return Ok(None);
        }

        let event = self.events[self.current_index].clone();
        self.current_index += 1;

        Ok(Some(Record::new(event)))
    }

    async fn close(&mut self) -> StreamResult<()> {
        self.events.clear();
        self.current_index = 0;

        Ok(())
    }
}
