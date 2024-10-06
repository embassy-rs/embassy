//! USB Host implementation
//!
//! Requires an [USBHostDriver] implementation.
//!

#![allow(async_fn_in_trait)]

use core::marker::PhantomData;

use embassy_futures::select::{select, Either};
use embassy_time::Timer;
use embassy_usb_driver::host::{channel, ChannelError, DeviceEvent, EndpointDescriptor, HostError, RequestType, SetupPacket, UsbChannel, UsbHostDriver};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::{MappedMutexGuard, Mutex, MutexGuard};

use crate::control::Request;

pub mod descriptor;
pub mod hub;

use descriptor::*;

// FIXME: Why is there no alias already?..
type NoopMutex<T> = Mutex<NoopRawMutex, T>;
type NoopMutexGuard<'a, T> = MutexGuard<'a, NoopRawMutex, T>;
type NoopMappedMutexGuard<'a, T> = MappedMutexGuard<'a, NoopRawMutex, T>;

pub struct Device {
    pub addr: u8,
    pub dev_desc: DeviceDescriptor,
    pub cfg_desc: ConfigurationDescriptor,
}

/// Channel is dropped by referenced driver
pub struct Channel<'d, D, T, DIR>
where 
    T: channel::Type,
    DIR: channel::Direction,
    D: UsbHostDriver,
{
    dev_addr: u8,
    channel: D::Channel<T, DIR>,
    driver: &'d D,
    registry: &'d UsbDeviceRegistryRef<'d>
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
        match select(
            self.registry.wait_disconnect(self.dev_addr), 
            self.channel.control_in(setup, buf)
        ).await {
            Either::First(_) => Err(ChannelError::Disconnected),
            Either::Second(res) => res,
        }
    }

    async fn control_out(&mut self, setup: &SetupPacket, buf: &[u8]) -> Result<usize, ChannelError>
    where 
        T: channel::IsControl,
        DIR: channel::IsOut {
        match select(
            self.registry.wait_disconnect(self.dev_addr), 
            self.channel.control_out(setup, buf)
        ).await {
            Either::First(_) => Err(ChannelError::Disconnected),
            Either::Second(res) => res,
        }
    }

    async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, ChannelError>
    where 
        DIR: channel::IsIn {
        match select(
            self.registry.wait_disconnect(self.dev_addr), 
            self.channel.request_in(buf)
        ).await {
            Either::First(_) => Err(ChannelError::Disconnected),
            Either::Second(res) => res,
        }
    }

    async fn request_out(&mut self, buf: &[u8]) -> Result<usize, ChannelError>
    where 
        DIR: channel::IsOut {
        match select(
            self.registry.wait_disconnect(self.dev_addr), 
            self.channel.request_out(buf)
        ).await {
            Either::First(_) => Err(ChannelError::Disconnected),
            Either::Second(res) => res,
        }
    }
} 

/// Extension trait with convenience methods for control channels
pub trait ControlChannelExt<D: channel::Direction>: UsbChannel<channel::Control, D>  {
    // CONTROL IN methods
    /// Request and try to parse the device descriptor.
    async fn request_descriptor<T: USBDescriptor, const SIZE: usize>(
        &mut self, 
        class: bool,
    ) -> Result<T, HostError>
    where D: channel::IsIn
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct DeviceInfo {
    addr: u8,
    needs_pre: bool,
    /// `(hub, port)`
    parent_hub: Option<(u8, u8)>,
}

impl DeviceInfo {
    pub const fn empty() -> Self {
        Self {
            addr: 0,
            needs_pre: false,
            parent_hub: None
        }
    }

    pub fn take(&mut self) -> Self {
        let mut slot = Self::empty();
        core::mem::swap(&mut slot, self);
        slot
    }
}

pub struct UsbDeviceRegistry<const N: usize>([DeviceInfo; N]);

struct UsbDeviceRegistryRef<'a>{ 
    // Change variance
    phantom: PhantomData<&'a ()>,
    mtx: NoopMutex<*mut [DeviceInfo]>,
}

