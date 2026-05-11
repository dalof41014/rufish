//! # rufish
//!
//! An asynchronous Redfish client library for BMC/server management.
//!
//! ## Quick Start
//!
//! ```no_run
//! use rufish::RedfishClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = RedfishClient::new("10.0.0.5", "admin", "password")?;
//!     client.login().await?;
//!
//!     let root = client.get_service_root().await?;
//!     println!("Redfish version: {:?}", root.redfish_version);
//!
//!     let systems = client.list_systems().await?;
//!     println!("Systems: {:?}", systems.members_count);
//!
//!     client.logout().await?;
//!     Ok(())
//! }
//! ```

mod client;
mod error;
mod types;

pub use client::RedfishClient;
pub use error::{RedfishError, Result};
pub use types::*;
