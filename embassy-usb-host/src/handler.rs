//! USB host device enumeration helpers.
#![allow(missing_docs)]

use embassy_time::Timer;
use embassy_usb_driver::Speed;
use embassy_usb_driver::host::pipe::{self, IsIn, IsOut};
use embassy_usb_driver::host::{HostError, SplitInfo, SplitSpeed, UsbPipe};

use crate::control::ControlPipeExt;
use crate::descriptor::{ConfigurationDescriptor, ConfigurationDescriptorChain, DeviceDescriptor, USBDescriptor};

/// How many times a descriptor read is re-attempted before its failure is
/// taken as the answer.
pub(crate) const DESCRIPTOR_RETRIES: u32 = 3;

/// Milliseconds between those attempts.
pub(crate) const DESCRIPTOR_RETRY_MS: u64 = 5;

/// Runs a descriptor read, re-attempting it a few times before taking a
/// failure as the answer.
///
/// Some devices STALL a descriptor read once and answer the identical
/// request milliseconds later — nothing about the request differs between
/// the two. Observed here on a USB gamepad (0428:4001), which refuses the
/// first configuration-descriptor read issued after it is addressed.
/// Linux's usbcore makes the same allowance for every descriptor read in
/// `usb_get_descriptor`, commented "some devices are flakey".
///
/// Safe in a way it would not be for a state-changing request: a
/// descriptor read is idempotent, and a control transfer restarts from
/// its SETUP, which clears a control endpoint's protocol stall (USB 2.0
/// §8.5.3.4) — so there is nothing left over from the failed attempt to
/// undo first. `SET_ADDRESS` and `SET_CONFIGURATION` deliberately do not
/// go through here: repeating one of those is not the same as issuing it
/// once.
pub(crate) async fn retry_descriptor<T, E>(mut read: impl AsyncFnMut() -> Result<T, E>) -> Result<T, E> {
    let mut retries = DESCRIPTOR_RETRIES;
    loop {
        match read().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                if retries == 0 {
                    return Err(e);
                }
                retries -= 1;
                Timer::after_millis(DESCRIPTOR_RETRY_MS).await;
            }
        }
    }
}

/// How a device's traffic reaches it on the bus.
///
/// Unifies the device speed with the optional split-transaction (or legacy
/// `PRE` prefix) routing. Illegal combinations (a high-speed device reached
/// through a TT or PRE) are unrepresentable: [`BusRoute::Translated`] wraps
/// a [`SplitInfo`] whose [`SplitSpeed`] only admits low or full speed.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BusRoute {
    /// No bus-level translation. The device runs at its native speed and is
    /// addressed directly. Covers root-port devices and any device whose
    /// traffic is not re-clocked by an intermediate hub (high-speed device
    /// behind a high-speed hub, full-speed device behind a full-speed hub).
    Direct(Speed),

    /// The device is reached through a transaction translator on a
    /// high-speed controller (USB 2.0 §11.14), or via a legacy `PRE` prefix
    /// on a full-speed controller (USB 1.1 §11.8.6). The device's speed is
    /// recorded inside the [`SplitInfo`].
    Translated(SplitInfo),
}

impl BusRoute {
    /// Speed at which the target device operates.
    pub const fn device_speed(self) -> Speed {
        match self {
            BusRoute::Direct(s) => s,
            BusRoute::Translated(info) => match info.device_speed() {
                SplitSpeed::Low => Speed::Low,
                SplitSpeed::Full => Speed::Full,
            },
        }
    }

    /// Returns the [`SplitInfo`] describing TT/PRE routing, or `None` when
    /// the device is reached directly.
    pub const fn split(self) -> Option<SplitInfo> {
        match self {
            BusRoute::Direct(_) => None,
            BusRoute::Translated(info) => Some(info),
        }
    }
}

/// Information obtained through preliminary enumeration.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EnumerationInfo {
    /// Assigned device address.
    pub device_address: u8,
    /// How the device's traffic reaches it on the bus (speed + optional
    /// split-transaction / PRE-prefix routing).
    pub route: BusRoute,
    /// Parsed device descriptor.
    pub device_desc: DeviceDescriptor,
}

impl EnumerationInfo {
    /// Negotiated device speed.
    pub const fn speed(&self) -> Speed {
        self.route.device_speed()
    }

