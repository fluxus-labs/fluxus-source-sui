//! Fluxus Sui Source
//!
//! A Rust library for integrating Sui blockchain data into the Fluxus framework,
//! providing real-time access to Sui network transactions and events.
//!
//! ## Features
//!
//! - **Sui Network Integration**: Direct connection to Sui blockchain networks (Mainnet, Testnet, Devnet).
//! - **Transaction Streaming**: Real-time streaming of Sui blockchain transactions with configurable batch sizes.
//! - **Flexible Configuration**: Customizable polling intervals and transaction batch sizes.
//! - **Efficient Data Processing**: Optimized for handling high-throughput blockchain data streams.
//! - **Seamless Framework Integration**: Built for smooth integration with the Fluxus data processing framework.
//!
//! ## Usage Examples
//!
//! ### Basic Source Implementation
//! ```rust,no_run
//! use fluxus_source_sui::SuiSource;
//! use fluxus::sources::Source;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a new Sui source with 500ms polling interval and batch size of 10
//!     let mut source = SuiSource::new_with_mainnet(500, 10);
//!     
//!     // Initialize the source
//!     source.init().await.unwrap();
//!     
//!     // Process transactions
//!     while let Ok(Some(transaction)) = source.next().await {
//!         println!("Transaction: {:?}", transaction);
//!     }
//! }
//! ```

mod sui;

pub use sui::*;
