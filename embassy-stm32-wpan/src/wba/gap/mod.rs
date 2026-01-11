//! GAP (Generic Access Profile) implementation for WBA BLE stack
//!
//! This module provides high-level APIs for:
//! - Advertising (broadcaster/peripheral role)
//! - Scanning (observer role) - TODO
//! - Connection establishment (central/peripheral) - TODO

pub mod advertiser;
pub mod types;

pub use advertiser::Advertiser;
pub use types::{AdvData, AdvParams, AdvType, OwnAddressType};
