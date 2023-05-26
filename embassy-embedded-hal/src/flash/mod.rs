//! Utilities related to flash.

mod concat_flash;
#[cfg(test)]
pub(crate) mod mem_flash;
mod partition;

pub use concat_flash::ConcatFlash;
pub use partition::Partition;