impl<const N: usize> UsbDeviceRegistry<N> {
    pub const fn new() -> Self {
        const { core::assert!(N > 0) }
        Self([const { DeviceInfo::empty() }; N])
    }

    fn by_ref(&mut self) -> UsbDeviceRegistryRef {
        UsbDeviceRegistryRef {
            phantom: PhantomData,
            mtx: Mutex::new(&mut self.0 as *mut _),
        }
    }
} 

impl<'r> UsbDeviceRegistryRef<'r> {
    async fn with_lock<R>(&self, f: impl FnOnce(&mut [DeviceInfo]) -> R) -> R {
        let ptr = self.mtx.lock().await;
        // SAFETY: Protected by mutex
        let slice = unsafe { ptr.as_mut().unwrap() };

        f(slice)
    }
    
    /// Returns `true` if info was added
    pub async fn add_device(&self, info: &DeviceInfo) -> bool {
        // 0 means free slot
        let res = self.find_by_addr(0, |slot| {
            *slot = info.clone()
        }).await;
        res
    }

    /// Find device by address
    pub async fn find_by_addr(&self, addr: u8, modify: impl FnOnce(&mut DeviceInfo)) -> bool {
        let mut found = false;
        self.with_lock(|slice| {
            if let Some(info) = slice.iter_mut().find(|d| d.addr == addr) {
                modify(info);
                found = true;
            }
        }).await;
        found
    }
    
    /// Find address by hub and port
    pub async fn find_by_port(&self, hub_addr: u8, hub_port: u8) -> Option<u8> {
        let mut addr = None;
        self.with_lock(|slice| {
            if let Some(info) = slice.iter_mut()
                .find(|d| d.parent_hub.is_some_and(|h| h.0 == hub_addr && h.1 == hub_port)) {
                addr = Some(info.addr);
            }
        }).await;
        addr
    }

    /// Remove device by address
    /// 
    /// If device is a hub, also remove downstream devices
    /// 
    /// Return count of removed devices
    pub async fn remove_device(&self, addr: u8) -> u8 {
        let mut removed = 0;
        self.with_lock(|slice| {
            if let Some(dev) = slice.iter_mut().find(|d| d.addr == addr) {
                dev.take();
                removed += 1;
            }

            // FIXME/TODO: Chained hubs
            for dev in slice {
                if dev.parent_hub.is_some_and(|h| h.0 == addr) {
                    dev.take();
                    removed += 1;
                }
            }
        }).await;

        removed
    }

    /// Device exists
    pub async fn alive(&self, addr: u8) -> bool {
        self.find_by_addr(addr, |_| {}).await
    }
    
    /// Device needs PRE packet
    pub async fn needs_pre(&self, addr: u8) -> Option<bool> {
        let mut slot = None;
        self.find_by_addr(addr, |dev| { slot.replace(dev.needs_pre); }).await;
        slot
    }

    /// Returns next free address
    pub async fn next_addr(&self) -> u8 {
        let mut ret = 1;
        self.with_lock(|slice| {
            let addr = slice.iter().map(|d| d.addr).max().unwrap().wrapping_add(1);
            
            // Wrapped around
            if addr != 0 {
                ret = addr
            }
        }).await;
        ret
    }

    pub async fn wait_disconnect(&self, addr: u8) {
        loop {
            if !self.alive(addr).await {
                return
            }
            embassy_time::Timer::after_millis(50).await;
        }
    }
}

pub struct UsbHost<'r, D: UsbHostDriver> {
    driver: D,
    control: NoopMutex<D::Channel<channel::Control, channel::InOut>>,
    /// Device registry
    registry: UsbDeviceRegistryRef<'r>,
}

