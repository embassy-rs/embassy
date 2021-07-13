#![macro_use]

#[cfg_attr(dma_v1, path = "v1.rs")]
#[cfg_attr(dma_v2, path = "v2.rs")]
mod _version;

#[cfg(dma)]
#[allow(unused)]
pub use _version::*;
