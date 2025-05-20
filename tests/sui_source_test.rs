use fluxus::sources::Source;
use fluxus_source_sui::SuiSource;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_sui_source_initialization() {
    // Create a new SuiSource instance with mainnet configuration
    let mut source = SuiSource::new_with_mainnet(500, 10);

    // Test initialization
    let init_result = source.init().await;
    assert!(init_result.is_ok(), "Initialization should succeed");
}

#[tokio::test]
async fn test_sui_source_custom_endpoint() {
    // Create SuiSource with custom RPC endpoint
    let mut source = SuiSource::new("https://fullnode.testnet.sui.io:443".to_string(), 500, 10);

    // Test initialization
    let init_result = source.init().await;
    assert!(
        init_result.is_ok(),
        "Initialization with testnet should succeed"
    );
}

#[tokio::test]
async fn test_sui_source_data_fetching() {
    // Create SuiSource instance
    let mut source = SuiSource::new_with_mainnet(500, 5);

    // Initialize
    source.init().await.expect("Initialization failed");

    // Get first batch of transactions
    let result = source.next().await;
    assert!(result.is_ok(), "Fetching transaction data should succeed");

    if let Ok(Some(transaction)) = result {
        // Validate basic transaction fields
        assert!(
            !transaction.data.transaction_digest.is_empty(),
            "Transaction ID should not be empty"
        );
        assert!(
            !transaction.data.sender.is_empty(),
            "Sender address should not be empty"
        );
    }
}

#[tokio::test]
async fn test_sui_source_polling_interval() {
    // Create SuiSource with longer polling interval
    let mut source = SuiSource::new_with_mainnet(1000, 5);
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
async fn test_sui_source_batch_size() {
    // Create SuiSource with specified batch size
    let batch_size = 3;
    let mut source = SuiSource::new_with_mainnet(500, batch_size);
    source.init().await.expect("Initialization failed");

    // Get multiple batches of data
    let mut transaction_count = 0;
    for _ in 0..5 {
        if let Ok(Some(_)) = source.next().await {
            transaction_count += 1;
        }
        sleep(Duration::from_millis(100)).await;
    }

    assert!(
        transaction_count > 0,
        "Should successfully fetch transaction data"
    );
}

#[tokio::test]
async fn test_sui_source_error_handling() {
    // Create SuiSource with invalid RPC endpoint
    let mut source = SuiSource::new("https://invalid.endpoint.example.com".to_string(), 500, 10);

    // Initialization should fail
    let init_result = source.init().await;
    assert!(
        init_result.is_err(),
        "Initialization with invalid endpoint should fail"
    );
}
