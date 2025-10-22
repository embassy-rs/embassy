//! Direct Memory Access (DMA) driver.

#[cfg_attr(feature = "lpc55-core0", path = "./dma/lpc55.rs")]
mod inner;
pub use inner::*;
