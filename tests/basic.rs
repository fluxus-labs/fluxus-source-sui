use fluxus::sources::Source;
use fluxus_source_sui::EventSource;

#[tokio::test]
async fn test_basic() {
    let testnet_uri = "https://fullnode.testnet.sui.io:443";
    let mut source = EventSource::new(testnet_uri).unwrap();

    source.init().await.unwrap();

    while let Ok(Some(data)) = source.next().await {
        println!("Data: {:?}", data);
    }
}