impl<'r, D: UsbHostDriver> UsbHost<'r, D> {
    pub fn new<const N: usize>(driver: D, registry: &'r mut UsbDeviceRegistry<N>) -> Self {
        let channel = driver.alloc_channel(0, &EndpointDescriptor::control(0, 64), false).ok().unwrap();
        
        Self {
            driver,
            control: Mutex::new(channel),
            // Decouple const generic
            registry: registry.by_ref(),
        }
    }

    /// Process events and enumerate devices, returns new [Device] or enumeration error
    pub async fn poll(&self) -> Result<Device, HostError> {
        // TODO: Handle devices in hubs
        loop {
            trace!("Wait for device event");
            match self.driver.wait_for_device_event().await {
                DeviceEvent::Connected => {
                    debug!("Device connected to root");
                    self.driver.bus_reset().await;

                    let chan = &mut self.control.lock().await; 
                    return configure_device(
                        &self.driver, 
                        chan, 
                        &self.registry, 
                        false,
                        None,
                    ).await;
                },
                DeviceEvent::Disconnected => {
                    debug!("Device disconnected from root");

                    // Root device should always have addr 1
                    let count = self.registry.remove_device(1).await;
                    debug!("Disconnected {} devices", count);
                    continue;
                },
            }
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
        let pre = self.registry.needs_pre(addr).await
            .ok_or(HostError::NoSuchDevice)?;
        let packet_size = if pre { 8 } else { 64 };
        self.driver.retarget_channel(&mut ch, addr, packet_size, pre)?;
        Ok(ch)
    }

    pub async fn alloc_channel<'h: 'r, T: channel::Type, DIR: channel::Direction>(
        &'h self,
        addr: u8,
        endpoint: &EndpointDescriptor
    ) -> Result<Channel<'h, D, T, DIR>, HostError> {
        trace!("Alloc channel for endpoint: {}", endpoint);
        if endpoint.ep_type() != T::ep_type() {
            return Err(HostError::InvalidDescriptor)
        }

        let Some(needs_pre) = self.registry.needs_pre(addr).await else {
            return Err(HostError::NoSuchDevice)
        };
        
        Ok(Channel {
            dev_addr: addr,
            channel: self.driver.alloc_channel(addr, endpoint, needs_pre)?,
            driver: &self.driver,
            registry: &self.registry
        })
    }
    
    pub async fn alloc_control_channel<'h: 'r, DIR: channel::Direction>(
        &'h self,
        addr: u8,
    ) -> Result<Channel<'h, D, channel::Control, DIR>, HostError> {
        // TODO: PRE
        self.alloc_channel(addr, &EndpointDescriptor::control(0, 64)).await
    }
}

/// Shared functionality between host and hubs
async fn configure_device<D: UsbHostDriver>(
    driver: &D, 
    chan: &mut D::Channel<channel::Control, channel::InOut>,
    registry: &UsbDeviceRegistryRef<'_>,
    needs_pre: bool,
    parent_hub: Option<(u8, u8)>
) -> Result<Device, HostError> {
    driver.retarget_channel(chan, 0, 8, needs_pre)?;

    Timer::after_millis(1).await;
    trace!("Request Partial Device Descriptor");
    let max_packet_size0 = {
        let mut max_retries = 10;
        loop {
            match chan
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

    let addr = registry.next_addr().await;

    // TODO: Handle errors properly
    trace!("Set address {}", addr);               
    chan.device_set_address(addr).await?;
    driver.retarget_channel(chan, addr, max_packet_size0, needs_pre)?;
    
    if !registry.add_device(&DeviceInfo { 
        addr, 
        needs_pre,
        parent_hub
    }).await {
        // TODO: Log and ignore?
        return Err(HostError::OutOfSlots)
    }

    trace!("Request Device Descriptor");
    let dev_desc = chan
        .request_descriptor::<DeviceDescriptor, { DeviceDescriptor::SIZE }>(false)
        .await?;

    trace!("Device Descriptor: {:?}", dev_desc);

    let cfg_desc = chan
        .request_descriptor::<ConfigurationDescriptor, { ConfigurationDescriptor::SIZE }>(false)
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
}

