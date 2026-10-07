mod asynch;
mod blocking;

pub use asynch::{FirmwareState, FirmwareUpdater};
pub use blocking::{BlockingFirmwareState, BlockingFirmwareUpdater};
use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

use crate::verification::VerificationError;

/// Firmware updater flash configuration holding the two flashes used by the updater
///
/// If only a single flash is actually used, then that flash should be partitioned into two partitions before use.
/// The easiest way to do this is to use [`FirmwareUpdaterConfig::from_linkerfile`] or [`FirmwareUpdaterConfig::from_linkerfile_blocking`] which will partition
/// the provided flash according to symbols defined in the linkerfile.
pub struct FirmwareUpdaterConfig<DFU, STATE> {
    /// The dfu flash partition
    pub dfu: DFU,
    /// The state flash partition
    pub state: STATE,
}

/// Errors returned by FirmwareUpdater
#[derive(Debug)]
pub enum FirmwareUpdaterError {
    /// Error from flash.
    Flash(NorFlashErrorKind),
    /// Crypto errors.
    Verification(VerificationError),
    /// Bad state.
    BadState,
}

#[cfg(feature = "defmt")]
impl defmt::Format for FirmwareUpdaterError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            FirmwareUpdaterError::Flash(_) => defmt::write!(fmt, "FirmwareUpdaterError::Flash(_)"),
            FirmwareUpdaterError::Verification(_) => defmt::write!(fmt, "FirmwareUpdaterError::Verification(_)"),
            FirmwareUpdaterError::BadState => defmt::write!(fmt, "FirmwareUpdaterError::BadState"),
        }
    }
}

impl<E> From<E> for FirmwareUpdaterError
where
    E: NorFlashError,
{
    fn from(error: E) -> Self {
        FirmwareUpdaterError::Flash(error.kind())
    }
}

impl From<VerificationError> for FirmwareUpdaterError {
    fn from(error: VerificationError) -> Self {
        FirmwareUpdaterError::Verification(error)
    }
}
