//! Host class driver traits and shared error types.
//!
//! A host class driver does two things:
//!
//! 1. [`HostClass::probe`] inspects the enumerated device and its configuration
//!    descriptor and returns a small, owned `Info` value when the class is
//!    present.
//! 2. [`HostClass::open`] allocates the pipes described by `Info` and performs
//!    any class-specific setup (for example `SET_INTERFACE`).
//!
//! The split keeps matching synchronous and side-effect free while allowing
//! opening to be async.

use embassy_usb_driver::host::{HostError, PipeError, UsbHostAllocator};

pub use super::descriptor::{ClassCode, ClassMatch};
use super::handler::EnumerationInfo;

/// Shared error type for host class drivers.
///
/// Class-specific error enums can wrap this type or convert into it. New code
/// should prefer returning `ClassError` directly so applications can handle
/// the common cases uniformly.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClassError {
    /// A pipe transfer failed.
    Transfer(PipeError),
    /// The class is not present in the configuration descriptor.
    NoMatch,
    /// No free pipe was available for one of the endpoints.
    NoPipe,
    /// A descriptor or a class response was malformed.
    InvalidDescriptor,
    /// The device returned a well-formed but unexpected response.
    InvalidResponse,
    /// A control request was not acknowledged by the device.
    RequestFailed,
    /// The class or a requested feature is not supported.
    Unsupported(&'static str),
}

impl From<PipeError> for ClassError {
    fn from(e: PipeError) -> Self {
        Self::Transfer(e)
    }
}

impl From<HostError> for ClassError {
    fn from(e: HostError) -> Self {
        match e {
            HostError::PipeError(e) => Self::Transfer(e),
            HostError::RequestFailed => Self::RequestFailed,
            HostError::InvalidDescriptor => Self::InvalidDescriptor,
            HostError::OutOfPipes => Self::NoPipe,
            HostError::OutOfSlots => Self::Unsupported("no free device slot"),
            HostError::NoSuchDevice => Self::Unsupported("device not found"),
            HostError::InsufficientMemory => Self::Unsupported("insufficient memory"),
            HostError::Other(message) => Self::Unsupported(message),
            // `HostError` is non-exhaustive.
            _ => Self::Unsupported("host error"),
        }
    }
}

impl core::fmt::Display for ClassError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(e) => write!(f, "transfer error: {e:?}"),
            Self::NoMatch => write!(f, "class not found in configuration"),
            Self::NoPipe => write!(f, "no free host pipe"),
            Self::InvalidDescriptor => write!(f, "invalid descriptor"),
            Self::InvalidResponse => write!(f, "invalid class response"),
            Self::RequestFailed => write!(f, "control request failed"),
            Self::Unsupported(what) => write!(f, "unsupported: {what}"),
        }
    }
}

impl core::error::Error for ClassError {}

#[cfg(feature = "embedded-io")]
impl embedded_io_async::Error for ClassError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        use embedded_io_async::ErrorKind;

        match self {
            Self::Transfer(e) => match e {
                PipeError::Disconnected => ErrorKind::NotConnected,
                PipeError::BufferOverflow => ErrorKind::OutOfMemory,
                PipeError::Timeout => ErrorKind::TimedOut,
                _ => ErrorKind::Other,
            },
            Self::NoMatch => ErrorKind::NotFound,
            Self::NoPipe => ErrorKind::OutOfMemory,
            Self::InvalidDescriptor | Self::InvalidResponse => ErrorKind::InvalidData,
            Self::RequestFailed => ErrorKind::Other,
            Self::Unsupported(_) => ErrorKind::Unsupported,
        }
    }
}

/// A host class driver that can be opened from an enumerated device.
///
/// The trait is intentionally not dyn-compatible: [`open`](Self::open) is async
/// and generic over the pipe allocator. Use static dispatch; a feature-gated
/// `AnyClass` enum is a possible future extension for dynamic binding.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a USB host class driver",
    label = "not a host class driver",
    note = "implement `embassy_usb::host::class::HostClass` for this type"
)]
pub trait HostClass<'d, A: UsbHostAllocator<'d>>: Sized {
    /// Human-readable class name, used for logging.
    const NAME: &'static str;

    /// Cheap, owned description of the interface(s) and endpoints to use.
    ///
    /// This must not borrow from the configuration descriptor buffer, so that
    /// it can outlive the probe call and be passed to [`open`](Self::open).
    type Info: Copy;

    /// Find the class in an enumerated device. Returns `None` when the class is
    /// not present. This must not allocate pipes or issue control requests.
    ///
    /// `cfg` is the full configuration descriptor returned by the device and
    /// `enum_info` carries the device descriptor and bus routing information.
    fn probe(enum_info: &EnumerationInfo, cfg: &[u8]) -> Option<Self::Info>;

    /// Allocate pipes and perform any class-specific setup.
    async fn open(alloc: &A, cfg: &[u8], info: Self::Info, enum_info: &EnumerationInfo) -> Result<Self, ClassError>;

    /// Convenience wrapper around [`probe`](Self::probe).
    fn is_present(enum_info: &EnumerationInfo, cfg: &[u8]) -> bool {
        Self::probe(enum_info, cfg).is_some()
    }
}

#[cfg(test)]
mod tests {
    use embassy_usb_driver::host::{HostError, PipeError};

    use super::ClassError;

    #[test]
    fn pipe_errors_convert_to_transfer_errors() {
        assert!(matches!(
            ClassError::from(PipeError::Stall),
            ClassError::Transfer(PipeError::Stall)
        ));
    }

    #[test]
    fn host_errors_map_to_the_shared_variants() {
        assert!(matches!(ClassError::from(HostError::OutOfPipes), ClassError::NoPipe));
        assert!(matches!(
            ClassError::from(HostError::InvalidDescriptor),
            ClassError::InvalidDescriptor
        ));
        assert!(matches!(
            ClassError::from(HostError::RequestFailed),
            ClassError::RequestFailed
        ));
    }
}
