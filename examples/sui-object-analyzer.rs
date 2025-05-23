use fluxus::api::{DataStream, io::CollectionSink};
use fluxus::sources::Source;
use fluxus::utils::window::WindowConfig;
use fluxus_source_sui::SuiObjectSource;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt().init();

    // Create a Sui object source using Mainnet, polling every 500ms, monitoring a specific address
    let target_address =
        "0xac5bceec1b789ff840d7d4e6ce4ce61c90d190a7f8c4f4ddf0bff6ee2413c33c".to_string();
    let mut sui_object_source = SuiObjectSource::new_with_mainnet(500, target_address, 10);
    sui_object_source
        .init()
        .await
        .expect("Failed to initialize Sui object source");

    process_stream(sui_object_source).await;
}

async fn process_stream(sui_object_source: SuiObjectSource) {
    // Create a HashMap to store object type counts
    pub type ObjectTypeCount = HashMap<String, u32>;

    // Create a sink to collect results
    let sink: CollectionSink<ObjectTypeCount> = CollectionSink::new();
    let sink_clone = sink.clone();

    // Set a timeout duration for the entire processing
    let timeout_duration = Duration::from_secs(30);
    let start_time = std::time::Instant::now();

    // Process stream with 10-second tumbling window
    let processing = tokio::spawn(async move {
        DataStream::new(sui_object_source)
            .parallel(2)
            .window(WindowConfig::tumbling(Duration::from_secs(10)))
            .aggregate(HashMap::new(), |mut counts, objects| {
                for object in objects {
                    tracing::debug!("Processing object: {:?}", object);
                    *counts.entry(object.object_type).or_insert(0) += 1;
                }
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
            for (object_type, count) in data {
                tracing::info!("{}: {}", object_type, count);
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
