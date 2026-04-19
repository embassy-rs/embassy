//! USB host device enumeration helpers.
#![allow(missing_docs)]

use embassy_usb_driver::Speed;
use embassy_usb_driver::host::channel::{self, IsIn, IsOut};
use embassy_usb_driver::host::{HostError, SplitInfo, UsbChannel};

use crate::control::ControlChannelExt;
use crate::descriptor::{ConfigurationDescriptor, DeviceDescriptor, USBDescriptor};

/// Information obtained through preliminary enumeration.
#[derive(Clone, Copy, Debug)]
pub struct EnumerationInfo {
    /// Assigned device address.
    pub device_address: u8,
    /// Split-transaction routing, when this device is behind a hub that
    /// requires splits (HS host reaching LS/FS device through a HS hub, or
    /// FS host reaching LS device through a FS hub). `None` for a device
    /// attached directly to the host at its native speed.
    pub split: Option<SplitInfo>,
    /// Negotiated speed.
    pub speed: Speed,
    /// Parsed device descriptor.
    pub device_desc: DeviceDescriptor,
}

impl EnumerationInfo {
    /// Retrieves the active device configuration, or sets the default if none is active.
    pub async fn active_config_or_set_default<'a, D: IsIn + IsOut, C: UsbChannel<channel::Control, D>>(
        &self,
        channel: &mut C,
        cfg_desc_buf: &'a mut [u8],
    ) -> Result<ConfigurationDescriptor<'a>, HostError> {
        Ok(match channel.active_configuration_value().await? {
            Some(_) => self.get_active_configuration(channel, cfg_desc_buf).await?.unwrap(),
            None => {
                let default_cfg = self.get_configuration(0, channel, cfg_desc_buf).await?;
                channel.set_configuration(default_cfg.configuration_value).await?;
                default_cfg
            }
        })
    }

    /// Retrieves the active device configuration, or `None` if none is active.
    pub async fn get_active_configuration<'a, D: IsIn, C: UsbChannel<channel::Control, D>>(
        &self,
        channel: &mut C,
        cfg_desc_buf: &'a mut [u8],
    ) -> Result<Option<ConfigurationDescriptor<'a>>, HostError> {
        let cfg_id = match channel.active_configuration_value().await? {
            Some(v) => v.into(),
            None => return Ok(None),
        };

        let mut index = None;
        let mut cfg_len = 0;
        for i in 0..self.device_desc.num_configurations {
            let cfg_desc_short = channel
                .request_descriptor::<ConfigurationDescriptor, { ConfigurationDescriptor::SIZE }>(i, false)
                .await?;

            if cfg_desc_short.configuration_value == cfg_id {
                if cfg_desc_short.total_len as usize > cfg_desc_buf.len() {
                    return Err(HostError::InsufficientMemory);
                }
                cfg_len = cfg_desc_short.total_len as usize;
                index.replace(i);
                break;
            }
        }

        let index = index.ok_or(HostError::Other("Active configuration not found on device"))?;
        let dest = &mut cfg_desc_buf[0..cfg_len];
        channel
            .request_descriptor_bytes(ConfigurationDescriptor::DESC_TYPE, index, dest)
            .await?;

        let cfg = ConfigurationDescriptor::try_from_slice(cfg_desc_buf).map_err(|_| HostError::InvalidDescriptor)?;
        Ok(Some(cfg))
    }

    /// Retrieve a device configuration by index.
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
        let dest = &mut cfg_desc_buf[0..total_len];
        channel
            .request_descriptor_bytes(ConfigurationDescriptor::DESC_TYPE, index, dest)
            .await?;

        trace!(
            "Full Configuration Descriptor [{}]: {:?}",
            cfg_desc_short.total_len, dest
        );

        ConfigurationDescriptor::try_from_slice(dest).map_err(|_| HostError::InvalidDescriptor)
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
