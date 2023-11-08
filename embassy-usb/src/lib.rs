#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub use embassy_usb_driver as driver;

mod builder;
pub mod class;
pub mod control;
pub mod descriptor;
mod descriptor_reader;
pub mod msos;
pub mod types;

mod config {
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}

use embassy_futures::select::{select, Either};
use heapless::Vec;

pub use crate::builder::{Builder, Config, FunctionBuilder, InterfaceAltBuilder, InterfaceBuilder};
use crate::config::{MAX_HANDLER_COUNT, MAX_INTERFACE_COUNT};
use crate::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use crate::descriptor::{descriptor_type, lang_id};
use crate::descriptor_reader::foreach_endpoint;
use crate::driver::{Bus, ControlPipe, Direction, Driver, EndpointAddress, Event};
use crate::types::{InterfaceNumber, StringIndex};

/// The global state of the USB device.
///
/// In general class traffic is only possible in the `Configured` state.
#[repr(u8)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UsbDeviceState {
    /// The USB device has no power.
    Unpowered,

    /// The USB device is disabled.
    Disabled,

    /// The USB device has just been enabled or reset.
    Default,

    /// The USB device has received an address from the host.
    Addressed,

    /// The USB device has been configured and is fully functional.
    Configured,
}

/// Error returned by [`UsbDevice::remote_wakeup`].
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RemoteWakeupError {
    /// The USB device is not suspended, or remote wakeup was not enabled.
    InvalidState,
    /// The underlying driver doesn't support remote wakeup.
    Unsupported,
}

impl From<driver::Unsupported> for RemoteWakeupError {
    fn from(_: driver::Unsupported) -> Self {
        RemoteWakeupError::Unsupported
    }
}

/// The bConfiguration value for the not configured state.
pub const CONFIGURATION_NONE: u8 = 0;

/// The bConfiguration value for the single configuration supported by this device.
pub const CONFIGURATION_VALUE: u8 = 1;

const STRING_INDEX_MANUFACTURER: u8 = 1;
const STRING_INDEX_PRODUCT: u8 = 2;
const STRING_INDEX_SERIAL_NUMBER: u8 = 3;
const STRING_INDEX_CUSTOM_START: u8 = 4;

/// Handler for device events and control requests.
///
/// All methods are optional callbacks that will be called by
/// [`UsbDevice::run()`](crate::UsbDevice::run)
pub trait Handler {
    /// Called when the USB device has been enabled or disabled.
    fn enabled(&mut self, _enabled: bool) {}

    /// Called after a USB reset after the bus reset sequence is complete.
    fn reset(&mut self) {}

    /// Called when the host has set the address of the device to `addr`.
    fn addressed(&mut self, _addr: u8) {}

    /// Called when the host has enabled or disabled the configuration of the device.
    fn configured(&mut self, _configured: bool) {}

    /// Called when the bus has entered or exited the suspend state.
    fn suspended(&mut self, _suspended: bool) {}

    /// Called when remote wakeup feature is enabled or disabled.
    fn remote_wakeup_enabled(&mut self, _enabled: bool) {}

    /// Called when a "set alternate setting" control request is done on the interface.
    fn set_alternate_setting(&mut self, iface: InterfaceNumber, alternate_setting: u8) {
        let _ = iface;
        let _ = alternate_setting;
    }

    /// Called when a control request is received with direction HostToDevice.
    ///
    /// # Arguments
    ///
    /// * `req` - The request from the SETUP packet.
    /// * `data` - The data from the request.
    ///
    /// # Returns
    ///
    /// If you didn't handle this request (for example if it's for the wrong interface), return
    /// `None`. In this case, the the USB stack will continue calling the other handlers, to see
    /// if another handles it.
    ///
    /// If you did, return `Some` with either `Accepted` or `Rejected`. This will make the USB stack
    /// respond to the control request, and stop calling other handlers.
    fn control_out(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        let _ = (req, data);
        None
    }

