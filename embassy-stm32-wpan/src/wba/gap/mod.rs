//! GAP (Generic Access Profile) implementation for WBA BLE stack
//!
//! This module provides high-level APIs for:
//! - Advertising (broadcaster/peripheral role)
//! - Scanning (observer role)
//! - Connection management (central/peripheral roles)

pub mod advertiser;
pub mod connection;
pub mod scanner;
pub mod types;

pub use advertiser::Advertiser;
pub use connection::{
    Connection, ConnectionInitParams, ConnectionManager, ConnectionParams, ConnectionRole, DisconnectReason, GapEvent,
    LePhy, MAX_CONNECTIONS,
};
pub use scanner::{ParsedAdvData, ScanFilterPolicy, ScanParams, ScanType, Scanner};
pub use types::{AdvData, AdvParams, AdvType, OwnAddressType};
