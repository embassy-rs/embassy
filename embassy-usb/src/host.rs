//! USB Host implementation
//!
//! Requires an [USBHostDriver] implementation.
//!

#![allow(async_fn_in_trait)]

use embassy_time::Timer;
use embassy_usb_driver::host::{channel, ChannelError, DeviceEvent, EndpointDescriptor, HostError, RequestType, SetupPacket, UsbChannel, UsbHostDriver};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::{Mutex, MutexGuard};

use heapless::Vec;

use crate::control::Request;
use crate::descriptor::descriptor_type;

type StringIndex = u8;
// FIXME: Why is there no alias already?..
type NoopMutex<T> = Mutex<NoopRawMutex, T>;
type NoopMutexGuard<'a, T> = MutexGuard<'a, NoopRawMutex, T>;

/// First 8 bytes of the DeviceDescriptor. This is used to figure out the `max_packet_size0` value to reconfigure channel 0.
/// All USB devices support max_packet_size0=8 which is why the first 8 bytes of the descriptor can always be read.
#[allow(missing_docs)]
#[derive(Debug)]
pub struct DeviceDescriptorPartial {
    _padding: [u8; 7],
    pub max_packet_size0: u8,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub struct DeviceDescriptor {
    pub len: u8,
    pub descriptor_type: u8,
    pub bcd_usb: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size0: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub bcd_device: u16,
    pub manufacturer: StringIndex,
    pub product: StringIndex,
    pub serial_number: StringIndex,
    pub num_configurations: u8,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub struct ConfigurationDescriptor {
    pub len: u8,
    pub descriptor_type: u8,
    pub total_len: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration_name: StringIndex,
    pub attributes: u8,
    pub max_power: u8,

    /// All additional bytes end up in this buffer.
    /// This includes the interface descriptors
    pub buffer: [u8; 256],
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub struct InterfaceDescriptor<'a> {
    pub len: u8,
    pub descriptor_type: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub interface_name: StringIndex,

    /// All additional bytes end up in this buffer.
    /// This buffer can then be used to parse endpoint descriptors or class descriptors
    pub buffer: &'a [u8],
}

/// Trait to be implemented by fixed size descriptors for automatic parsing.
pub trait USBDescriptor {
    /// Fixed size of the descriptor
    /// For varying length descriptors, this cannot be used and they have to be parsed outside of this module.
    const SIZE: usize;

    /// The descriptor type that has to match the type of this descriptor.
    const DESC_TYPE: u8;

    /// The type returned on error
    type Error;

    /// Try to parse the descriptor from a byte slice
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl USBDescriptor for DeviceDescriptorPartial {
    const SIZE: usize = 8;

    const DESC_TYPE: u8 = descriptor_type::DEVICE;

    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        Ok(Self {
            _padding: [0; 7],
            max_packet_size0: bytes[7],
        })
    }
}

impl USBDescriptor for DeviceDescriptor {
    const SIZE: usize = 18;

    const DESC_TYPE: u8 = descriptor_type::DEVICE;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        Ok(Self {
            len: bytes[0],
            descriptor_type: bytes[1],
            bcd_usb: u16::from_le_bytes([bytes[2], bytes[3]]),
            device_class: bytes[4],
            device_subclass: bytes[5],
            device_protocol: bytes[6],
            max_packet_size0: bytes[7],
            vendor_id: u16::from_le_bytes([bytes[8], bytes[9]]),
            product_id: u16::from_le_bytes([bytes[10], bytes[11]]),
            bcd_device: u16::from_le_bytes([bytes[12], bytes[13]]),
            manufacturer: bytes[14],
            product: bytes[15],
            serial_number: bytes[16],
            num_configurations: bytes[17],
        })
    }
}

impl USBDescriptor for ConfigurationDescriptor {
    const SIZE: usize = 9;

    const DESC_TYPE: u8 = descriptor_type::CONFIGURATION;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        let mut buffer = [0u8; 256];
        let rest_of_desc = &bytes[Self::SIZE..];
        buffer[..rest_of_desc.len()].copy_from_slice(rest_of_desc);

