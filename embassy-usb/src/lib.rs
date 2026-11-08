#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

/// Get max value in const context.
macro_rules! const_max {
    ($first:expr $(, $next:expr)* $(,)?) => {{
        let mut max = $first;
        $(
            if max < $next {
                max = $next;
            }
        )*
        max
    }};
}

pub use embassy_usb_driver as driver;

pub mod class;
pub mod control;
pub mod descriptor;
pub mod device;
pub mod types;

/// USB host support.
pub mod host;

/// Milliseconds a new connection must stay stable.
pub(crate) const DEVICE_DEBOUNCE_STABLE: u64 = 100;
/// Milliseconds before giving up on a bouncing connection.
pub(crate) const DEVICE_DEBOUNCE_TIMEOUT: u64 = 2000;

mod config {
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}

pub use device::{
    Builder, CONFIGURATION_NONE, CONFIGURATION_VALUE, Config, FunctionBuilder, Handler, InterfaceAltBuilder,
    InterfaceBuilder, RemoteWakeupError, UsbBufferReport, UsbDevice, UsbDeviceSpeed, UsbDeviceState, UsbVersion, msos,
};
