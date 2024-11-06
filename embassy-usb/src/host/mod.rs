//! USB Host implementation
//!
//! Requires an [USBHostDriver] implementation.
//!

#![allow(async_fn_in_trait)]

use core::marker::PhantomData;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::{MappedMutexGuard, Mutex, MutexGuard};
use embassy_time::Timer;
use embassy_usb_driver::host::channel::{Control, Direction, InOut};
use embassy_usb_driver::{
    host::{channel, ChannelError, DeviceEvent, HostError, RequestType, SetupPacket, UsbChannel, UsbHostDriver},
    Speed,
};
use embassy_usb_driver::{Endpoint, EndpointInfo, EndpointType};

use crate::control::Request;
use crate::handlers::EnumerationInfo;

pub mod descriptor;

use descriptor::*;

// type NoopMutex<T> = Mutex<NoopRawMutex, T>;
// type NoopMutexGuard<'a, T> = MutexGuard<'a, NoopRawMutex, T>;
// type NoopMappedMutexGuard<'a, T> = MappedMutexGuard<'a, NoopRawMutex, T>;

pub struct Device {
    pub addr: u8,
    pub dev_desc: DeviceDescriptor,
    pub cfg_desc: ConfigurationDescriptor,
}

/// Extension trait with convenience methods for control channels
pub trait ControlChannelExt<D: channel::Direction>: UsbChannel<channel::Control, D> {
    // CONTROL IN methods
    /// Request and try to parse the device descriptor.
    async fn request_descriptor<T: USBDescriptor, const SIZE: usize>(&mut self, class: bool) -> Result<T, HostError>
    where
        D: channel::IsIn,
    {
        let mut buf = [0u8; SIZE];

        // The wValue field specifies the descriptor type in the high byte
        // and the descriptor index in the low byte.
        let value = (T::DESC_TYPE as u16) << 8;

        let ty = if class {
            RequestType::TYPE_CLASS
        } else {
            RequestType::TYPE_STANDARD
        };

        let packet = SetupPacket {
            request_type: RequestType::IN | ty | RequestType::RECIPIENT_DEVICE,
            request: Request::GET_DESCRIPTOR,
            value,               // descriptor type & index
            index: 0,            // zero or language ID
            length: SIZE as u16, // descriptor length
        };

        self.control_in(&packet, &mut buf).await?;
        trace!("Descriptor {}: {=[u8]}", core::any::type_name::<T>(), buf);

        T::try_from_bytes(&buf).map_err(|e| {
            // TODO: Log error or make descriptor error not generic
            // error!("Device [{}]: Descriptor parse failed: {}", addr, e);
            HostError::InvalidDescriptor
        })
    }

    /// Request the underlying bytes for a descriptor of a specific type.
    /// bytes.len() determines how many bytes are read at maximum.
    /// This can be used for descriptors of varying length, which are parsed by the caller.
    async fn request_descriptor_bytes<T: USBDescriptor>(&mut self, buf: &mut [u8]) -> Result<usize, HostError>
    where
        D: channel::IsIn,
    {
        // The wValue field specifies the descriptor type in the high byte
        // and the descriptor index in the low byte.
        let value = (T::DESC_TYPE as u16) << 8;

        let packet = SetupPacket {
            request_type: RequestType::IN | RequestType::TYPE_STANDARD | RequestType::RECIPIENT_DEVICE,
            request: Request::GET_DESCRIPTOR,
            value,                    // descriptor type & index
            index: 0,                 // zero or language ID
            length: buf.len() as u16, // descriptor length
        };

        let len = self.control_in(&packet, buf).await?;
        Ok(len)
    }

    /// Request the underlying bytes for an additional descriptor of a specific interface.
    /// Useful for class specific descriptors of varying length.
    /// bytes.len() determines how many bytes are read at maximum.
    async fn interface_request_descriptor_bytes<T: USBDescriptor>(
        &mut self,
        interface_num: u8,
        buf: &mut [u8],
    ) -> Result<usize, HostError>
    where
        D: channel::IsIn,
    {
        // The wValue field specifies the descriptor type in the high byte
        // and the descriptor index in the low byte.
        let value = (T::DESC_TYPE as u16) << 8;

        let packet = SetupPacket {
            request_type: RequestType::IN | RequestType::TYPE_STANDARD | RequestType::RECIPIENT_INTERFACE,
            request: Request::GET_DESCRIPTOR,
            value,                       // descriptor type & index
            index: interface_num as u16, // zero or language ID
            length: buf.len() as u16,    // descriptor length
        };

        let len = self.control_in(&packet, buf).await?;
        Ok(len)
    }

    // CONTROL OUT methods

    /// SET_CONFIGURATION control request.
    /// Selects the configuration with the given index `config_no`.
    async fn set_configuration(&mut self, config_no: u8) -> Result<(), HostError>
    where
        D: channel::IsOut,
    {
        let packet = SetupPacket {
            request_type: RequestType::OUT | RequestType::TYPE_STANDARD | RequestType::RECIPIENT_DEVICE,
            request: Request::SET_CONFIGURATION,
            value: config_no as u16,
            index: 0,
            length: 0,
        };

        self.control_out(&packet, &[]).await?;

        Ok(())
    }

