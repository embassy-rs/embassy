//! HCI (Host Controller Interface) layer for STM32WBA BLE stack
//!
//! This module provides the low-level interface between the BLE host and controller.
//! Unlike WB55 which uses IPCC for inter-processor communication, WBA uses direct
//! C function calls since it's a single-core architecture.

pub mod command;
pub mod event;
pub mod host_if;
pub mod types;

pub use command::CommandSender;
pub use event::{Event, EventCode, EventParams, read_event};
pub use host_if::HostStack_Process;
pub use types::*;
