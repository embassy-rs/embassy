#![macro_use]

#[cfg_attr(eth_v1, path = "v1.rs")]
#[cfg_attr(eth_v2, path = "v2/mod.rs")]
mod _version;

pub use _version::*;
