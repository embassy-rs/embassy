//! Flash Partition utilities

use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

#[cfg(feature = "nightly")]
mod asynch;
mod blocking;

#[cfg(feature = "nightly")]
pub use asynch::Partition;
pub use blocking::BlockingPartition;

/// Partition error
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<T> {
    /// The requested flash area is outside the partition
    OutOfBounds,
    /// Underlying flash error
    Flash(T),
}

impl<T: NorFlashError> NorFlashError for Error<T> {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Error::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            Error::Flash(f) => f.kind(),
        }
    }
}
