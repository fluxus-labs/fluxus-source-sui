# Fluxus Sui Source

A Rust library for integrating Sui blockchain data into the Fluxus framework, providing real-time access to Sui network transactions and events.

## Overview

fluxus-source-sui is a specialized Rust library that enables seamless integration with the Sui blockchain network. It provides a robust foundation for building data processing applications that require real-time access to Sui blockchain transactions and events.

## Features

- **Sui Network Integration**: Direct connection to Sui blockchain networks (Mainnet, Testnet, Devnet).
- **Transaction Streaming**: Real-time streaming of Sui blockchain transactions with configurable batch sizes.
- **Flexible Configuration**: Customizable polling intervals and transaction batch sizes.
- **Efficient Data Processing**: Optimized for handling high-throughput blockchain data streams.
- **Seamless Framework Integration**: Built for smooth integration with the Fluxus data processing framework.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fluxus-source-sui = "0.1"
```

## Usage

### Basic Source Implementation

```rust
use fluxus_source_sui::SuiSource;
use fluxus::sources::Source;

#[tokio::main]
async fn main() {
    // Create a new Sui source with 500ms polling interval and batch size of 10
    let mut source = SuiSource::new_with_mainnet(500, 10);
    
    // Initialize the source
    source.init().await.unwrap();
    
    // Process transactions
    while let Ok(Some(transaction)) = source.next().await {
        println!("Transaction: {:?}", transaction);
    }
}
```

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.