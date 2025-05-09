use fluxus::api::{DataStream, io::CollectionSink};
use fluxus::sources::Source;
use fluxus::utils::window::WindowConfig;
use fluxus_source_demo::DemoSource;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt().init();

    // Create a demo source that generates events every 100ms
    let mut demo_source = DemoSource::new(100);
    demo_source
        .init()
        .await
        .expect("Failed to initialize demo source");

    process_stream(demo_source).await;
}

async fn process_stream(demo_source: DemoSource) {
    // Create a HashMap to store event type counts
    pub type EventTypeCount = HashMap<String, u32>;

    // Create a sink to collect the results
    let sink: CollectionSink<EventTypeCount> = CollectionSink::new();

    // Process the stream with a 5-second tumbling window
    DataStream::new(demo_source)
        .parallel(2)
        .window(WindowConfig::tumbling(Duration::from_secs(5)))
        .aggregate(HashMap::new(), |mut counts, event| {
            *counts.entry(event.event_type).or_insert(0) += 1;
            counts
        })
        .sink(sink.clone())
        .await
        .expect("Stream processing failed");

    // Print the results from each window
    for result in sink.get_data() {
        let mut events: Vec<_> = result.iter().collect();
        println!("\nWindow results:");
        events.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        for (event, count) in events {
            println!("{}: {}", event, count);
        }
    }
}