    /// Called when a control request is received with direction DeviceToHost.
    ///
    /// You should write the response somewhere (usually to `buf`, but you may use another buffer
    /// owned by yourself, or a static buffer), then return `InResponse::Accepted(data)`.
    ///
    /// # Arguments
    ///
    /// * `req` - The request from the SETUP packet.
    ///
    /// # Returns
    ///
    /// If you didn't handle this request (for example if it's for the wrong interface), return
    /// `None`. In this case, the the USB stack will continue calling the other handlers, to see
    /// if another handles it.
    ///
    /// If you did, return `Some` with either `Accepted` or `Rejected`. This will make the USB stack
    /// respond to the control request, and stop calling other handlers.
    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        let _ = (req, buf);
        None
    }

    /// Called when a GET_DESCRIPTOR STRING control request is received.
    fn get_string(&mut self, index: StringIndex, lang_id: u16) -> Option<&str> {
        let _ = (index, lang_id);
        None
    }
}

struct Interface {
    current_alt_setting: u8,
    num_alt_settings: u8,
}

/// A report of the used size of the runtime allocated buffers
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UsbBufferReport {
    /// Number of device descriptor bytes used
    pub device_descriptor_used: usize,
    /// Number of config descriptor bytes used
    pub config_descriptor_used: usize,
    /// Number of bos descriptor bytes used
    pub bos_descriptor_used: usize,
    /// Number of msos descriptor bytes used
    pub msos_descriptor_used: usize,
    /// Size of the control buffer
    pub control_buffer_size: usize,
}

/// Main struct for the USB device stack.
pub struct UsbDevice<'d, D: Driver<'d>> {
    control_buf: &'d mut [u8],
    control: D::ControlPipe,
    inner: Inner<'d, D>,
}

struct Inner<'d, D: Driver<'d>> {
    bus: D::Bus,

    config: Config<'d>,
    device_descriptor: &'d [u8],
    config_descriptor: &'d [u8],
    bos_descriptor: &'d [u8],
    msos_descriptor: crate::msos::MsOsDescriptorSet<'d>,

    device_state: UsbDeviceState,
    suspended: bool,
    remote_wakeup_enabled: bool,
    self_powered: bool,

    /// Our device address, or 0 if none.
    address: u8,
    /// SET_ADDRESS requests have special handling depending on the driver.
    /// This flag indicates that requests must be handled by `ControlPipe::accept_set_address()`
    /// instead of regular `accept()`.
    set_address_pending: bool,

    interfaces: Vec<Interface, MAX_INTERFACE_COUNT>,
    handlers: Vec<&'d mut dyn Handler, MAX_HANDLER_COUNT>,
}

