use fluxus::sources::Source;
use fluxus_source_sui::SuiObjectSource;
use std::time::Duration;
use sui_sdk::SUI_MAINNET_URL;
use tokio::time::sleep;

const TEST_ADDRESS: &str = "0xac5bceec1b789ff840d7d4e6ce4ce61c90d190a7f8c4f4ddf0bff6ee2413c33c";

#[tokio::test]
async fn test_sui_object_source_initialization() {
    // Create a new SuiObjectSource instance with mainnet configuration
    let mut source = SuiObjectSource::new_with_mainnet(500, TEST_ADDRESS.to_string(), 10);

    // Test initialization
    let init_result = source.init().await;
    assert!(init_result.is_ok(), "Initialization should succeed");
}

#[tokio::test]
async fn test_sui_object_source_custom_endpoint() {
    // Create SuiObjectSource with custom RPC endpoint
    let mut source = SuiObjectSource::new(
        SUI_MAINNET_URL.to_string(),
        500,
        TEST_ADDRESS.to_string(),
        10,
    );

    // Test initialization
    let init_result = source.init().await;
    assert!(
        init_result.is_ok(),
        "Initialization with testnet should succeed"
    );
}

#[tokio::test]
async fn test_sui_object_source_data_fetching() {
    // Create SuiObjectSource instance
    let mut source = SuiObjectSource::new_with_mainnet(500, TEST_ADDRESS.to_string(), 5);

    // Initialize
    source.init().await.expect("Initialization failed");

    // Get first batch of objects
    let result = source.next().await;
    assert!(result.is_ok(), "Fetching object data should succeed");

    if let Ok(Some(object)) = result {
        // Validate basic object fields
        assert!(!object.data.id.is_empty(), "Object ID should not be empty");
        assert!(
            !object.data.object_type.is_empty(),
            "Object type should not be empty"
        );
    }
}

#[tokio::test]
async fn test_sui_object_source_polling_interval() {
    // Create SuiObjectSource with longer polling interval
    let mut source = SuiObjectSource::new_with_mainnet(1000, TEST_ADDRESS.to_string(), 5);
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
async fn test_sui_object_source_batch_size() {
    // Create SuiObjectSource with specified batch size
    let batch_size = 3;
    let mut source = SuiObjectSource::new_with_mainnet(500, TEST_ADDRESS.to_string(), batch_size);
    source.init().await.expect("Initialization failed");

    // Get multiple batches of data
    let mut object_count = 0;
    for _ in 0..5 {
        if let Ok(Some(_)) = source.next().await {
            object_count += 1;
        }
        sleep(Duration::from_millis(100)).await;
    }

    assert!(object_count > 0, "Should successfully fetch object data");
}

#[tokio::test]
async fn test_sui_object_source_error_handling() {
    // Create SuiObjectSource with invalid RPC endpoint
    let mut source = SuiObjectSource::new(
        "http://invalid-endpoint".to_string(),
        500,
        TEST_ADDRESS.to_string(),
        10,
    );

    // Test initialization
    let init_result = source.init().await;
    assert!(init_result.is_err(), "Should fail with invalid endpoint");
}