        Ok(Self {
            len: bytes[0],
            descriptor_type: bytes[1],
            total_len: u16::from_le_bytes([bytes[2], bytes[3]]),
            num_interfaces: bytes[4],
            configuration_value: bytes[5],
            configuration_name: bytes[6],
            attributes: bytes[7],
            max_power: bytes[8],
            buffer,
        })
    }
}

impl ConfigurationDescriptor {
    /// Try to find and parse the interface with interface number `index`
    pub fn parse_interface<'a>(&'a self, index: usize) -> Option<InterfaceDescriptor<'a>> {
        if index >= self.num_interfaces as usize {
            return None;
        }

        let mut dest_buffer = self.buffer_sliced();

        let mut start = None;

        // Find interface that matches the requested index
        while let Some((offset, interface_number)) = Self::identify_interface(dest_buffer) {
            if interface_number == index as u8 {
                // start of interface
                start = Some(offset);
                break;
            }
            dest_buffer = &dest_buffer[offset + InterfaceDescriptor::SIZE..];
        }

        // start is relative to current dest_buffer.
        let Some(start) = start else { return None };

        // Find next interface if any
        let next_interface_buffer = &dest_buffer[start + InterfaceDescriptor::SIZE..];

        let interface_bytes = if let Some((offset, _)) = Self::identify_interface(next_interface_buffer) {
            let end = start + InterfaceDescriptor::SIZE + offset;
            &dest_buffer[start..end]
        } else {
            &dest_buffer[start..]
        };

        InterfaceDescriptor::try_from_bytes(interface_bytes).ok()
    }

    fn buffer_sliced(&self) -> &[u8] {
        // The confiuration descriptor's own bytes are already consumed.
        let end = self.total_len as usize - Self::SIZE;
        &self.buffer[..end]
    }

    // Returns the offset to the next interface descriptor as well as the interface_number (index in descriptor)
    fn identify_interface(slice: &[u8]) -> Option<(usize, u8)> {
        let mut offset = 0;
        let mut desc_len = slice[offset] as usize;
        let mut desc_type = slice[offset + 1];

        while desc_type != InterfaceDescriptor::DESC_TYPE || desc_len != InterfaceDescriptor::SIZE {
            // 'flush' buffer until end of descriptor
            offset += desc_len.max(1); // at least 1 byute to prevent infinite loop
            if offset + InterfaceDescriptor::SIZE > slice.len() {
                // end of slice
                return None;
            }

            desc_len = slice[offset] as usize;
            desc_type = slice[offset + 1];
        }

        let interface_number = slice[offset + 2];
        return Some((offset, interface_number));
    }
}

/// InterfaceDescriptor does not implement [USBDescriptor] because it has a borrowed buffer.
/// Since we cannot request an interface decriptor from the device by itself it does not strictly need to implement [USBDescriptor].
impl<'a> InterfaceDescriptor<'a> {
    const SIZE: usize = 9;

    const DESC_TYPE: u8 = descriptor_type::INTERFACE;

    fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, ()> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        Ok(Self {
            len: bytes[0],
            descriptor_type: bytes[1],
            interface_number: bytes[2],
            alternate_setting: bytes[3],
            num_endpoints: bytes[4],
            interface_class: bytes[5],
            interface_subclass: bytes[6],
            interface_protocol: bytes[7],
            interface_name: bytes[8],
            buffer: &bytes[Self::SIZE..],
        })
    }

    /// Try to parse a class descriptor of a given type
    pub fn parse_class_descriptor<T: USBDescriptor>(&self) -> Option<T> {
        Self::identify_descriptor::<T>(self.buffer)
            .map(|i| T::try_from_bytes(&self.buffer[i..]).ok())
            .flatten()
    }

    /// Parse up to `L` endpoints corresponding to this interface.
    /// Returns a vector of EndpointDescriptors. The length of the vector is `min(L, self.num_endpoints)`.
    pub fn parse_endpoints<const L: usize>(&self) -> Vec<EndpointDescriptor, L> {
        let mut endpoints: Vec<EndpointDescriptor, L> = Vec::new();

        let mut working_buffer = &self.buffer[..];
        for _ in 0..self.num_endpoints.min(L as u8) {
            if let Some(endpoint) = Self::identify_descriptor::<EndpointDescriptor>(working_buffer)
                .map(|i| {
                    working_buffer = &working_buffer[i..];
                    EndpointDescriptor::try_from_bytes(working_buffer).ok()
                })
                .flatten()
            {
                // safe because we limited the iterations.
                endpoints.push(endpoint).ok();
            }
            working_buffer = &working_buffer[EndpointDescriptor::SIZE..];
        }

        endpoints
    }

    // Returns the offset to the first matching descriptor in the slice
    fn identify_descriptor<T: USBDescriptor>(slice: &[u8]) -> Option<usize> {
        let mut offset = 0;
        let mut desc_len = slice[offset] as usize;
        let mut desc_type = slice[offset + 1];

        while desc_type != T::DESC_TYPE || desc_len != T::SIZE {
            // 'flush' buffer until end of descriptor
            offset += desc_len.max(1); // at least 1 byute to prevent infinite loop
            if offset + T::SIZE > slice.len() {
                // end of slice
                return None;
            }

            desc_len = slice[offset] as usize;
            desc_type = slice[offset + 1];
        }

        return Some(offset);
    }
}

impl USBDescriptor for EndpointDescriptor {
    const SIZE: usize = 7;

    const DESC_TYPE: u8 = descriptor_type::ENDPOINT;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        Ok(Self {
            len: bytes[0],
            descriptor_type: bytes[1],
            endpoint_address: bytes[2],
            attributes: bytes[3],
            max_packet_size: u16::from_le_bytes([bytes[4], bytes[5]]),
            interval: bytes[6],
        })
    }
}


pub struct Device {
    pub addr: u8,
    pub dev_desc: DeviceDescriptor,
    pub cfg_desc: ConfigurationDescriptor,
}


/// Channel wrapper with convenient methods and drop behaviour selection
pub trait ChannelDrop<D: UsbHostDriver, T: channel::Type, DIR: channel::Direction> {
    fn drop(&self, _ch: &mut D::Channel<T, DIR>) {}
}

/// Channel is dropped by referenced driver
pub struct Channel<'d, D, T, DIR>
where 
    T: channel::Type,
    DIR: channel::Direction,
    D: UsbHostDriver,
{
    channel: D::Channel<T, DIR>,
    driver: &'d D,
    // TODO: Pass device registry
}

impl<D, T, DIR> Drop for Channel<'_, D, T, DIR>
where 
    T: channel::Type,
    DIR: channel::Direction,
    D: UsbHostDriver,
{
    fn drop(&mut self) {
        trace!("Drop channel");
        self.driver.drop_channel(&mut self.channel)
    }
}

impl<D, T, DIR> UsbChannel<T, DIR> for Channel<'_, D, T, DIR>
where 
    T: channel::Type,
    DIR: channel::Direction,
    D: UsbHostDriver,
{
    async fn control_in(&mut self, setup: &SetupPacket, buf: &mut [u8]) -> Result<usize, ChannelError>
    where 
        T: channel::IsControl,
        DIR: channel::IsIn {
        self.channel.control_in(setup, buf).await
    }

    async fn control_out(&mut self, setup: &SetupPacket, buf: &[u8]) -> Result<usize, ChannelError>
    where 
        T: channel::IsControl,
        DIR: channel::IsOut {
        self.channel.control_out(setup, buf).await
    }

    async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, ChannelError>
    where 
        DIR: channel::IsIn {
        self.channel.request_in(buf).await
    }

    async fn request_out(&mut self, buf: &[u8]) -> Result<usize, ChannelError>
    where 
        DIR: channel::IsOut {
        self.channel.request_out(buf).await
    }
} 

