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

use embassy_usb_driver::host::{ChannelError, DeviceEvent, HostError, SetupPacket, UsbChannel, UsbHostDriver, channel};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType, Speed};

use crate::descriptor::{ConfigurationDescriptor, DeviceDescriptor, USBDescriptor};

/// Convert an 8-byte SETUP array to a [`SetupPacket`].
pub(crate) fn bytes_to_setup(b: &[u8; 8]) -> SetupPacket {
    use embassy_usb_driver::host::RequestType;
    SetupPacket {
        request_type: RequestType::from_bits_truncate(b[0]),
        request: b[1],
        value: u16::from_le_bytes([b[2], b[3]]),
        index: u16::from_le_bytes([b[4], b[5]]),
        length: u16::from_le_bytes([b[6], b[7]]),
    }
}

/// USB host enumeration error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EnumerationError {
    /// Transfer failed during enumeration.
    Transfer(ChannelError),
    /// Invalid or unexpected descriptor received.
    InvalidDescriptor,
    /// Configuration buffer too small
    ConfigBufferTooSmall(usize),
    /// No free channel for EP0 or no free device address.
    NoChannel,
    /// The device did not respond to a control request after retries.
    RequestFailed,
}

impl From<ChannelError> for EnumerationError {
    fn from(e: ChannelError) -> Self {
        Self::Transfer(e)
    }
}

impl From<HostError> for EnumerationError {
    fn from(e: HostError) -> Self {
        match e {
            HostError::ChannelError(e) => Self::Transfer(e),
            HostError::InvalidDescriptor => Self::InvalidDescriptor,
            HostError::RequestFailed => Self::RequestFailed,
            _ => Self::NoChannel,
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
            Self::NoChannel => write!(f, "No free channel or no free device address"),
            Self::RequestFailed => write!(f, "Device did not respond"),
        }
    }
}

impl core::error::Error for EnumerationError {}

/// USB host controller.
///
/// Manages device connection, enumeration, and class driver binding.
pub struct UsbHost<D: UsbHostDriver> {
    driver: D,
    /// Bitmask of in-use USB device addresses (1–127).
    /// Bit `n` of `addr_bitmap[n / 64]` is set when address `n` is assigned.
    addr_bitmap: [u64; 2],
}

impl<D: UsbHostDriver> UsbHost<D> {
    /// Create a new USB host from a driver.
    pub fn new(driver: D) -> Self {
        Self {
            driver,
            addr_bitmap: [0u64; 2],
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
    /// Returns the device descriptor, assigned address, and bytes written to config_buf.
    pub async fn enumerate(
        &mut self,
        speed: Speed,
        config_buf: &mut [u8],
    ) -> Result<(DeviceDescriptor, u8, usize), EnumerationError> {
        use crate::control::ControlChannelExt;

        let ep0_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: speed.max_packet_size(),
            interval_ms: 0,
        };

        let mut ch = self
            .driver
            .alloc_channel::<channel::Control, channel::InOut>(0, &ep0_info, false)
            .map_err(|_| EnumerationError::NoChannel)?;

        // Steps 1–3: GET_DESCRIPTOR (partial + full), SET_ADDRESS, retarget channel.
        let addr = self.alloc_address().ok_or(EnumerationError::NoChannel)?;
        let enum_info = ch.enumerate_device(speed, addr, false).await?;
        let dev_desc = enum_info.device_desc;

        info!(
            "Device: VID={:04x} PID={:04x} class={:02x}",
            dev_desc.vendor_id, dev_desc.product_id, dev_desc.device_class
        );

        // Step 4: Get configuration descriptor header (9 bytes).
        let setup = bytes_to_setup(&control::get_config_descriptor(0, 9));
        let n = ch.control_in(&setup, &mut config_buf[..9]).await?;

        if n < 9 {
            return Err(EnumerationError::InvalidDescriptor);
        }

        let config_header = ConfigurationDescriptor::try_from_bytes(&config_buf[..9])
            .map_err(|_| EnumerationError::InvalidDescriptor)?;
        let total_len = config_header.total_len as usize;

        if total_len > config_buf.len() {
            return Err(EnumerationError::ConfigBufferTooSmall(total_len));
        }

        // Get full configuration descriptor.
        let setup = bytes_to_setup(&control::get_config_descriptor(0, total_len as u16));
        let n = ch.control_in(&setup, &mut config_buf[..total_len]).await?;

        // USB 2.0 §9.4.3: the device must return exactly total_len bytes for a full config descriptor.
        if n != total_len {
            return Err(EnumerationError::InvalidDescriptor);
        }

        trace!("Config descriptor: {} bytes", n);

        // Step 5: SET_CONFIGURATION.
        let setup = bytes_to_setup(&control::set_configuration(config_header.configuration_value));
        ch.control_out(&setup, &[]).await?;

        info!("Device configured (config={})", config_header.configuration_value);

        // Channel is released on drop.
        drop(ch);

        Ok((dev_desc, addr, n))
    }
}
