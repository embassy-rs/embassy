//! Module for QSPI device error
use super::traits::{Error, ErrorKind};
use core::fmt::{self, Debug, Display, Formatter};

/// Error type for [`ExclusiveDevice`] operations.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DeviceError<BUS, CS> {
    /// An inner QSPI bus operation failed.
    Qspi(BUS),
    /// Asserting or deasserting CS failed.
    Cs(CS),
}

impl<BUS: Display, CS: Display> Display for DeviceError<BUS, CS> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Qspi(bus) => write!(f, "SPI bus error: {}", bus),
            Self::Cs(cs) => write!(f, "SPI CS error: {}", cs),
        }
    }
}

impl<BUS: Debug + Display, CS: Debug + Display> core::error::Error for DeviceError<BUS, CS> {}

impl<BUS, CS> Error for DeviceError<BUS, CS>
where
    BUS: Error + Debug,
    CS: Debug,
{
    #[inline]
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Qspi(e) => e.kind(),
            Self::Cs(_) => ErrorKind::ChipSelectFault,
        }
    }
}