/// Extension trait with convenience methods for control channels
pub trait ControlChannelExt<D: channel::Direction>: UsbChannel<channel::Control, D>  {
    // CONTROL IN methods
    /// Request and try to parse the device descriptor.
    async fn request_descriptor<T: USBDescriptor, const SIZE: usize>(
        &mut self, 
    ) -> Result<T, HostError>
    where D: channel::IsIn
    {
        let mut buf = [0u8; SIZE];

        // The wValue field specifies the descriptor type in the high byte
        // and the descriptor index in the low byte.
        let value = (T::DESC_TYPE as u16) << 8;

        let packet = SetupPacket {
            request_type: RequestType::IN | RequestType::TYPE_STANDARD | RequestType::RECIPIENT_DEVICE,
            request: Request::GET_DESCRIPTOR,
            value,               // descriptor type & index
            index: 0,            // zero or language ID
            length: SIZE as u16, // descriptor length
        };

        self.control_in(&packet, &mut buf).await?;

        T::try_from_bytes(&buf).map_err(|e| { 
            // TODO: Log error or make descriptor error not generic
            // error!("Device [{}]: Descriptor parse failed: {}", addr, e);
            HostError::InvalidDescriptor 
        })
    }
    
    /// Request the underlying bytes for a descriptor of a specific type.
    /// bytes.len() determines how many bytes are read at maximum.
    /// This can be used for descriptors of varying length, which are parsed by the caller.
    async fn request_descriptor_bytes<T: USBDescriptor>(
        &mut self, 
        buf: &mut [u8],
    ) -> Result<usize, HostError> 
    where D: channel::IsIn
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
    where D: channel::IsIn
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
    where D: channel::IsOut
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
    where D: channel::IsOut
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
    async fn class_request_out(
        &mut self, 
        request: u8, 
        value: u16, 
        index: u16, 
        buf: &mut [u8]
    ) -> Result<(), HostError>
    where D: channel::IsOut
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
}

impl<D: channel::Direction, C> ControlChannelExt<D> for C where C: UsbChannel<channel::Control, D> {}

pub struct UsbHost<D: UsbHostDriver, const DEV: usize = 1> {
    driver: D,
    control: NoopMutex<D::Channel<channel::Control, channel::InOut>>,
    /// Device registry
    devices: NoopMutex<Vec<u8, DEV>>,
}

impl<D: UsbHostDriver> UsbHost<D> {
    pub fn new(driver: D) -> Self {
        let channel = driver.alloc_channel(0, &EndpointDescriptor::control(0, 64), false).ok().unwrap();
        
        Self {
            driver,
            control: Mutex::new(channel),
            devices: Mutex::default(),
        }
    }

