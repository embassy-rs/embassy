//! Hash Accelerator (HASH)
#[cfg_attr(hash_v1, path = "v1.rs")]
#[cfg_attr(hash_v2, path = "v2v3.rs")]
#[cfg_attr(hash_v3, path = "v2v3.rs")]
mod _version;

pub use _version::*;
