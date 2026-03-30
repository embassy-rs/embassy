#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod class;
pub mod control;
pub mod descriptor;

use embassy_usb_driver::host::{ChannelError, DeviceEvent, HostError, SetupPacket, UsbChannel, UsbHostDriver, channel};
use embassy_usb_driver::{Direction as UsbDirection, EndpointAddress, EndpointInfo, EndpointType, Speed};

use crate::descriptor::{ConfigDescriptor, DeviceDescriptor};

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
    /// No free channel for EP0.
    NoChannel,
}

impl From<ChannelError> for EnumerationError {
    fn from(e: ChannelError) -> Self {
        Self::Transfer(e)
    }
}

impl From<HostError> for EnumerationError {
    fn from(_: HostError) -> Self {
        Self::NoChannel
    }
}

impl core::fmt::Display for EnumerationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_e) => write!(f, "Transfer error during enumeration"),
            Self::InvalidDescriptor => write!(f, "Invalid descriptor"),
            Self::NoChannel => write!(f, "No free channel"),
        }
    }
}

impl core::error::Error for EnumerationError {}

/// USB host controller.
///
/// Manages device connection, enumeration, and class driver binding.
pub struct UsbHost<D: UsbHostDriver> {
    driver: D,
    next_address: u8,
}

impl<D: UsbHostDriver> UsbHost<D> {
    /// Create a new USB host from a driver.
    pub fn new(driver: D) -> Self {
        Self {
            driver,
            next_address: 1,
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
        // Step 1: Get device descriptor (first 8 bytes) on address 0.
        // Use the speed-specific default MPS for the initial control transfer.
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

        let mut desc_buf = [0u8; 18];

        // GET_DESCRIPTOR(Device, 8 bytes)
        let setup = bytes_to_setup(&control::get_device_descriptor(8));
        let n = ch.control_in(&setup, &mut desc_buf[..8]).await?;

        if n < 8 {
            return Err(EnumerationError::InvalidDescriptor);
        }

        let max_packet_size_0 = desc_buf[7];
        trace!("EP0 max packet size: {}", max_packet_size_0);

        // Step 2: SET_ADDRESS
        let addr = self.next_address;
        self.next_address = if self.next_address >= 127 {
            1
        } else {
            self.next_address + 1
        };

        let setup = bytes_to_setup(&control::set_address(addr));
        ch.control_out(&setup, &[]).await?;

        // Retarget channel to new address and max packet size
        let new_ep0_info = EndpointInfo {
            addr: EndpointAddress::from_parts(0, UsbDirection::In),
            ep_type: EndpointType::Control,
            max_packet_size: max_packet_size_0 as u16,
            interval_ms: 0,
        };
        ch.retarget_channel(addr, &new_ep0_info, false)
            .map_err(|_| EnumerationError::NoChannel)?;

        trace!("Device assigned address {}", addr);

        // Step 3: Get full device descriptor (18 bytes)
        let setup = bytes_to_setup(&control::get_device_descriptor(18));
        let n = ch.control_in(&setup, &mut desc_buf).await?;

        if n < 18 {
            return Err(EnumerationError::InvalidDescriptor);
        }

        let dev_desc = DeviceDescriptor::parse(&desc_buf).ok_or(EnumerationError::InvalidDescriptor)?;
        info!(
            "Device: VID={:04x} PID={:04x} class={:02x}",
            dev_desc.vendor_id, dev_desc.product_id, dev_desc.device_class
        );

        // Step 4: Get configuration descriptor header (9 bytes)
        let setup = bytes_to_setup(&control::get_config_descriptor(0, 9));
        let n = ch.control_in(&setup, &mut config_buf[..9]).await?;

        if n < 9 {
            return Err(EnumerationError::InvalidDescriptor);
        }

        let config_header = ConfigDescriptor::parse(&config_buf[..9]).ok_or(EnumerationError::InvalidDescriptor)?;
        let total_len = config_header.total_length as usize;

        if total_len > config_buf.len() {
            return Err(EnumerationError::InvalidDescriptor);
        }

        // Get full configuration descriptor
        let setup = bytes_to_setup(&control::get_config_descriptor(0, total_len as u16));
        let n = ch.control_in(&setup, &mut config_buf[..total_len]).await?;

        trace!("Config descriptor: {} bytes", n);

        // Step 5: SET_CONFIGURATION
        let setup = bytes_to_setup(&control::set_configuration(config_header.config_value));
        ch.control_out(&setup, &[]).await?;

        info!("Device configured (config={})", config_header.config_value);

        // Channel is released on drop
        drop(ch);

        Ok((dev_desc, addr, n))
    }
}