    /// Process events and enumerate devices, returns new [Device] or enumeration error
    pub async fn poll(&self) -> Result<Device, HostError> {
        // TODO: Handle devices in hubs
        match self.driver.wait_for_device_event().await {
            DeviceEvent::Connected => {
                self.driver.bus_reset().await;

                // TODO: PRE
                let chan = &mut self.control.lock().await; 
                // After reset device has address 0                
                self.driver.retarget_channel(chan, 0, 8, false)?;
                
                Timer::after_millis(1).await;
                trace!("Request Partial Device Descriptor");
                let max_packet_size0 = {
                    let mut max_retries = 10;
                    loop {
                        match chan
                            .request_descriptor::<DeviceDescriptorPartial, { DeviceDescriptorPartial::SIZE }>()
                            .await
                        {
                            Ok(desc) => break desc.max_packet_size0,
                            Err(_) => {
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
                
                let addr = {
                    let devices = &mut self.devices.lock().await;
                    // Find unused addr
                    let addr = match devices.iter().copied().max().unwrap_or(0).checked_add(1) {
                        Some(a) => a,
                        // Wrapped around
                        None => 1,
                    };
                
                    devices.push(addr).map_err(|_| HostError::OutOfSlots)?;
                    addr
                };
                
                trace!("Set address {}", addr);               
                chan.device_set_address(addr).await?;
                self.driver.retarget_channel(chan, addr, max_packet_size0, false)?;
                
                trace!("Request Device Descriptor");
                let dev_desc = chan
                    .request_descriptor::<DeviceDescriptor, { DeviceDescriptor::SIZE }>()
                    .await?;
                
                trace!("Device Descriptor: {:?}", dev_desc);

                let cfg_desc = chan
                    .request_descriptor::<ConfigurationDescriptor, { ConfigurationDescriptor::SIZE }>()
                    .await?;

                let total_len = cfg_desc.total_len as usize;
                let mut desc_buffer = [0u8; 256];
                let dest_buffer = &mut desc_buffer[0..total_len];

                chan.request_descriptor_bytes::<ConfigurationDescriptor>(dest_buffer)
                    .await?;
                trace!("Full Configuration Descriptor [{}]: {:?}", cfg_desc.total_len, dest_buffer);
                
                chan.set_configuration(cfg_desc.configuration_value).await?;

                match ConfigurationDescriptor::try_from_bytes(&dest_buffer) {
                    Ok(cfg) => {
                        Ok(Device { addr, dev_desc, cfg_desc: cfg })
                    },
                    Err(_) => {
                        Err(HostError::InvalidDescriptor)
                    },
                }
            },
            DeviceEvent::Disconnected => {
                todo!("remove from registry")
            },
        }
    }

    // TODO: Max packet size
    /// Acquire host control channel, configured for device at `addr`
    /// 
    /// This channel must be dropped before using `host.poll()` again
    pub async fn control_channel(
        &self,
        addr: u8,
    ) -> Result<NoopMutexGuard<D::Channel<channel::Control, channel::InOut>>, HostError> { 
        let mut ch = self.control.lock().await;
        self.driver.retarget_channel(&mut ch, addr, 64, false)?;
        Ok(ch)
    }

    pub fn alloc_channel<'h, T: channel::Type, DIR: channel::Direction>(
        &'h self,
        addr: u8,
        endpoint: &EndpointDescriptor
    ) -> Result<Channel<'h, D, T, DIR>, HostError> {
        trace!("Alloc channel for endpoint: {}", endpoint);
        if endpoint.ep_type() != T::ep_type() {
            return Err(HostError::InvalidDescriptor)
        }
        // TODO: PRE
        Ok(Channel {
            channel: self.driver.alloc_channel(addr, endpoint, false)?,
            driver: &self.driver
        })
    }
}


// =============================================

#[cfg(test)]
mod test {
    use super::{ConfigurationDescriptor, USBDescriptor};

    #[test]
    fn test_parse_interface_descriptor() {
        // This configuration descriptor has 2 HID interfaces with HID descriptors
        let desc_bytes = [
            9, 2, 66, 0, 2, 1, 0, 160, 101, 9, 4, 0, 0, 1, 3, 1, 1, 0, 9, 33, 16, 1, 0, 1, 34, 63, 0, 7, 5, 129, 3, 8,
            0, 1, 9, 4, 1, 0, 2, 3, 1, 0, 0, 9, 33, 16, 1, 0, 1, 34, 39, 0, 7, 5, 131, 3, 64, 0, 1, 7, 5, 3, 3, 64, 0,
            1,
        ];

        let cfg = ConfigurationDescriptor::try_from_bytes(desc_bytes.as_slice()).unwrap();
        assert_eq!(cfg.num_interfaces, 2);

        let interface0 = cfg.parse_interface(0).unwrap();
        assert_eq!(interface0.interface_number, 0);

        let interface0_buffer_ref = [9u8, 33, 16, 1, 0, 1, 34, 63, 0, 7, 5, 129, 3, 8, 0, 1];
        assert_eq!(interface0.buffer.len(), interface0_buffer_ref.len());

        let interface1 = cfg.parse_interface(1).unwrap();
        assert_eq!(interface1.interface_number, 1);

        let interface1_buffer_ref = [
            9u8, 33, 16, 1, 0, 1, 34, 39, 0, 7, 5, 131, 3, 64, 0, 1, 7, 5, 3, 3, 64, 0, 1,
        ];
        assert_eq!(interface1.buffer.len(), interface1_buffer_ref.len());
    }

    #[test]
    fn test_parse_endpoint_descriptor() {
        // This configuration descriptor has 2 HID interfaces with HID descriptors
        let desc_bytes = [
            9, 2, 66, 0, 2, 1, 0, 160, 101, 9, 4, 0, 0, 1, 3, 1, 1, 0, 9, 33, 16, 1, 0, 1, 34, 63, 0, 7, 5, 129, 3, 8,
            0, 1, 9, 4, 1, 0, 2, 3, 1, 0, 0, 9, 33, 16, 1, 0, 1, 34, 39, 0, 7, 5, 131, 3, 64, 0, 1, 7, 5, 3, 3, 64, 0,
            1,
        ];

        let cfg = ConfigurationDescriptor::try_from_bytes(desc_bytes.as_slice()).unwrap();
        assert_eq!(cfg.num_interfaces, 2);

        let interface0 = cfg.parse_interface(0).unwrap();
        assert_eq!(interface0.interface_number, 0);

        assert_eq!(interface0.num_endpoints, 1);

        let endpoints = interface0.parse_endpoints::<2>();
        assert_eq!(endpoints.len(), 1);

        let ep = endpoints[0];
        assert_eq!(ep.endpoint_address, 0x81);
        assert_eq!(ep.max_packet_size, 8);

        let interface1 = cfg.parse_interface(1).unwrap();
        assert_eq!(interface1.interface_number, 1);
        assert_eq!(interface1.num_endpoints, 2);

        let endpoints = interface1.parse_endpoints::<2>();
        assert_eq!(endpoints.len(), 2);
    }

    #[test]
    fn test_parse_custom_descriptor() {
        // Define a custom descriptor (HID descriptor in this case)
        struct HIDDescriptor {
            len: u8,
            descriptor_type: u8,
            bcd_hid: u16,
            country_code: u8,
            num_descriptors: u8,
            descriptor_type0: u8,
            descriptor_length0: u16,
        }

        impl USBDescriptor for HIDDescriptor {
            const SIZE: usize = 9; // only valid for 1 descriptor
            const DESC_TYPE: u8 = 33;
            type Error = ();
            fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
                if bytes.len() < Self::SIZE {
                    return Err(());
                }
                if bytes[1] != Self::DESC_TYPE {
                    return Err(());
                }
                Ok(Self {
                    len: bytes[0],
                    descriptor_type: bytes[1],
                    bcd_hid: u16::from_le_bytes([bytes[2], bytes[3]]),
                    country_code: bytes[4],
                    num_descriptors: bytes[5],
                    descriptor_type0: bytes[6],
                    descriptor_length0: u16::from_le_bytes([bytes[7], bytes[8]]),
                })
            }
        }
        // This configuration descriptor has 2 HID interfaces with HID descriptors
        let desc_bytes = [
            9, 2, 66, 0, 2, 1, 0, 160, 101, 9, 4, 0, 0, 1, 3, 1, 1, 0, 9, 33, 16, 1, 0, 1, 34, 63, 0, 7, 5, 129, 3, 8,
            0, 1, 9, 4, 1, 0, 2, 3, 1, 0, 0, 9, 33, 16, 1, 0, 1, 34, 39, 0, 7, 5, 131, 3, 64, 0, 1, 7, 5, 3, 3, 64, 0,
            1,
        ];

        let cfg = ConfigurationDescriptor::try_from_bytes(desc_bytes.as_slice()).unwrap();
        assert_eq!(cfg.num_interfaces, 2);

        let interface0 = cfg.parse_interface(0).unwrap();
        assert_eq!(interface0.interface_number, 0);

        let hid_desc: HIDDescriptor = interface0.parse_class_descriptor().unwrap();

        assert_eq!(hid_desc.len, 9);
        assert_eq!(hid_desc.descriptor_type, 33);

        assert_eq!(hid_desc.bcd_hid, 0x0110);
        assert_eq!(hid_desc.country_code, 0);
        assert_eq!(hid_desc.num_descriptors, 1);
        assert_eq!(hid_desc.descriptor_type0, 34);
        assert_eq!(hid_desc.descriptor_length0, 63);
    }
}
