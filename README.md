# Fluxus Source Demo

A template project for creating custom Fluxus source components, demonstrating the implementation of a basic data source with extensible features.

## Overview

fluxus-source-demo is a Rust library template that showcases how to build a Fluxus source component. It provides a foundation for implementing custom data sources with various input methods and processing capabilities.

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
fluxus-source-demo = "0.1"
```

## Usage

### Basic Source Implementation

```rust
use fluxus_source_demo::DemoSource;
use fluxus::sources::Source;

#[tokio::main]
async fn main() {
    // Create a new demo source
    let mut source = DemoSource::new(100);
    
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