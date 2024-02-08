//! Hash Accelerator (HASH)
#[cfg_attr(hash_v1, path = "v1v3v4.rs")]
#[cfg_attr(hash_v2, path = "v2.rs")]
#[cfg_attr(hash_v3, path = "v1v3v4.rs")]
#[cfg_attr(hash_v4, path = "v1v3v4.rs")]
mod _version;

pub use _version::*;
