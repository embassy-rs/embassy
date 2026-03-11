#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod class;
pub mod control;
pub mod descriptor;

pub use embassy_usb_host_driver as driver;
use embassy_usb_host_driver::{
    DeviceEndpoint, DeviceSpeed, Direction, EndpointType, HostBus, HostChannel, PortEvent, TransferError,
};

use crate::descriptor::{ConfigDescriptor, DeviceDescriptor};

/// USB host enumeration error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EnumerationError {
    /// Transfer failed during enumeration.
    Transfer(TransferError),
    /// Invalid or unexpected descriptor received.
    InvalidDescriptor,
    /// No free channel for EP0.
    NoChannel,
}

impl From<TransferError> for EnumerationError {
    fn from(e: TransferError) -> Self {
        Self::Transfer(e)
    }
}

impl From<embassy_usb_host_driver::ChannelAllocError> for EnumerationError {
    fn from(_: embassy_usb_host_driver::ChannelAllocError) -> Self {
        Self::NoChannel
    }
}

/// USB host controller.
///
/// Manages device connection, enumeration, and class driver binding.
pub struct UsbHost<B: HostBus> {
    bus: B,
    next_address: u8,
}

impl<B: HostBus> UsbHost<B> {
    /// Create a new USB host from a bus.
    pub fn new(bus: B) -> Self {
        Self { bus, next_address: 1 }
    }

    /// Get a mutable reference to the underlying bus.
    pub fn bus_mut(&mut self) -> &mut B {
        &mut self.bus
    }

    /// Get a reference to the underlying bus.
    pub fn bus(&self) -> &B {
        &self.bus
    }

    /// Enable the host controller.
    pub async fn enable(&mut self) {
        self.bus.enable().await;
    }

    /// Wait for a device to connect.
    pub async fn wait_for_connection(&mut self) -> DeviceSpeed {
        loop {
            let event = self.bus.poll().await;
            if let PortEvent::Connected = event {
                info!("USB device connected, resetting port...");
                self.bus.reset().await;

                // Wait for Enabled event after reset
                loop {
                    let event = self.bus.poll().await;
                    if let PortEvent::Enabled { speed } = event {
                        info!("Port enabled, speed: {:?}", speed);
                        return speed;
                    }
                    if let PortEvent::Disconnected = event {
                        break; // Try again
                    }
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
        speed: DeviceSpeed,
        config_buf: &mut [u8],
    ) -> Result<(DeviceDescriptor, u8, usize), EnumerationError> {
        // Step 1: Get device descriptor (first 8 bytes) on address 0, MPS=8
        let ep0 = DeviceEndpoint {
            device_address: 0,
            ep_number: 0,
            direction: Direction::In,
            ep_type: EndpointType::Control,
            max_packet_size: 8,
            speed,
        };

        let mut ch = self.bus.alloc_channel(&ep0)?;
        let mut desc_buf = [0u8; 18];

        // GET_DESCRIPTOR(Device, 8 bytes)
        let setup = control::get_device_descriptor(8);
        let n = ch.control_transfer(&setup, Direction::In, &mut desc_buf[..8]).await?;

        if n < 8 {
            return Err(EnumerationError::InvalidDescriptor);
        }

        let max_packet_size_0 = desc_buf[7];
        trace!("EP0 max packet size: {}", max_packet_size_0);

        // Step 2: SET_ADDRESS
        let addr = self.next_address;
        self.next_address = self.next_address.wrapping_add(1).max(1);

        let setup = control::set_address(addr);
        ch.control_transfer(&setup, Direction::Out, &mut []).await?;

        // Retarget channel to new address
        ch.retarget(addr, max_packet_size_0 as u16);

        trace!("Device assigned address {}", addr);

        // Step 3: Get full device descriptor (18 bytes)
        let setup = control::get_device_descriptor(18);
        let n = ch.control_transfer(&setup, Direction::In, &mut desc_buf).await?;

        if n < 18 {
            return Err(EnumerationError::InvalidDescriptor);
        }

        let dev_desc = DeviceDescriptor::parse(&desc_buf).ok_or(EnumerationError::InvalidDescriptor)?;
        info!(
            "Device: VID={:04x} PID={:04x} class={:02x}",
            dev_desc.vendor_id, dev_desc.product_id, dev_desc.device_class
        );

        // Step 4: Get configuration descriptor header (9 bytes)
        let setup = control::get_config_descriptor(0, 9);
        let n = ch.control_transfer(&setup, Direction::In, &mut config_buf[..9]).await?;

        if n < 9 {
            return Err(EnumerationError::InvalidDescriptor);
        }

        let config_header = ConfigDescriptor::parse(&config_buf[..9]).ok_or(EnumerationError::InvalidDescriptor)?;
        let total_len = config_header.total_length as usize;

        if total_len > config_buf.len() {
            return Err(EnumerationError::InvalidDescriptor);
        }

        // Get full configuration descriptor
        let setup = control::get_config_descriptor(0, total_len as u16);
        let n = ch
            .control_transfer(&setup, Direction::In, &mut config_buf[..total_len])
            .await?;

        trace!("Config descriptor: {} bytes", n);

        // Step 5: SET_CONFIGURATION
        let setup = control::set_configuration(config_header.config_value);
        ch.control_transfer(&setup, Direction::Out, &mut []).await?;

        info!("Device configured (config={})", config_header.config_value);

        // Channel is released on drop
        drop(ch);

        Ok((dev_desc, addr, n))
    }
}
