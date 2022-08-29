#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod builder;
pub mod control;
pub mod descriptor;
mod descriptor_reader;
pub mod driver;
pub mod types;

use embassy_futures::select::{select, Either};
use heapless::Vec;

pub use self::builder::{Builder, Config};
use self::control::*;
use self::descriptor::*;
use self::driver::{Bus, Driver, Event};
use self::types::*;
use crate::descriptor_reader::foreach_endpoint;
use crate::driver::ControlPipe;

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

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RemoteWakeupError {
    InvalidState,
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

pub const MAX_INTERFACE_COUNT: usize = 4;

const STRING_INDEX_MANUFACTURER: u8 = 1;
const STRING_INDEX_PRODUCT: u8 = 2;
const STRING_INDEX_SERIAL_NUMBER: u8 = 3;
const STRING_INDEX_CUSTOM_START: u8 = 4;

/// A handler trait for changes in the device state of the [UsbDevice].
pub trait DeviceStateHandler {
    /// Called when the USB device has been enabled or disabled.
    fn enabled(&self, _enabled: bool) {}

    /// Called when the host resets the device.
    fn reset(&self) {}

    /// Called when the host has set the address of the device to `addr`.
    fn addressed(&self, _addr: u8) {}

    /// Called when the host has enabled or disabled the configuration of the device.
    fn configured(&self, _configured: bool) {}

    /// Called when the bus has entered or exited the suspend state.
    fn suspended(&self, _suspended: bool) {}

    /// Called when remote wakeup feature is enabled or disabled.
    fn remote_wakeup_enabled(&self, _enabled: bool) {}
}

struct Interface<'d> {
    handler: Option<&'d mut dyn ControlHandler>,
    current_alt_setting: u8,
    num_alt_settings: u8,
    num_strings: u8,
}

pub struct UsbDevice<'d, D: Driver<'d>> {
    control_buf: &'d mut [u8],
    control: D::ControlPipe,
    inner: Inner<'d, D>,
}

struct Inner<'d, D: Driver<'d>> {
    bus: D::Bus,
    handler: Option<&'d dyn DeviceStateHandler>,

    config: Config<'d>,
    device_descriptor: &'d [u8],
    config_descriptor: &'d [u8],
    bos_descriptor: &'d [u8],

    device_state: UsbDeviceState,
    suspended: bool,
    remote_wakeup_enabled: bool,
    self_powered: bool,

    /// Our device address, or 0 if none.
    address: u8,
    /// When receiving a set addr control request, we have to apply it AFTER we've
    /// finished handling the control request, as the status stage still has to be
    /// handled with addr 0.
    /// If true, do a set_addr after finishing the current control req.
    set_address_pending: bool,

    interfaces: Vec<Interface<'d>, MAX_INTERFACE_COUNT>,
}

