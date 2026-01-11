//! GATT (Generic Attribute Profile) implementation for WBA BLE stack
//!
//! This module provides a Rust wrapper around ST's GATT implementation.
//! For Phase 1, this is a thin wrapper that uses ST's C GATT functions directly.

pub mod server;
pub mod types;

pub use server::GattServer;
pub use types::{
    CharProperties, CharacteristicHandle, GattEventMask, SecurityPermissions, ServiceHandle, ServiceType, Uuid,
    UuidType,
};
