use fluxus::sources::Source;
use fluxus_source_sui::SuiEventSource;
use std::time::Duration;
use sui_sdk::SUI_TESTNET_URL;
use tokio::time::sleep;

#[tokio::test]
async fn test_sui_event_source_initialization() {
    // Create a new SuiEventSource instance with mainnet configuration
    let mut source = SuiEventSource::new_with_mainnet(500, 10);

    // Test initialization
    let init_result = source.init().await;
    assert!(init_result.is_ok(), "Initialization should succeed");
}

#[tokio::test]
async fn test_sui_event_source_custom_endpoint() {
    // Create SuiEventSource with custom RPC endpoint
    let mut source = SuiEventSource::new(SUI_TESTNET_URL.to_string(), 500, 10);

    // Test initialization
    let init_result = source.init().await;
    assert!(
        init_result.is_ok(),
        "Initialization with testnet should succeed"
    );
}

#[tokio::test]
async fn test_sui_event_source_data_fetching() {
    // Create SuiEventSource instance
    let mut source = SuiEventSource::new_with_mainnet(500, 5);

    // Initialize
    source.init().await.expect("Initialization failed");

    // Get first batch of events
    let result = source.next().await;
    assert!(result.is_ok(), "Fetching event data should succeed");

    if let Ok(Some(event)) = result {
        // Validate basic event fields
        assert!(
            !event.data.id.tx_digest.to_string().is_empty(),
            "Event ID should not be empty"
        );
        assert!(
            !event.data.event_type.is_empty(),
            "Event type should not be empty"
        );
    }
}

#[tokio::test]
async fn test_sui_event_source_polling_interval() {
    // Create SuiEventSource with longer polling interval
    let mut source = SuiEventSource::new_with_mainnet(1000, 5);
    source.init().await.expect("Initialization failed");

    // Record start time
    let start = std::time::Instant::now();

    // Fetch data twice
    let _ = source.next().await;
    let _ = source.next().await;

    // Verify polling interval is respected
    let elapsed = start.elapsed();
    assert!(
        elapsed >= Duration::from_millis(1000),
        "Should respect polling interval"
    );
}

#[tokio::test]
async fn test_sui_event_source_batch_size() {
    // Create SuiEventSource with specified batch size
    let batch_size = 3;
    let mut source = SuiEventSource::new_with_mainnet(500, batch_size);
    source.init().await.expect("Initialization failed");

    // Get multiple batches of data
    let mut event_count = 0;
    for _ in 0..5 {
        if let Ok(Some(_)) = source.next().await {
            event_count += 1;
        }
        sleep(Duration::from_millis(100)).await;
    }

    assert!(event_count > 0, "Should successfully fetch event data");
}

#[tokio::test]
async fn test_sui_event_source_error_handling() {
    // Create SuiEventSource with invalid RPC endpoint
    let mut source = SuiEventSource::new("http://invalid-endpoint".to_string(), 500, 10);

    // Test initialization
    let init_result = source.init().await;
    assert!(init_result.is_err(), "Should fail with invalid endpoint");
}
