//! Utilities related to flash.

mod concat_flash;
#[cfg(test)]
pub(crate) mod mem_flash;
pub mod partition;

pub use concat_flash::ConcatFlash;
