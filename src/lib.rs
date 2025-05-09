//! Fluxus Source Demo
//!
//! A template project for creating custom Fluxus source components,
//! demonstrating the implementation of a basic data source with extensible features.
//!
//! ## Features
//!
//! - **Comprehensive Source Structure**: A well - defined implementation structure for building Fluxus data sources.
//! - **Robust Configuration Handling**: Example code demonstrating how to manage and process configurations.
//! - **Efficient Asynchronous Processing**: Support for asynchronous message handling, enhancing performance.
//! - **Reliable Error Management**: Built - in error handling and retry mechanisms to ensure data integrity.
//! - **Seamless Framework Integration**: Designed for easy integration with the Fluxus framework.
//!
//! ## Usage Examples
//!
//! ### Basic Source Implementation
//! ```rust,no_run
//! use fluxus_source_demo::DemoSource;
//! use fluxus::sources::Source;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a new demo source
//!     let mut source = DemoSource::new(100);
//!     
//!     // Initialize the source
//!     source.init().await.unwrap();
//!     
//!     // Process data
//!     while let Ok(Some(data)) = source.next().await {
//!         println!("Data: {:?}", data);
//!     }
//! }
//! ```

mod demo;
pub use demo::*;
