#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod class;
pub mod control;
pub mod descriptor;
pub mod handler;

use embassy_usb_driver::host::{DeviceEvent, HostError, PipeError, UsbHostDriver, UsbPipe, pipe};
pub use embassy_usb_driver::host::{SplitInfo, SplitSpeed};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType, Speed};

use crate::control::{ControlPipeExt, SetupPacket};
use crate::descriptor::{ConfigurationDescriptor, DeviceDescriptor, USBDescriptor};
pub use crate::handler::BusRoute;
use crate::handler::EnumerationInfo;

/// USB host enumeration error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EnumerationError {
    /// Transfer failed during enumeration.
    Transfer(PipeError),
    /// Invalid or unexpected descriptor received.
    InvalidDescriptor,
    /// Configuration buffer too small
    ConfigBufferTooSmall(usize),
    /// No free pipe for EP0 or no free device address.
    NoPipe,
    /// The device did not respond to a control request after retries.
    RequestFailed,
}

impl From<PipeError> for EnumerationError {
    fn from(e: PipeError) -> Self {
        Self::Transfer(e)
    }
}

impl From<HostError> for EnumerationError {
    fn from(e: HostError) -> Self {
        match e {
            HostError::PipeError(e) => Self::Transfer(e),
            HostError::InvalidDescriptor => Self::InvalidDescriptor,
            HostError::RequestFailed => Self::RequestFailed,
            _ => Self::NoPipe,
        }
    }
}

impl core::fmt::Display for EnumerationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_e) => write!(f, "Transfer error during enumeration"),
            Self::InvalidDescriptor => write!(f, "Invalid descriptor"),
            Self::ConfigBufferTooSmall(size) => {
                write!(f, "Configuration buffer too small: device requires {} bytes", size)
            }
            Self::NoPipe => write!(f, "No free pipe or no free device address"),
            Self::RequestFailed => write!(f, "Device did not respond"),
        }
    }
}

impl core::error::Error for EnumerationError {}

/// USB host controller.
///
/// Manages device connection, enumeration, and class driver binding.
pub struct UsbHost<'d, D: UsbHostDriver<'d>> {
    driver: D,
    /// Bitmask of in-use USB device addresses (1–127).
    /// Bit `n` of `addr_bitmap[n / 64]` is set when address `n` is assigned.
    addr_bitmap: [u64; 2],
    _phantom: core::marker::PhantomData<&'d ()>,
}

