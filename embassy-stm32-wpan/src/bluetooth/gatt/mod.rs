//! GATT (Generic Attribute Profile) implementation for WBA BLE stack
//!
//! This module provides a Rust wrapper around ST's GATT implementation,
//! including server functionality and event handling.

pub mod events;
pub mod client;
pub mod server;
pub mod types;

pub use events::{
    CHAR_CCCD_HANDLE_OFFSET, CHAR_VALUE_HANDLE_OFFSET, CccdValue, GattClientEvent, GattEvent, aci_event_code,
    client_events_from_vendor_event, from_vendor_event, is_cccd_handle, is_value_handle,
};
pub use client::GattClient;
pub use server::GattServer;
pub use types::{
    CharProperties, CharacteristicHandle, GattEventMask, SecurityPermissions, ServiceHandle, ServiceType, Uuid,
    UuidType,
};