    /// Execute the SET_ADDRESS control request. Assign the given address to the device.
    /// Usually done during enumeration.
    ///
    /// # WARNING
    /// This method can break host assumptions. Please do not use it manually
    async fn device_set_address(&mut self, new_addr: u8) -> Result<(), HostError>
    where
        D: channel::IsOut,
    {
        let packet = SetupPacket {
            request_type: RequestType::OUT | RequestType::TYPE_STANDARD | RequestType::RECIPIENT_DEVICE,
            request: Request::SET_ADDRESS,
            value: new_addr as u16,
            index: 0,
            length: 0,
        };

        self.control_out(&packet, &[]).await?;

        Ok(())
    }

    /// Execute a control request with request type Class and recipient Interface
    async fn class_request_out(&mut self, request: u8, value: u16, index: u16, buf: &[u8]) -> Result<(), HostError>
    where
        D: channel::IsOut,
    {
        let packet = SetupPacket {
            request_type: RequestType::OUT | RequestType::TYPE_CLASS | RequestType::RECIPIENT_INTERFACE,
            request,
            value,
            index,
            length: buf.len() as u16,
        };

        self.control_out(&packet, buf).await?;

        Ok(())
    }

    /// Enumerate *the* currently pending device
    ///  the device is expected to be reset right before this
    ///
    /// - `speed` is generally provided by the hardware (core or hub)
    /// - `new_device_address` is generated by the software from any available
    /// - `ls_over_fs` is derived from the topology (tracker in software), if the device is LS but is plugged into an FS/HS hub it needs this flag set
    async fn enumerate_device(
        &mut self,
        speed: Speed,
        new_device_address: u8,
        ls_over_fs: bool,
    ) -> Result<EnumerationInfo, HostError>
    where
        D: channel::IsIn + channel::IsOut,
    {
        self.retarget_channel(
            0,
            &EndpointInfo::new(0.into(), EndpointType::Control, speed.max_packet_size()),
            ls_over_fs,
        )?;

        trace!("[enum] Attempting to get max_packet_size for new device");
        let max_packet_size0 = {
            let mut max_retries = 10;
            loop {
                match self
                    .request_descriptor::<DeviceDescriptorPartial, { DeviceDescriptorPartial::SIZE }>(false)
                    .await
                {
                    Ok(desc) => break desc.max_packet_size0,
                    Err(e) => {
                        warn!("Request descriptor error: {}, retries: {}", e, max_retries);
                        if max_retries > 0 {
                            max_retries -= 1;
                            Timer::after_millis(1).await;
                            trace!("Retry Device Descriptor");
                            continue;
                        } else {
                            return Err(HostError::RequestFailed);
                        }
                    }
                }
            }
        };

        trace!(
            "[enum] got max packet size for new device {}, attempting to set address",
            max_packet_size0
        );

        self.device_set_address(new_device_address).await?;

        // TODO: device has 2ms to change internally by spec but may be faster, we can retry to speed up enumertion
        Timer::after_millis(2).await;

        trace!("[enum] Finished setting address");
        self.retarget_channel(
            new_device_address,
            &EndpointInfo::new(0.into(), EndpointType::Control, max_packet_size0 as u16),
            ls_over_fs,
        )?;

        let device_desc = self
            .request_descriptor::<DeviceDescriptor, { DeviceDescriptor::SIZE }>(false)
            .await?;

        trace!("Device Descriptor: {:?}", device_desc);

        let cfg_desc_short = self
            .request_descriptor::<ConfigurationDescriptor, { ConfigurationDescriptor::SIZE }>(false)
            .await?;

        let total_len = cfg_desc_short.total_len as usize;
        let mut desc_buffer = [0u8; 256];
        let dest_buffer = &mut desc_buffer[0..total_len];

        self.request_descriptor_bytes::<ConfigurationDescriptor>(dest_buffer)
            .await?;

        trace!(
            "Full Configuration Descriptor [{}]: {:?}",
            cfg_desc_short.total_len,
            dest_buffer
        );

        self.set_configuration(cfg_desc_short.configuration_value).await?;

        let cfg_desc =
            ConfigurationDescriptor::try_from_bytes(&dest_buffer).map_err(|_| HostError::InvalidDescriptor)?;

        Ok(EnumerationInfo {
            device_address: new_device_address,
            ls_over_fs,
            speed,
            device_desc,
            cfg_desc,
        })
    }
}

impl<D: channel::Direction, C> ControlChannelExt<D> for C where C: UsbChannel<channel::Control, D> {}

/// Extensions for the UsbHostDriver trait
pub trait UsbHostBusExt: UsbHostDriver {
    /// Enumerates the root port of the device
    async fn enumerate_root(&mut self, speed: Speed, new_device_address: u8) -> Result<EnumerationInfo, HostError> {
        // Need to reset bus to initialize device?
        self.bus_reset().await;

        let mut channel = self.alloc_channel::<Control, InOut>(
            0,
            &EndpointInfo::new(0.into(), EndpointType::Control, speed.max_packet_size()),
            false,
        )?;

        channel.enumerate_device(speed, new_device_address, false).await
    }
}

impl<C: UsbHostDriver> UsbHostBusExt for C {}
