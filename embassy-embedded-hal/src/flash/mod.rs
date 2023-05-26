//! Utilities related to flash.

mod concat_flash;
#[cfg(test)]
pub(crate) mod mem_flash;
#[cfg(feature = "nightly")]
mod partition;

pub use concat_flash::ConcatFlash;
#[cfg(feature = "nightly")]
pub use partition::Partition;