impl<'d, D: Driver<'d>> UsbDevice<'d, D> {
    pub(crate) fn build(
        driver: D,
        config: Config<'d>,
        handlers: Vec<&'d mut dyn Handler, MAX_HANDLER_COUNT>,
        device_descriptor: &'d [u8],
        config_descriptor: &'d [u8],
        bos_descriptor: &'d [u8],
        msos_descriptor: crate::msos::MsOsDescriptorSet<'d>,
        interfaces: Vec<Interface, MAX_INTERFACE_COUNT>,
        control_buf: &'d mut [u8],
    ) -> UsbDevice<'d, D> {
        // Start the USB bus.
        // This prevent further allocation by consuming the driver.
        let (bus, control) = driver.start(config.max_packet_size_0 as u16);

        Self {
            control_buf,
            control,
            inner: Inner {
                bus,
                config,
                device_descriptor,
                config_descriptor,
                bos_descriptor,
                msos_descriptor,

                device_state: UsbDeviceState::Unpowered,
                suspended: false,
                remote_wakeup_enabled: false,
                self_powered: false,
                address: 0,
                set_address_pending: false,
                interfaces,
                handlers,
            },
        }
    }

    /// Returns a report of the consumed buffers
    ///
    /// Useful for tuning buffer sizes for actual usage
    pub fn buffer_usage(&self) -> UsbBufferReport {
        UsbBufferReport {
            device_descriptor_used: self.inner.device_descriptor.len(),
            config_descriptor_used: self.inner.config_descriptor.len(),
            bos_descriptor_used: self.inner.bos_descriptor.len(),
            msos_descriptor_used: self.inner.msos_descriptor.len(),
            control_buffer_size: self.control_buf.len(),
        }
    }

    /// Runs the `UsbDevice` forever.
    ///
    /// This future may leave the bus in an invalid state if it is dropped.
    /// After dropping the future, [`UsbDevice::disable()`] should be called
    /// before calling any other `UsbDevice` methods to fully reset the
    /// peripheral.
    pub async fn run(&mut self) -> ! {
        loop {
            self.run_until_suspend().await;
            self.wait_resume().await;
        }
    }

    /// Runs the `UsbDevice` until the bus is suspended.
    ///
    /// This future may leave the bus in an invalid state if it is dropped.
    /// After dropping the future, [`UsbDevice::disable()`] should be called
    /// before calling any other `UsbDevice` methods to fully reset the
    /// peripheral.
    pub async fn run_until_suspend(&mut self) {
        while !self.inner.suspended {
            let control_fut = self.control.setup();
            let bus_fut = self.inner.bus.poll();
            match select(bus_fut, control_fut).await {
                Either::First(evt) => self.inner.handle_bus_event(evt).await,
                Either::Second(req) => self.handle_control(req).await,
            }
        }
    }

    /// Disables the USB peripheral.
    pub async fn disable(&mut self) {
        if self.inner.device_state != UsbDeviceState::Disabled {
            self.inner.bus.disable().await;
            self.inner.device_state = UsbDeviceState::Disabled;
            self.inner.suspended = false;
            self.inner.remote_wakeup_enabled = false;

            for h in &mut self.inner.handlers {
                h.enabled(false);
            }
        }
    }

    /// Waits for a resume condition on the USB bus.
    ///
    /// This future is cancel-safe.
    pub async fn wait_resume(&mut self) {
        while self.inner.suspended {
            let evt = self.inner.bus.poll().await;
            self.inner.handle_bus_event(evt).await;
        }
    }

    /// Initiates a device remote wakeup on the USB bus.
    ///
    /// If the bus is not suspended or remote wakeup is not enabled, an error
    /// will be returned.
    ///
    /// This future may leave the bus in an inconsistent state if dropped.
    /// After dropping the future, [`UsbDevice::disable()`] should be called
    /// before calling any other `UsbDevice` methods to fully reset the peripheral.
    pub async fn remote_wakeup(&mut self) -> Result<(), RemoteWakeupError> {
        if self.inner.suspended && self.inner.remote_wakeup_enabled {
            self.inner.bus.remote_wakeup().await?;
            self.inner.suspended = false;

            for h in &mut self.inner.handlers {
                h.suspended(false);
            }

            Ok(())
        } else {
            Err(RemoteWakeupError::InvalidState)
        }
    }

    async fn handle_control(&mut self, req: [u8; 8]) {
        let req = Request::parse(&req);

        trace!("control request: {:?}", req);

        match req.direction {
            Direction::In => self.handle_control_in(req).await,
            Direction::Out => self.handle_control_out(req).await,
        }
    }

    async fn handle_control_in(&mut self, req: Request) {
        const DEVICE_DESCRIPTOR_LEN: usize = 18;

        let mut resp_length = req.length as usize;
        let max_packet_size = self.control.max_packet_size();

        // If we don't have an address yet, respond with max 1 packet.
        // The host doesn't know our EP0 max packet size yet, and might assume
        // a full-length packet is a short packet, thinking we're done sending data.
        // See https://github.com/hathach/tinyusb/issues/184
        if self.inner.address == 0 && max_packet_size < DEVICE_DESCRIPTOR_LEN && max_packet_size < resp_length {
            trace!("received control req while not addressed: capping response to 1 packet.");
            resp_length = max_packet_size;
        }

        match self.inner.handle_control_in(req, self.control_buf) {
            InResponse::Accepted(data) => {
                let len = data.len().min(resp_length);
                let need_zlp = len != resp_length && (len % max_packet_size) == 0;

                let chunks = data[0..len]
                    .chunks(max_packet_size)
                    .chain(need_zlp.then(|| -> &[u8] { &[] }));

                for (first, last, chunk) in first_last(chunks) {
                    match self.control.data_in(chunk, first, last).await {
                        Ok(()) => {}
                        Err(e) => {
                            warn!("control accept_in failed: {:?}", e);
                            return;
                        }
                    }
                }
            }
            InResponse::Rejected => self.control.reject().await,
        }
    }

    async fn handle_control_out(&mut self, req: Request) {
        let req_length = req.length as usize;
        let max_packet_size = self.control.max_packet_size();
        let mut total = 0;

        if req_length > self.control_buf.len() {
            warn!(
                "got CONTROL OUT with length {} higher than the control_buf len {}, rejecting.",
                req_length,
                self.control_buf.len()
            );
            self.control.reject().await;
            return;
        }

        let chunks = self.control_buf[..req_length].chunks_mut(max_packet_size);
        for (first, last, chunk) in first_last(chunks) {
            let size = match self.control.data_out(chunk, first, last).await {
                Ok(x) => x,
                Err(e) => {
                    warn!("usb: failed to read CONTROL OUT data stage: {:?}", e);
                    return;
                }
            };
            total += size;
            if size < max_packet_size || total == req_length {
                break;
            }
        }

        let data = &self.control_buf[0..total];
        #[cfg(feature = "defmt")]
        trace!("  control out data: {:02x}", data);
        #[cfg(not(feature = "defmt"))]
        trace!("  control out data: {:02x?}", data);

        match self.inner.handle_control_out(req, data) {
            OutResponse::Accepted => {
                if self.inner.set_address_pending {
                    self.control.accept_set_address(self.inner.address).await;
                    self.inner.set_address_pending = false;
                } else {
                    self.control.accept().await;
                }
            }
            OutResponse::Rejected => self.control.reject().await,
        }
    }
}