impl<'d, D: Driver<'d>> UsbDevice<'d, D> {
    pub(crate) fn build(
        driver: D,
        config: Config<'d>,
        handler: Option<&'d dyn DeviceStateHandler>,
        device_descriptor: &'d [u8],
        config_descriptor: &'d [u8],
        bos_descriptor: &'d [u8],
        interfaces: Vec<Interface<'d>, MAX_INTERFACE_COUNT>,
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
                handler,
                device_descriptor,
                config_descriptor,
                bos_descriptor,

                device_state: UsbDeviceState::Unpowered,
                suspended: false,
                remote_wakeup_enabled: false,
                self_powered: false,
                address: 0,
                set_address_pending: false,
                interfaces,
            },
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
    pub async fn run_until_suspend(&mut self) -> () {
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

            if let Some(h) = &self.inner.handler {
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

            if let Some(h) = &self.inner.handler {
                h.suspended(false);
            }

            Ok(())
        } else {
            Err(RemoteWakeupError::InvalidState)
        }
    }

    async fn handle_control(&mut self, req: [u8; 8]) {
        let req = Request::parse(&req);

        trace!("control request: {:02x}", req);

        match req.direction {
            UsbDirection::In => self.handle_control_in(req).await,
            UsbDirection::Out => self.handle_control_out(req).await,
        }

        if self.inner.set_address_pending {
            self.inner.bus.set_address(self.inner.address);
            self.inner.set_address_pending = false;
        }
    }

    async fn handle_control_in(&mut self, req: Request) {
        let mut resp_length = req.length as usize;
        let max_packet_size = self.control.max_packet_size();

        // If we don't have an address yet, respond with max 1 packet.
        // The host doesn't know our EP0 max packet size yet, and might assume
        // a full-length packet is a short packet, thinking we're done sending data.
        // See https://github.com/hathach/tinyusb/issues/184
        const DEVICE_DESCRIPTOR_LEN: usize = 18;
        if self.inner.address == 0
            && max_packet_size < DEVICE_DESCRIPTOR_LEN
            && (max_packet_size as usize) < resp_length
        {
            trace!("received control req while not addressed: capping response to 1 packet.");
            resp_length = max_packet_size;
        }

        match self.inner.handle_control_in(req, &mut self.control_buf) {
            InResponse::Accepted(data) => {
                let len = data.len().min(resp_length);
                let need_zlp = len != resp_length && (len % usize::from(max_packet_size)) == 0;

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
            OutResponse::Accepted => self.control.accept().await,
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

                for iface in self.interfaces.iter_mut() {
                    iface.current_alt_setting = 0;
                    if let Some(h) = &mut iface.handler {
                        h.reset();
                        h.set_alternate_setting(0);
                    }
                }

                if let Some(h) = &self.handler {
                    h.reset();
                }
            }
            Event::Resume => {
                trace!("usb: resume");
                self.suspended = false;
                if let Some(h) = &self.handler {
                    h.suspended(false);
                }
            }
            Event::Suspend => {
                trace!("usb: suspend");
                self.suspended = true;
                if let Some(h) = &self.handler {
                    h.suspended(true);
                }
            }
            Event::PowerDetected => {
                trace!("usb: power detected");
                self.bus.enable().await;
                self.device_state = UsbDeviceState::Default;

                if let Some(h) = &self.handler {
                    h.enabled(true);
                }
            }
            Event::PowerRemoved => {
                trace!("usb: power removed");
                self.bus.disable().await;
                self.device_state = UsbDeviceState::Unpowered;

                if let Some(h) = &self.handler {
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
                    if let Some(h) = &self.handler {
                        h.remote_wakeup_enabled(false);
                    }
                    OutResponse::Accepted
                }
                (Request::SET_FEATURE, Request::FEATURE_DEVICE_REMOTE_WAKEUP) => {
                    self.remote_wakeup_enabled = true;
                    if let Some(h) = &self.handler {
                        h.remote_wakeup_enabled(true);
                    }
                    OutResponse::Accepted
                }
                (Request::SET_ADDRESS, addr @ 1..=127) => {
                    self.address = addr as u8;
                    self.set_address_pending = true;
                    self.device_state = UsbDeviceState::Addressed;
                    if let Some(h) = &self.handler {
                        h.addressed(self.address);
                    }
                    OutResponse::Accepted
                }
                (Request::SET_CONFIGURATION, CONFIGURATION_VALUE_U16) => {
                    debug!("SET_CONFIGURATION: configured");
                    self.device_state = UsbDeviceState::Configured;

                    // Enable all endpoints of selected alt settings.
                    foreach_endpoint(self.config_descriptor, |ep| {
                        let iface = &self.interfaces[ep.interface as usize];
                        self.bus
                            .endpoint_set_enabled(ep.ep_address, iface.current_alt_setting == ep.interface_alt);
                    })
                    .unwrap();

                    // Notify handler.
                    if let Some(h) = &self.handler {
                        h.configured(true);
                    }

                    OutResponse::Accepted
                }
                (Request::SET_CONFIGURATION, CONFIGURATION_NONE_U16) => match self.device_state {
                    UsbDeviceState::Default => OutResponse::Accepted,
                    _ => {
                        debug!("SET_CONFIGURATION: unconfigured");
                        self.device_state = UsbDeviceState::Addressed;

                        // Disable all endpoints.
                        foreach_endpoint(self.config_descriptor, |ep| {
                            self.bus.endpoint_set_enabled(ep.ep_address, false);
                        })
                        .unwrap();

                        // Notify handler.
                        if let Some(h) = &self.handler {
                            h.configured(false);
                        }

                        OutResponse::Accepted
                    }
                },
                _ => OutResponse::Rejected,
            },
            (RequestType::Standard, Recipient::Interface) => {
                let iface = match self.interfaces.get_mut(req.index as usize) {
                    Some(iface) => iface,
                    None => return OutResponse::Rejected,
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
                            if ep.interface == req.index as u8 {
                                self.bus
                                    .endpoint_set_enabled(ep.ep_address, iface.current_alt_setting == ep.interface_alt);
                            }
                        })
                        .unwrap();

                        // TODO check it is valid (not out of range)
                        // TODO actually enable/disable endpoints.

                        if let Some(handler) = &mut iface.handler {
                            handler.set_alternate_setting(new_altsetting);
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
            (RequestType::Class, Recipient::Interface) => {
                let iface = match self.interfaces.get_mut(req.index as usize) {
                    Some(iface) => iface,
                    None => return OutResponse::Rejected,
                };
                match &mut iface.handler {
                    Some(handler) => handler.control_out(req, data),
                    None => OutResponse::Rejected,
                }
            }
            _ => OutResponse::Rejected,
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
                let iface = match self.interfaces.get_mut(req.index as usize) {
                    Some(iface) => iface,
                    None => return InResponse::Rejected,
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
                    Request::GET_DESCRIPTOR => match &mut iface.handler {
                        Some(handler) => handler.get_descriptor(req, buf),
                        None => InResponse::Rejected,
                    },
                    _ => InResponse::Rejected,
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
            (RequestType::Class, Recipient::Interface) => {
                let iface = match self.interfaces.get_mut(req.index as usize) {
                    Some(iface) => iface,
                    None => return InResponse::Rejected,
                };

                match &mut iface.handler {
                    Some(handler) => handler.control_in(req, buf),
                    None => InResponse::Rejected,
                }
            }
            _ => InResponse::Rejected,
        }
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
                            // Find out which iface owns this string index.
                            let mut index_left = index - STRING_INDEX_CUSTOM_START;
                            let mut the_iface = None;
                            for iface in &mut self.interfaces {
                                if index_left < iface.num_strings {
                                    the_iface = Some(iface);
                                    break;
                                }
                                index_left -= iface.num_strings;
                            }

                            if let Some(iface) = the_iface {
                                if let Some(handler) = &mut iface.handler {
                                    let index = StringIndex::new(index);
                                    let lang_id = req.index;
                                    handler.get_string(index, lang_id)
                                } else {
                                    warn!("String requested to an interface with no handler.");
                                    None
                                }
                            } else {
                                warn!("String requested but didn't match to an interface.");
                                None
                            }
                        }
                    };

                    if let Some(s) = s {
                        if buf.len() < 2 {
                            panic!("control buffer too small");
                        }

                        buf[1] = descriptor_type::STRING;
                        let mut pos = 2;
                        for c in s.encode_utf16() {
                            if pos >= buf.len() {
                                panic!("control buffer too small");
                            }

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
