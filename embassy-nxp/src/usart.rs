#![macro_use]

//! Universal Synchronous/Asynchronous Receiver/Transmitter (USART) driver.

#[cfg_attr(lpc55, path = "./usart/lpc55.rs")]
mod inner;
pub use inner::*;