    /// Split-transaction routing, when this device is behind a hub that
    /// requires splits (HS host reaching LS/FS device through a HS hub, or
    /// FS host reaching LS device through a FS hub). `None` for a device
    /// reached directly.
    pub const fn split(&self) -> Option<SplitInfo> {
        self.route.split()
    }
}

impl EnumerationInfo {
    /// Retrieves the active device configuration, or sets the default if none is active.
    pub async fn active_config_or_set_default<'a, D: IsIn + IsOut, C: UsbPipe<pipe::Control, D>>(
        &self,
        channel: &mut C,
        cfg_desc_buf: &'a mut [u8],
    ) -> Result<ConfigurationDescriptorChain<'a>, HostError> {
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
    pub async fn get_active_configuration<'a, D: IsIn, C: UsbPipe<pipe::Control, D>>(
        &self,
        channel: &mut C,
        cfg_desc_buf: &'a mut [u8],
    ) -> Result<Option<ConfigurationDescriptorChain<'a>>, HostError> {
        let cfg_id = match channel.active_configuration_value().await? {
            Some(v) => v.into(),
            None => return Ok(None),
        };

        let mut index = None;
        let mut cfg_len = 0;
        for i in 0..self.device_desc.num_configurations {
            let cfg_desc_short = retry_descriptor(async || {
                channel
                    .request_descriptor::<ConfigurationDescriptor, { ConfigurationDescriptor::BUF_SIZE }>(i, false)
                    .await
            })
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
        retry_descriptor(async || {
            channel
                .request_descriptor_bytes(ConfigurationDescriptor::DESC_TYPE, index, &mut *dest)
                .await
        })
        .await?;

        let cfg =
            ConfigurationDescriptorChain::try_from_slice(cfg_desc_buf).map_err(|_| HostError::InvalidDescriptor)?;
        Ok(Some(cfg))
    }

    /// Retrieve a device configuration by index.
    pub async fn get_configuration<'a, D: pipe::IsIn, C: UsbPipe<pipe::Control, D>>(
        &self,
        index: u8,
        channel: &mut C,
        cfg_desc_buf: &'a mut [u8],
    ) -> Result<ConfigurationDescriptorChain<'a>, HostError> {
        if index >= self.device_desc.num_configurations {
            return Err(HostError::InvalidDescriptor);
        }

        // Both reads below are retried, for the same reason the device
        // descriptor above them already is: some devices STALL a
        // descriptor read once and answer the identical request
        // milliseconds later. Leaving the configuration descriptor as
        // the only unretried read in enumeration means one such device
        // never enumerates at all, having already got past every step
        // that does allow for it.
        //
        // Observed on a USB gamepad (0428:4001), which STALLs the first
        // configuration-descriptor read issued after it is addressed and
        // then answers the same request on the next attempt. Linux's
        // usbcore makes the same allowance for every descriptor read in
        // `usb_get_descriptor`, commented "some devices are flakey".
        //
        // Retrying is safe here in a way it would not be for a
        // state-changing request: a descriptor read is idempotent, and a
        // control transfer restarts from its SETUP, which clears a
        // control endpoint's protocol stall (USB 2.0 §8.5.3.4) — so
        // there is no leftover state from the failed attempt to undo
        // first.
        let cfg_desc_short = retry_descriptor(async || {
            channel
                .request_descriptor::<ConfigurationDescriptor, { ConfigurationDescriptor::BUF_SIZE }>(index, false)
                .await
        })
        .await?;

        let total_len = cfg_desc_short.total_len as usize;
        if total_len > cfg_desc_buf.len() {
            return Err(HostError::InsufficientMemory);
        }
        let dest = &mut cfg_desc_buf[0..total_len];
        retry_descriptor(async || {
            channel
                .request_descriptor_bytes(ConfigurationDescriptor::DESC_TYPE, index, &mut *dest)
                .await
        })
        .await?;

        trace!(
            "Full Configuration Descriptor [{}]: {:?}",
            cfg_desc_short.total_len, dest
        );

        ConfigurationDescriptorChain::try_from_slice(dest).map_err(|_| HostError::InvalidDescriptor)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HandlerEvent<T> {
    NoChange,
    HandlerDisconnected,
    HandlerEvent(T),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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
