use fluxus::api::{DataStream, io::CollectionSink};
use fluxus::sources::Source;
use fluxus::utils::window::WindowConfig;
use fluxus_source_sui::SuiEventSource;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt().init();

    // Create a Sui event source using Mainnet, polling every 500ms, fetching max 10 events
    let mut sui_event_source = SuiEventSource::new_with_mainnet(500, 10);
    sui_event_source
        .init()
        .await
        .expect("Failed to initialize Sui event source");

    process_stream(sui_event_source).await;
}

async fn process_stream(sui_event_source: SuiEventSource) {
    // Create a HashMap to store event type counts
    pub type EventTypeCount = HashMap<String, u32>;

    // Create a sink to collect results
    let sink: CollectionSink<EventTypeCount> = CollectionSink::new();
    let sink_clone = sink.clone();

    // Set a timeout duration for the entire processing
    let timeout_duration = Duration::from_secs(30);
    let start_time = std::time::Instant::now();

    // Process stream with 10-second tumbling window
    let processing = tokio::spawn(async move {
        DataStream::new(sui_event_source)
            .parallel(2)
            .window(WindowConfig::tumbling(Duration::from_secs(10)))
            .aggregate(HashMap::new(), |mut counts, event| {
                tracing::debug!("Processing event: {:?}", event);
                *counts.entry(event.event_type).or_insert(0) += 1;
                counts
            })
            .sink(sink_clone)
            .await
            .expect("Stream processing failed");
    });

    let mut i = 0;
    // Wait for either timeout or data collection
    loop {
        if start_time.elapsed() >= timeout_duration {
            tracing::info!(
                "Processing timeout reached after {} seconds",
                timeout_duration.as_secs()
            );
            break;
        }

        // Check for data every second
        tokio::time::sleep(Duration::from_secs(1)).await;
        if let Some(data) = sink.get_last_element() {
            // Print results for each window
            for (event_type, count) in data {
                tracing::info!("{}: {}", event_type, count);
            }

            if i == 10 {
                break;
            }
        }
        i += 1;
    }

    // Cleanup
    processing.abort();
}