impl<'d, D: Driver<'d>> Inner<'d, D> {
    async fn handle_bus_event(&mut self, evt: Event) {
        match evt {
            Event::Reset => {
                trace!("usb: reset");
                self.device_state = UsbDeviceState::Default;
                self.suspended = false;
                self.remote_wakeup_enabled = false;
                self.address = 0;

                for h in &mut self.handlers {
                    h.reset();
                }

                for (i, iface) in self.interfaces.iter_mut().enumerate() {
                    iface.current_alt_setting = 0;

                    for h in &mut self.handlers {
                        h.set_alternate_setting(InterfaceNumber::new(i as _), 0);
                    }
                }
            }
            Event::Resume => {
                trace!("usb: resume");
                self.suspended = false;
                for h in &mut self.handlers {
                    h.suspended(false);
                }
            }
            Event::Suspend => {
                trace!("usb: suspend");
                self.suspended = true;
                for h in &mut self.handlers {
                    h.suspended(true);
                }
            }
            Event::PowerDetected => {
                trace!("usb: power detected");
                self.bus.enable().await;
                self.device_state = UsbDeviceState::Default;

                for h in &mut self.handlers {
                    h.enabled(true);
                }
            }
            Event::PowerRemoved => {
                trace!("usb: power removed");
                self.bus.disable().await;
                self.device_state = UsbDeviceState::Unpowered;

                for h in &mut self.handlers {
                    h.enabled(false);
                }
            }
        }
    }

    fn handle_control_out(&mut self, req: Request, data: &[u8]) -> OutResponse {
        const CONFIGURATION_NONE_U16: u16 = CONFIGURATION_NONE as u16;
        const CONFIGURATION_VALUE_U16: u16 = CONFIGURATION_VALUE as u16;

        match (req.request_type, req.recipient) {
            (RequestType::Standard, Recipient::Device) => match (req.request, req.value) {
                (Request::CLEAR_FEATURE, Request::FEATURE_DEVICE_REMOTE_WAKEUP) => {
                    self.remote_wakeup_enabled = false;
                    for h in &mut self.handlers {
                        h.remote_wakeup_enabled(false);
                    }
                    OutResponse::Accepted
                }
                (Request::SET_FEATURE, Request::FEATURE_DEVICE_REMOTE_WAKEUP) => {
                    self.remote_wakeup_enabled = true;
                    for h in &mut self.handlers {
                        h.remote_wakeup_enabled(true);
                    }
                    OutResponse::Accepted
                }
                (Request::SET_ADDRESS, addr @ 1..=127) => {
                    self.address = addr as u8;
                    self.set_address_pending = true;
                    self.device_state = UsbDeviceState::Addressed;
                    for h in &mut self.handlers {
                        h.addressed(self.address);
                    }
                    OutResponse::Accepted
                }
                (Request::SET_CONFIGURATION, CONFIGURATION_VALUE_U16) => {
                    debug!("SET_CONFIGURATION: configured");
                    self.device_state = UsbDeviceState::Configured;

                    // Enable all endpoints of selected alt settings.
                    foreach_endpoint(self.config_descriptor, |ep| {
                        let iface = &self.interfaces[ep.interface.0 as usize];
                        self.bus
                            .endpoint_set_enabled(ep.ep_address, iface.current_alt_setting == ep.interface_alt);
                    })
                    .unwrap();

                    // Notify handlers.
                    for h in &mut self.handlers {
                        h.configured(true);
                    }

                    OutResponse::Accepted
                }
                (Request::SET_CONFIGURATION, CONFIGURATION_NONE_U16) => {
                    if self.device_state != UsbDeviceState::Default {
                        debug!("SET_CONFIGURATION: unconfigured");
                        self.device_state = UsbDeviceState::Addressed;

                        // Disable all endpoints.
                        foreach_endpoint(self.config_descriptor, |ep| {
                            self.bus.endpoint_set_enabled(ep.ep_address, false);
                        })
                        .unwrap();

                        // Notify handlers.
                        for h in &mut self.handlers {
                            h.configured(false);
                        }
                    }
                    OutResponse::Accepted
                }
                _ => OutResponse::Rejected,
            },
            (RequestType::Standard, Recipient::Interface) => {
                let iface_num = InterfaceNumber::new(req.index as _);
                let Some(iface) = self.interfaces.get_mut(iface_num.0 as usize) else {
                    return OutResponse::Rejected;
                };

                match req.request {
                    Request::SET_INTERFACE => {
                        let new_altsetting = req.value as u8;

                        if new_altsetting >= iface.num_alt_settings {
                            warn!("SET_INTERFACE: trying to select alt setting out of range.");
                            return OutResponse::Rejected;
                        }

                        iface.current_alt_setting = new_altsetting;

                        // Enable/disable EPs of this interface as needed.
                        foreach_endpoint(self.config_descriptor, |ep| {
                            if ep.interface == iface_num {
                                self.bus
                                    .endpoint_set_enabled(ep.ep_address, iface.current_alt_setting == ep.interface_alt);
                            }
                        })
                        .unwrap();

                        // TODO check it is valid (not out of range)

                        for h in &mut self.handlers {
                            h.set_alternate_setting(iface_num, new_altsetting);
                        }
                        OutResponse::Accepted
                    }
                    _ => OutResponse::Rejected,
                }
            }
            (RequestType::Standard, Recipient::Endpoint) => match (req.request, req.value) {
                (Request::SET_FEATURE, Request::FEATURE_ENDPOINT_HALT) => {
                    let ep_addr = ((req.index as u8) & 0x8f).into();
                    self.bus.endpoint_set_stalled(ep_addr, true);
                    OutResponse::Accepted
                }
                (Request::CLEAR_FEATURE, Request::FEATURE_ENDPOINT_HALT) => {
                    let ep_addr = ((req.index as u8) & 0x8f).into();
                    self.bus.endpoint_set_stalled(ep_addr, false);
                    OutResponse::Accepted
                }
                _ => OutResponse::Rejected,
            },
            _ => self.handle_control_out_delegated(req, data),
        }
    }

    fn handle_control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> InResponse<'a> {
        match (req.request_type, req.recipient) {
            (RequestType::Standard, Recipient::Device) => match req.request {
                Request::GET_STATUS => {
                    let mut status: u16 = 0x0000;
                    if self.self_powered {
                        status |= 0x0001;
                    }
                    if self.remote_wakeup_enabled {
                        status |= 0x0002;
                    }
                    buf[..2].copy_from_slice(&status.to_le_bytes());
                    InResponse::Accepted(&buf[..2])
                }
                Request::GET_DESCRIPTOR => self.handle_get_descriptor(req, buf),
                Request::GET_CONFIGURATION => {
                    let status = match self.device_state {
                        UsbDeviceState::Configured => CONFIGURATION_VALUE,
                        _ => CONFIGURATION_NONE,
                    };
                    buf[0] = status;
                    InResponse::Accepted(&buf[..1])
                }
                _ => InResponse::Rejected,
            },
            (RequestType::Standard, Recipient::Interface) => {
                let Some(iface) = self.interfaces.get_mut(req.index as usize) else {
                    return InResponse::Rejected;
                };

                match req.request {
                    Request::GET_STATUS => {
                        let status: u16 = 0;
                        buf[..2].copy_from_slice(&status.to_le_bytes());
                        InResponse::Accepted(&buf[..2])
                    }
                    Request::GET_INTERFACE => {
                        buf[0] = iface.current_alt_setting;
                        InResponse::Accepted(&buf[..1])
                    }
                    _ => self.handle_control_in_delegated(req, buf),
                }
            }
            (RequestType::Standard, Recipient::Endpoint) => match req.request {
                Request::GET_STATUS => {
                    let ep_addr: EndpointAddress = ((req.index as u8) & 0x8f).into();
                    let mut status: u16 = 0x0000;
                    if self.bus.endpoint_is_stalled(ep_addr) {
                        status |= 0x0001;
                    }
                    buf[..2].copy_from_slice(&status.to_le_bytes());
                    InResponse::Accepted(&buf[..2])
                }
                _ => InResponse::Rejected,
            },

            (RequestType::Vendor, Recipient::Device) => {
                if !self.msos_descriptor.is_empty()
                    && req.request == self.msos_descriptor.vendor_code()
                    && req.index == 7
                {
                    // Index 7 retrieves the MS OS Descriptor Set
                    InResponse::Accepted(self.msos_descriptor.descriptor())
                } else {
                    self.handle_control_in_delegated(req, buf)
                }
            }
            _ => self.handle_control_in_delegated(req, buf),
        }
    }

    fn handle_control_out_delegated(&mut self, req: Request, data: &[u8]) -> OutResponse {
        for h in &mut self.handlers {
            if let Some(res) = h.control_out(req, data) {
                return res;
            }
        }
        OutResponse::Rejected
    }

    fn handle_control_in_delegated<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> InResponse<'a> {
        unsafe fn extend_lifetime<'y>(r: InResponse<'_>) -> InResponse<'y> {
            core::mem::transmute(r)
        }

        for h in &mut self.handlers {
            if let Some(res) = h.control_in(req, buf) {
                // safety: the borrow checker isn't smart enough to know this pattern (returning a
                // borrowed value from inside the loop) is sound. Workaround by unsafely extending lifetime.
                // Also, Polonius (the WIP new borrow checker) does accept it.

                return unsafe { extend_lifetime(res) };
            }
        }
        InResponse::Rejected
    }

    fn handle_get_descriptor<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> InResponse<'a> {
        let (dtype, index) = req.descriptor_type_index();

        match dtype {
            descriptor_type::BOS => InResponse::Accepted(self.bos_descriptor),
            descriptor_type::DEVICE => InResponse::Accepted(self.device_descriptor),
            descriptor_type::CONFIGURATION => InResponse::Accepted(self.config_descriptor),
            descriptor_type::STRING => {
                if index == 0 {
                    buf[0] = 4; // len
                    buf[1] = descriptor_type::STRING;
                    buf[2] = lang_id::ENGLISH_US as u8;
                    buf[3] = (lang_id::ENGLISH_US >> 8) as u8;
                    InResponse::Accepted(&buf[..4])
                } else {
                    let s = match index {
                        STRING_INDEX_MANUFACTURER => self.config.manufacturer,
                        STRING_INDEX_PRODUCT => self.config.product,
                        STRING_INDEX_SERIAL_NUMBER => self.config.serial_number,
                        _ => {
                            let mut s = None;
                            for handler in &mut self.handlers {
                                let index = StringIndex::new(index);
                                let lang_id = req.index;
                                if let Some(res) = handler.get_string(index, lang_id) {
                                    s = Some(res);
                                    break;
                                }
                            }
                            s
                        }
                    };

                    if let Some(s) = s {
                        assert!(buf.len() >= 2, "control buffer too small");

                        buf[1] = descriptor_type::STRING;
                        let mut pos = 2;
                        for c in s.encode_utf16() {
                            assert!(pos + 2 < buf.len(), "control buffer too small");

                            buf[pos..pos + 2].copy_from_slice(&c.to_le_bytes());
                            pos += 2;
                        }

                        buf[0] = pos as u8;
                        InResponse::Accepted(&buf[..pos])
                    } else {
                        InResponse::Rejected
                    }
                }
            }
            _ => InResponse::Rejected,
        }
    }
}

fn first_last<T: Iterator>(iter: T) -> impl Iterator<Item = (bool, bool, T::Item)> {
    let mut iter = iter.peekable();
    let mut first = true;
    core::iter::from_fn(move || {
        let val = iter.next()?;
        let is_first = first;
        first = false;
        let is_last = iter.peek().is_none();
        Some((is_first, is_last, val))
    })
}
