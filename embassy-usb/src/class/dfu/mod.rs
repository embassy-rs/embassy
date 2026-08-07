//! USB Device Firmware Upgrade (DFU) class implementation.
//!
//! This module provides USB DFU 1.1 protocol support, split into two modes:
//! - `app_mode`: Runtime mode for applications to support detach requests
//! - `dfu_mode`: Bootloader mode for handling firmware downloads

pub mod consts;

/// DFU runtime mode (application side).
pub mod app_mode;
/// DFU bootloader mode (firmware download).
pub mod dfu_mode;
