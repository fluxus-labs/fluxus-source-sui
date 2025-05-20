use fluxus::{
    api::{CollectionSink, DataStream},
    sources::Source,
    utils::window::WindowConfig,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let testnet_uri = "https://fullnode.testnet.sui.io:443";
    let mut sui_source = fluxus_source_sui::EventSource::new(testnet_uri).unwrap();
    // We can use the latest checkpoint by not setting a specific one
    sui_source.init().await.unwrap_or_else(|e| {
        assert!(false, "init failed : {:?}", e);
    });

    process_stream(sui_source).await;
}

async fn process_stream(demo_source: fluxus_source_sui::EventSource) {
    type EventType = u64;
    let sink: CollectionSink<EventType> = CollectionSink::new();

    DataStream::new(demo_source)
        .parallel(2)
        .window(WindowConfig::tumbling(std::time::Duration::from_secs(5)))
        .aggregate(0, |mut counts, _event| {
            counts += 1;
            counts
        })
        .sink(sink.clone())
        .await
        .expect("Stream processing failed");
}
