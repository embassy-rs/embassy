#![macro_use]
//! Direct Memory Access (DMA) driver.

#[cfg_attr(lpc55, path = "./dma/lpc55.rs")]
mod inner;
pub use inner::*;
