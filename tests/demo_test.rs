use fluxus::sources::Source;
use fluxus_source_demo::DemoSource;
use tokio::test;

#[test]
async fn test_demo_source_creation() {
    // Test basic source creation
    let mut source = DemoSource::new(100);
    let _ = source.init().await;
    assert!(
        source.is_initialized(),
        "DemoSource should be created successfully"
    );
}

#[test]
async fn test_demo_source_initialization() {
    // Test source initialization
    let mut source = DemoSource::new(100);
    let init_result = source.init().await;
    assert!(init_result.is_ok(), "Source should initialize successfully");
}

#[test]
async fn test_demo_source_data_generation() {
    // Test data generation
    let mut source = DemoSource::new(10);
    source.init().await.unwrap();

    let mut count = 0;
    while let Ok(Some(_)) = source.next().await {
        count += 1;
        if count >= 10 {
            break;
        }
    }

    assert_eq!(
        count, 10,
        "Should generate the specified number of data items"
    );
}
