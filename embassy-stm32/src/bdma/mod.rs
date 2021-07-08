#![macro_use]

#[cfg_attr(bdma_v1, path = "v1.rs")]
#[cfg_attr(bdma_v2, path = "v2.rs")]
mod _version;

#[allow(unused)]
pub use _version::*;
