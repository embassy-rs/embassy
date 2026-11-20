//! HCI (Host Controller Interface) layer for STM32WBA BLE stack
//!
//! This module provides the low-level interface between the BLE host and controller.
//! Commands are declared by `stm32wb-hci` (vendor ACI) and `bt-hci` (Bluetooth Core)
//! and executed through the crate's [`ControllerAdapter`](crate::controller::ControllerAdapter).

pub mod types;

pub use types::*;
