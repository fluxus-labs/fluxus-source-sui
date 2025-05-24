use fluxus::api::{DataStream, io::CollectionSink};
use fluxus::sources::Source;
use fluxus::utils::window::WindowConfig;
use fluxus_source_sui::SuiTransactionSource;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt().init();

    // Create a Sui data source using Mainnet, polling every 500ms, fetching max 10 transactions
    let mut sui_transaction_source = SuiTransactionSource::new_with_mainnet(500, 10);
    sui_transaction_source
        .init()
        .await
        .expect("Failed to initialize Sui source");

    process_stream(sui_transaction_source).await;
}

async fn process_stream(sui_transaction_source: SuiTransactionSource) {
    // Create a HashMap to store transaction type counts
    pub type TransactionTypeCount = HashMap<String, u32>;

    // Create a sink to collect results
    let sink: CollectionSink<TransactionTypeCount> = CollectionSink::new();
    let sink_clone = sink.clone();

    // Set a timeout duration for the entire processing
    let timeout_duration = Duration::from_secs(30);
    let start_time = std::time::Instant::now();

    // Process stream with 10-second tumbling window
    let processing = tokio::spawn(async move {
        DataStream::new(sui_transaction_source)
            .parallel(2)
            .window(WindowConfig::tumbling(Duration::from_secs(10)))
            .aggregate(HashMap::new(), |mut counts, events| {
                for event in events {
                    tracing::debug!("Processing event: {:?}", event);
                    *counts.entry(event.transaction_type).or_insert(0) += 1;
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
            for (transaction_type, count) in data {
                tracing::info!("{}: {}", transaction_type, count);
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
