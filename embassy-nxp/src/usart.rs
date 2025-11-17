#![macro_use]

//! Universal Synchronous/Asynchronous Receiver/Transmitter (USART) driver.

#[cfg_attr(feature = "lpc55-core0", path = "./usart/lpc55.rs")]
mod inner;
pub use inner::*;
