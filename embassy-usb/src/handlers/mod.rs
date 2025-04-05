//! Implementations of common Host-side drivers

use core::num::NonZeroU8;

use embassy_usb_driver::{
    host::{
        channel::{self, Direction, IsIn, IsOut},
        HostError, UsbChannel, UsbHostDriver,
    },
    Speed,
};

use crate::host::descriptor::{ConfigurationDescriptor, DeviceDescriptor, USBDescriptor};
use crate::host::ControlChannelExt;

pub mod hub;
pub mod kbd;

pub struct DeviceFilter {
    /// Device base-class, 0 would mean it's defined on an interface-level
    pub base_class: Option<NonZeroU8>,
    pub sub_class: Option<u8>,
    pub protocol: Option<u8>,
}

pub struct StaticHandlerSpec {
    /// A non-exaustive filter for devices; the final filter is done inside try_register
    pub device_filter: Option<DeviceFilter>,
}

impl StaticHandlerSpec {
    pub fn new(device_filter: Option<DeviceFilter>) -> Self {
        StaticHandlerSpec { device_filter }
    }
}

/// Information obtained through preliminary enumeration, required for further configuration
#[derive(Debug)]
pub struct EnumerationInfo {
    /// Device address
    pub device_address: u8,
    /// Used to indicate a low-speed device over a full-speed or higher interface
    pub ls_over_fs: bool,
    /// Negotiated speed of the device
    pub speed: Speed,
    /// Device Specs
    pub device_desc: DeviceDescriptor,
}

impl EnumerationInfo {
    /// Retrieves active device configuration or sets the default if not yet configured
    ///
    /// A USB device can only have a single configuration active, this method ensures any previously
    ///  configured mode is maintained for interface/endpoint configuration
    pub async fn active_config_or_set_default<'a, D: IsIn + IsOut, C: UsbChannel<channel::Control, D>>(
        &self,
        channel: &mut C,
        cfg_desc_buf: &'a mut [u8],
    ) -> Result<ConfigurationDescriptor<'a>, HostError> {
        // FIXME: We can't just call `get_active_configuration`, as of writing this is an unfortunate limitation of the borrow checker (see https://users.rust-lang.org/t/yet-another-returning-this-value-requires-that-x-is-borrowed-for-a/112604/2)
        Ok(match channel.active_configuration_value().await? {
            Some(_) => self.get_active_configuration(channel, cfg_desc_buf).await?.unwrap(),
            None => {
                // No active configuration, set default
                let default_cfg = self.get_configuration(0, channel, cfg_desc_buf).await?;
                default_cfg.set_configuration(channel).await?;
                default_cfg
            }
        })
    }

    /// Retrieves the active device configuration, `None` if currently no configuration is active.
    pub async fn get_active_configuration<'a, D: IsIn, C: UsbChannel<channel::Control, D>>(
        &self,
        channel: &mut C,
        cfg_desc_buf: &'a mut [u8],
    ) -> Result<Option<ConfigurationDescriptor<'a>>, HostError> {
        let cfg_id = channel.active_configuration_value().await?;

        let cfg_id = match cfg_id {
            Some(v) => v.into(),
            None => return Ok(None),
        };

        let mut index = None;
        for i in 0..self.device_desc.num_configurations {
            let cfg_desc_short = channel
                .request_descriptor::<ConfigurationDescriptor, { ConfigurationDescriptor::SIZE }>(i, false)
                .await?;

            if cfg_desc_short.configuration_value == cfg_id {
                if cfg_desc_short.total_len as usize > cfg_desc_buf.len() {
                    return Err(HostError::InsufficientMemory);
                }

                index.replace(i);
                break;
            }
        }

        let index = index.ok_or(HostError::Other(
            "Active Configuration not found on device, bad device?",
        ))?;

        channel
            .request_descriptor_bytes::<ConfigurationDescriptor>(index, cfg_desc_buf)
            .await?;

        let cfg_desc =
            ConfigurationDescriptor::try_from_slice(cfg_desc_buf).map_err(|_| HostError::InvalidDescriptor)?;

        Ok(Some(cfg_desc))
    }

    /// Retrieve a device configuration by index up to a max of [`DeviceDescriptor::num_configurations`]
    pub async fn get_configuration<'a, D: channel::IsIn, C: UsbChannel<channel::Control, D>>(
        &self,
        index: u8,
        channel: &mut C,
        cfg_desc_buf: &'a mut [u8],
    ) -> Result<ConfigurationDescriptor<'a>, HostError> {
        if index >= self.device_desc.num_configurations {
            return Err(HostError::InvalidDescriptor);
        }

        let cfg_desc_short = channel
            .request_descriptor::<ConfigurationDescriptor, { ConfigurationDescriptor::SIZE }>(index, false)
            .await?;

        let total_len = cfg_desc_short.total_len as usize;
        if total_len > cfg_desc_buf.len() {
            return Err(HostError::InsufficientMemory);
        }
        let dest_buffer = &mut cfg_desc_buf[0..total_len];

        channel
            .request_descriptor_bytes::<ConfigurationDescriptor>(index, dest_buffer)
            .await?;

        trace!(
            "Full Configuration Descriptor [{}]: {:?}",
            cfg_desc_short.total_len,
            dest_buffer
        );

        let cfg_desc =
            ConfigurationDescriptor::try_from_slice(dest_buffer).map_err(|_| HostError::InvalidDescriptor)?;

        Ok(cfg_desc)
    }
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
    InvalidDescriptor,
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
    async fn try_register(bus: &Self::Driver, enum_info: &EnumerationInfo) -> Result<Self, RegisterError>;

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
