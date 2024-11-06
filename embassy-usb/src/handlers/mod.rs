//! Implementations of common Host-side drivers

use core::num::NonZeroU8;

use embassy_usb_driver::{
    host::{channel, HostError, UsbChannel, UsbHostDriver},
    Speed,
};

use crate::host::descriptor::{ConfigurationDescriptor, DeviceDescriptor};

pub mod hub;
pub mod kbd;

pub struct DeviceFilter {
    base_class: Option<NonZeroU8>, // 0 would mean it's defined on an interface-level
    sub_class: Option<u8>,
    protocol: Option<u8>,
}

pub struct StaticHandlerSpec {
    /// A non-exaustive filter for devices; the final filter is done inside try_register
    device_filter: Option<DeviceFilter>,
}

impl StaticHandlerSpec {
    pub fn new(device_filter: Option<DeviceFilter>) -> Self {
        StaticHandlerSpec { device_filter }
    }
}

pub struct EnumerationInfo {
    /// Device address
    pub device_address: u8,
    /// Used to indicate a low-speed device over a full-speed or higher interface
    pub ls_over_fs: bool,
    /// Negotiated speed of the device
    pub speed: Speed,
    // Device Specs
    pub device_desc: DeviceDescriptor,
    /// Device Configuration
    pub cfg_desc: ConfigurationDescriptor,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HandlerEvent<T> {
    NoChange,
    HandlerDisconnected,
    HandlerEvent(T),
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RegisterError {
    NoSupportedInterface,
    HostError(HostError),
}

impl From<HostError> for RegisterError {
    fn from(value: HostError) -> Self {
        RegisterError::HostError(value)
    }
}

/// The base functionality for a host-side usb driver
///
/// In order to speed up driver detection when large amount of drivers may be supported
/// we support a declaritive filter with `static_spec`, this allows a user to build a hash-table, b-tree or similar structure
/// which can be traversed efficiently upon enumeration.
pub trait UsbHostHandler: Sized {
    type Driver: UsbHostDriver;
    type PollEvent;

    /// A static specification of the handler
    fn static_spec() -> StaticHandlerSpec;

    /// Attempts to build a driver from a enumerated device
    /// It's expected that the device has been given a unique address through the enumeration process
    ///
    /// Typically this is implemented by retrieving the (interface) descriptors and sifting through those
    /// if a valid configuration is found the device is configured using `SET_CONFIGURATION`;
    /// finally the appropriate channels are allocated andn stored in the resulting handler
    ///
    /// NOTE: Channels are expected to self-clean on `Drop`. FIXME: this is not the case for stm32
    async fn try_register(bus: &Self::Driver, enum_info: EnumerationInfo) -> Result<Self, RegisterError>;

    /// Wait for changes to handler, the handler is expected to defer (`yield_now` or Timer::after) whenever idle.
    /// Handler users should not use `select` or `join` to avoid dropping futures.
    ///
    /// Events are handler-specific and may provide anything in their `HandlerEvent`
    ///  drivers that deal with sub-devices (e.g. Hubs) may use [`HandlerEvent::HandlerDisconnected`] to indicate all sub-devices
    ///  have also disconnected
    async fn wait_for_event(&mut self) -> Result<HandlerEvent<Self::PollEvent>, HostError>;
}

/// An extension to UsbHostHandler allowing the handler to be suspended in order to handle other devices (by dropping channels)
///
/// This should be implementable for most handlers however depending on the amount of channels
/// used it might not be worth implementing for all handlers.
pub trait UsbResumableHandler: UsbHostHandler {
    type UsbResumeInfo;

    /// In theory this doesn't need to be async, but a implementor might desire to run some checks upon resuming
    async fn try_resume(bus: &Self::Driver, resume_info: Self::UsbResumeInfo) -> Result<Self, ()>;

    // Consumes `Self` to gain resume info, this prevents any duplication
    async fn try_suspend(self, bus: &mut Self::Driver) -> Self::UsbResumeInfo;
}
