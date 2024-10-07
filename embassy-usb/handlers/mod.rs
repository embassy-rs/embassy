//! Implementations of common Host-side drivers

use core::num::NonZeroU8;

use crate::host::UsbHost;
pub mod hub;

pub struct DeviceFilter {
    base_class: Option<NonZeroU8>, // 0 would mean it's defined on an interface-level
    sub_class: Option<u8>,
    protocol: Option<u8>,
}

pub struct StaticHandlerSpec {
    device: Option<DeviceFilter>,
}

/// The base functionality for a host-side usb driver
///
/// In order to speed up driver detection when large amount of drivers may be supported
/// we support a declaritive filter with `static_spec`, this allows a user to build a hash-table, b-tree or similar structure
/// which can be traversed efficiently upon enumeration.
pub trait UsbHostHandler {
    const fn static_spec() -> StaticHandlerSpec;

    /// Attempts to build a driver from a enumerated device
    /// It's expected that the device has been given a unique address through the enumeration process
    ///
    /// Typically this is implemented by retrieving the (interface) descriptors and sifting through those
    /// if a valid configuration is found the device is configured using `SET_CONFIGURATION`;
    /// finally the appropriate channels are allocated andn stored in the resulting handler
    ///
    /// NOTE: Channels are expected to self-clean on `Drop`. FIXME: this is not the case for stm32
    async fn try_register(bus: &mut UsbHost, device_address: u8) -> Result<Self, ()>;
}

/// An extension to UsbHostHandler allowing the handler to be suspended in order to handle other devices (by dropping channels)
///
/// This should be implementable for most handlers however depending on the amount of channels
/// used it might not be worth implementing for all handlers.
pub trait UsbResumableHandler: UsbHostHandler {
    type UsbResumeInfo;

    /// In theory this doesn't need to be async, but a implementor might desire to run some checks upon resuming
    async fn try_resume(bus: &mut UsbHost, resume_info: Self::UsbResumeInfo) -> Result<Self, ()>;

    // Consumes `Self` to gain resume info, this prevents any duplication
    async fn try_suspend(self, bus: &mut UsbHost) -> Self::UsbResumeInfo;
}
