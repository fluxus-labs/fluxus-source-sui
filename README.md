# Fluxus Source Sui

A Fluxus source component for processing and analyzing Sui events streams, providing efficient access to historical Sui event data.

## Overview

fluxus-source-sui is a Rust library that showcases how to build a Fluxus source component for processing and analyzing Sui events streams. It provides a foundation for implementing custom data sources with various input methods and processing capabilities.

## Features

- **Comprehensive Source Structure**: A well - defined implementation structure for building Fluxus data sources.
- **Robust Configuration Handling**: Example code demonstrating how to manage and process configurations.
- **Efficient Asynchronous Processing**: Support for asynchronous message handling, enhancing performance.
- **Reliable Error Management**: Built - in error handling and retry mechanisms to ensure data integrity.
- **Seamless Framework Integration**: Designed for easy integration with the Fluxus framework.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fluxus-source-sui = "0.1"
```

## Usage

### Basic Source Implementation

```rust
use fluxus_source_sui::EventSource;
use fluxus::sources::Source;

#[tokio::main]
async fn main() {
    // Create a new demo source
    let testnet_uri = "https://fullnode.testnet.sui.io:443";
    let mut source = EventSource::new(testnet_uri);

    // Initialize the source
    source.init().await.unwrap();

    // Process data
    while let Ok(Some(data)) = source.next().await {
        println!("Data: {:?}", data);
    }
}
```

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
