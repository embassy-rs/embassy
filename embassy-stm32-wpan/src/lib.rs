//! The embassy-stm32-wpan crate aims to provide safe use of the commands necessary to interface
//! with the Cortex C0 CPU2 coprocessor of STM32WB microcontrollers. It implements safe wrappers
//! around the Transport Layer, and in particular the system, memory, BLE and Mac channels.
//!
//! # Design
//!
//! This crate loosely follows the Application Note 5289 "How to build wireless applications with
//! STM32WB MCUs"; several of the startup procedures laid out in Annex 14.1 are implemented using
//! inline copies of the code contained within the `stm32wb_copro` C library.
//!
//! BLE commands are implemented via use of the [stm32wb_hci] crate, for which the
//! [stm32wb_hci::Controller] trait has been implemented.

#![no_std]
#![allow(async_fn_in_trait)]
#![allow(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]
// #![warn(missing_docs)]
#![allow(static_mut_refs)] // TODO: Fix

#[cfg(feature = "wb55")]
mod wb55;

#[cfg(feature = "wb55")]
pub use wb55::*;

#[cfg(feature = "wba")]
mod wba;

#[cfg(feature = "wba")]
pub use wba::*;