impl<'d, D: UsbHostDriver<'d>> UsbHost<'d, D> {
    /// Create a new USB host from a driver.
    pub fn new(driver: D) -> Self {
        Self {
            driver,
            addr_bitmap: [0u64; 2],
            _phantom: core::marker::PhantomData,
        }
    }

    /// Allocate the next free device address (1–127), marking it as in use.
    fn alloc_address(&mut self) -> Option<u8> {
        for addr in 1u8..=127 {
            let word = (addr / 64) as usize;
            let bit = addr % 64;
            if self.addr_bitmap[word] & (1u64 << bit) == 0 {
                self.addr_bitmap[word] |= 1u64 << bit;
                return Some(addr);
            }
        }
        None
    }

    /// Release a previously allocated device address.
    pub fn free_address(&mut self, addr: u8) {
        if addr >= 1 && addr <= 127 {
            let word = (addr / 64) as usize;
            let bit = addr % 64;
            self.addr_bitmap[word] &= !(1u64 << bit);
        }
    }

    /// Get a reference to the underlying driver.
    pub fn driver(&self) -> &D {
        &self.driver
    }

    /// Get a mutable reference to the underlying driver.
    pub fn driver_mut(&mut self) -> &mut D {
        &mut self.driver
    }

    /// Wait for a device to connect.
    ///
    /// Issues a bus reset internally and returns the detected speed.
    pub async fn wait_for_connection(&mut self) -> Speed {
        loop {
            match self.driver.wait_for_device_event().await {
                DeviceEvent::Connected(speed) => {
                    info!("USB device connected, speed: {:?}", speed);
                    return speed;
                }
                DeviceEvent::Disconnected => {
                    // Spurious disconnect before connect; try again.
                    continue;
                }
                _ => {
                    // Overcurrent, remote-wakeup, or any future variant that
                    // isn't a fresh connection: keep waiting for a real
                    // Connected event.
                    continue;
                }
            }
        }
    }

    /// Enumerate a connected device.
    ///
    /// Performs the standard enumeration sequence:
    /// 1. Get device descriptor (first 8 bytes) to learn EP0 max packet size
    /// 2. SET_ADDRESS to assign a unique address
    /// 3. Get full device descriptor
    /// 4. Get configuration descriptor
    /// 5. SET_CONFIGURATION
    ///
    /// `route` describes how the device is reached on the bus (directly at
    /// its native speed, or via split transactions / legacy `PRE` through a
    /// hub's transaction translator).
    ///
    /// # Preconditions
    ///
    /// The caller must have placed the device into the default (address 0)
    /// state *before* calling this method. For a root-port device that
    /// means an upstream bus reset has completed; for a hub-attached
    /// device, the parent hub's port reset must have completed and
    /// [`BusRoute::Translated`] must carry the appropriate [`SplitInfo`].
    ///
    /// Returns the [`EnumerationInfo`] for the device and bytes written to `config_buf`.
    ///
    /// [`SplitInfo`]: embassy_usb_driver::host::SplitInfo
    pub async fn enumerate(
        &mut self,
        route: BusRoute,
        config_buf: &mut [u8],
    ) -> Result<(EnumerationInfo, usize), EnumerationError> {
        use embassy_time::Timer;

        use crate::descriptor::DeviceDescriptorPartial;

        let addr = self.alloc_address().ok_or(EnumerationError::NoPipe)?;

        let ep0_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: route.device_speed().max_packet_size(),
            interval_ms: 0,
        };

        let mut ch = self
            .driver
            .alloc_pipe::<pipe::Control, pipe::InOut>(0, &ep0_info, route.split())
            .map_err(|_| {
                self.free_address(addr);
                EnumerationError::NoPipe
            })?;

        trace!("[enum] Getting max_packet_size for new device");
        let max_packet_size0 = {
            let mut max_retries = 10;
            loop {
                match ch
                    .request_descriptor::<DeviceDescriptorPartial, { DeviceDescriptorPartial::SIZE }>(0, false)
                    .await
                {
                    Ok(desc) => break desc.max_packet_size0,
                    Err(e) => {
                        warn!("Request descriptor error: {:?}, retries: {}", e, max_retries);
                        if max_retries > 0 {
                            max_retries -= 1;
                            Timer::after_millis(1).await;
                            continue;
                        } else {
                            self.free_address(addr);
                            return Err(e.into());
                        }
                    }
                }
            }
        };
        // USB 2.0 §9.6.1: legal EP0 max packet sizes are 8, 16, 32, 64.
        if !matches!(max_packet_size0, 8 | 16 | 32 | 64) {
            self.free_address(addr);
            return Err(EnumerationError::InvalidDescriptor);
        }

        ch.device_set_address(addr).await?;
        // USB 2.0 §9.2.6.3: allow the device a 2ms recovery interval after SET_ADDRESS.
        Timer::after_millis(2).await;

        // Drop pipe to re-allocate with new address and correct max_packet_size.
        drop(ch);

        let ep0_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: max_packet_size0 as u16,
            interval_ms: 0,
        };

        let mut ch = self
            .driver
            .alloc_pipe::<pipe::Control, pipe::InOut>(addr, &ep0_info, route.split())
            .map_err(|_| {
                self.free_address(addr);
                EnumerationError::NoPipe
            })?;

        let retries = 5;
        let dev_desc = async {
            for _ in 0..retries {
                match ch
                    .request_descriptor::<DeviceDescriptor, { DeviceDescriptor::SIZE }>(0, false)
                    .await
                {
                    Err(HostError::PipeError(PipeError::Timeout)) => {
                        Timer::after_millis(1).await;
                        continue;
                    }
                    v => return v,
                }
            }
            Err(HostError::PipeError(PipeError::Timeout))
        }
        .await?;

        info!(
            "Device: VID={:04x} PID={:04x} class={:02x}",
            dev_desc.vendor_id, dev_desc.product_id, dev_desc.device_class
        );

        // Step 4: Get configuration descriptor header (9 bytes).
        let setup = SetupPacket::get_config_descriptor(0, 9);
        let n = ch
            .control_in(&setup.to_bytes(), &mut config_buf[..9])
            .await
            .inspect_err(|_| self.free_address(addr))?;

        if n < 9 {
            self.free_address(addr);
            return Err(EnumerationError::InvalidDescriptor);
        }

        let config_header = ConfigurationDescriptor::try_from_bytes(&config_buf[..9])
            .map_err(|_| EnumerationError::InvalidDescriptor)?;
        let total_len = config_header.total_len as usize;

        if total_len > config_buf.len() {
            self.free_address(addr);
            return Err(EnumerationError::ConfigBufferTooSmall(total_len));
        }

        // Get full configuration descriptor.
        let setup = SetupPacket::get_config_descriptor(0, total_len as u16);
        let n = ch.control_in(&setup.to_bytes(), &mut config_buf[..total_len]).await?;

        // USB 2.0 §9.4.3: the device must return exactly total_len bytes for a full config descriptor.
        if n != total_len {
            self.free_address(addr);
            return Err(EnumerationError::InvalidDescriptor);
        }

        trace!("Config descriptor: {} bytes", n);

        // Step 5: SET_CONFIGURATION.
        let setup = SetupPacket::set_configuration(config_header.configuration_value);
        ch.control_out(&setup.to_bytes(), &[])
            .await
            .inspect_err(|_| self.free_address(addr))?;

        info!("Device configured (config={})", config_header.configuration_value);

        // Pipe is released on drop.
        drop(ch);

        Ok((
            EnumerationInfo {
                device_address: addr,
                route,
                device_desc: dev_desc,
            },
            n,
        ))
    }
}
