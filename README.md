# Fluxus Sui Source

A Rust library for integrating Sui blockchain data into the Fluxus framework, providing real-time access to Sui network transactions, events, and objects.

## Overview

fluxus-source-sui is a specialized Rust library that enables seamless integration with the Sui blockchain network. It provides a robust foundation for building data processing applications that require real-time access to Sui blockchain transactions, events, and objects.

## Features

- **Sui Network Integration**: Direct connection to Sui blockchain networks (Mainnet, Testnet, Devnet).
- **Transaction Streaming**: Real-time streaming of Sui blockchain transactions with configurable batch sizes.
- **Event Monitoring**: Real-time streaming of Sui blockchain events.
- **Object Tracking**: Monitor changes to Sui objects owned by specific addresses.
- **Flexible Configuration**: Customizable polling intervals and batch sizes.
- **Efficient Data Processing**: Optimized for handling high-throughput blockchain data streams.
- **Seamless Framework Integration**: Built for smooth integration with the Fluxus data processing framework.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fluxus-source-sui = "0.1"
```

## Usage

### Transaction Source

```rust
use fluxus_source_sui::SuiTransactionSource;
use fluxus::sources::Source;

#[tokio::main]
async fn main() {
    // Create a new transaction source with 500ms polling interval and batch size of 10
    let mut source = SuiTransactionSource::new_with_mainnet(500, 10);
    
    // Initialize the source
    source.init().await.unwrap();
    
    // Process transactions
    while let Ok(Some(transaction)) = source.next().await {
        println!("Transaction: {:?}", transaction);
    }
}
```

### Event Source

```rust
use fluxus_source_sui::SuiEventSource;
use fluxus::sources::Source;

#[tokio::main]
async fn main() {
    // Create a new event source with 1s polling interval and batch size of 50
    let mut source = SuiEventSource::new_with_mainnet(1000, 50);
    
    // Initialize the source
    source.init().await.unwrap();
    
    // Process events
    while let Ok(Some(event)) = source.next().await {
        println!("Event: {:?}", event);
    }
}
```

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.