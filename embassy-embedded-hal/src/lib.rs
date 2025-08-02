#![no_std]
#![allow(async_fn_in_trait)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod adapter;
pub mod flash;
pub mod shared_bus;

pub use embassy_embedded_hal_04::{GetConfig, SetConfig};
