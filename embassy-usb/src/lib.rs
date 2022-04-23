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

use embassy::util::{select, Either};
use heapless::Vec;

use crate::descriptor_reader::foreach_endpoint;

use self::control::*;
use self::descriptor::*;
use self::driver::{Bus, Driver, Event};
use self::types::*;

pub use self::builder::Builder;
pub use self::builder::Config;

/// The global state of the USB device.
///
/// In general class traffic is only possible in the `Configured` state.
#[repr(u8)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UsbDeviceState {
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
    bus: D::Bus,
    handler: Option<&'d dyn DeviceStateHandler>,
    control: ControlPipe<D::ControlPipe>,

    config: Config<'d>,
    device_descriptor: &'d [u8],
    config_descriptor: &'d [u8],
    bos_descriptor: &'d [u8],
    control_buf: &'d mut [u8],

    device_state: UsbDeviceState,
    suspended: bool,
    remote_wakeup_enabled: bool,
    self_powered: bool,
    pending_address: u8,

    interfaces: Vec<Interface<'d>, MAX_INTERFACE_COUNT>,
}

impl<'d, D: Driver<'d>> UsbDevice<'d, D> {
    pub(crate) fn build(
        mut driver: D,
        config: Config<'d>,
        handler: Option<&'d dyn DeviceStateHandler>,
        device_descriptor: &'d [u8],
        config_descriptor: &'d [u8],
        bos_descriptor: &'d [u8],
        interfaces: Vec<Interface<'d>, MAX_INTERFACE_COUNT>,
        control_buf: &'d mut [u8],
    ) -> UsbDevice<'d, D> {
        let control = driver
            .alloc_control_pipe(config.max_packet_size_0 as u16)
            .expect("failed to alloc control endpoint");

        // Enable the USB bus.
        // This prevent further allocation by consuming the driver.
        let bus = driver.into_bus();

        Self {
            bus,
            config,
            handler,
            control: ControlPipe::new(control),
            device_descriptor,
            config_descriptor,
            bos_descriptor,
            control_buf,
            device_state: UsbDeviceState::Disabled,
            suspended: false,
            remote_wakeup_enabled: false,
            self_powered: false,
            pending_address: 0,
            interfaces,
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
        if self.device_state == UsbDeviceState::Disabled {
            self.bus.enable().await;
            self.device_state = UsbDeviceState::Default;

            if let Some(h) = &self.handler {
                h.enabled(true);
            }
        }

        loop {
            let control_fut = self.control.setup();
            let bus_fut = self.bus.poll();
            match select(bus_fut, control_fut).await {
                Either::First(evt) => {
                    self.handle_bus_event(evt);
                    if self.suspended {
                        return;
                    }
                }
                Either::Second(req) => match req {
                    Setup::DataIn(req, stage) => self.handle_control_in(req, stage).await,
                    Setup::DataOut(req, stage) => self.handle_control_out(req, stage).await,
                },
            }
        }
    }

    /// Disables the USB peripheral.
    pub async fn disable(&mut self) {
        if self.device_state != UsbDeviceState::Disabled {
            self.bus.disable().await;
            self.device_state = UsbDeviceState::Disabled;
            self.suspended = false;
            self.remote_wakeup_enabled = false;

            if let Some(h) = &self.handler {
                h.enabled(false);
            }
        }
    }

    /// Waits for a resume condition on the USB bus.
    ///
    /// This future is cancel-safe.
    pub async fn wait_resume(&mut self) {
        while self.suspended {
            let evt = self.bus.poll().await;
            self.handle_bus_event(evt);
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
        if self.suspended && self.remote_wakeup_enabled {
            self.bus.remote_wakeup().await?;
            self.suspended = false;

            if let Some(h) = &self.handler {
                h.suspended(false);
            }

            Ok(())
        } else {
            Err(RemoteWakeupError::InvalidState)
        }
    }

    fn handle_bus_event(&mut self, evt: Event) {
        match evt {
            Event::Reset => {
                trace!("usb: reset");
                self.device_state = UsbDeviceState::Default;
                self.suspended = false;
                self.remote_wakeup_enabled = false;
                self.pending_address = 0;

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
        }
    }

    async fn handle_control_out(&mut self, req: Request, stage: DataOutStage) {
        const CONFIGURATION_NONE_U16: u16 = CONFIGURATION_NONE as u16;
        const CONFIGURATION_VALUE_U16: u16 = CONFIGURATION_VALUE as u16;

        let (data, stage) = match self.control.data_out(self.control_buf, stage).await {
            Ok(data) => data,
            Err(_) => {
                warn!("usb: failed to read CONTROL OUT data stage.");
                return;
            }
        };

        match (req.request_type, req.recipient) {
            (RequestType::Standard, Recipient::Device) => match (req.request, req.value) {
                (Request::CLEAR_FEATURE, Request::FEATURE_DEVICE_REMOTE_WAKEUP) => {
                    self.remote_wakeup_enabled = false;
                    if let Some(h) = &self.handler {
                        h.remote_wakeup_enabled(false);
                    }
                    self.control.accept(stage)
                }
                (Request::SET_FEATURE, Request::FEATURE_DEVICE_REMOTE_WAKEUP) => {
                    self.remote_wakeup_enabled = true;
                    if let Some(h) = &self.handler {
                        h.remote_wakeup_enabled(true);
                    }
                    self.control.accept(stage)
                }
                (Request::SET_ADDRESS, addr @ 1..=127) => {
                    self.pending_address = addr as u8;
                    self.bus.set_address(self.pending_address);
                    self.device_state = UsbDeviceState::Addressed;
                    if let Some(h) = &self.handler {
                        h.addressed(self.pending_address);
                    }
                    self.control.accept(stage)
                }
                (Request::SET_CONFIGURATION, CONFIGURATION_VALUE_U16) => {
                    debug!("SET_CONFIGURATION: configured");
                    self.device_state = UsbDeviceState::Configured;

                    // Enable all endpoints of selected alt settings.
                    foreach_endpoint(self.config_descriptor, |ep| {
                        let iface = &self.interfaces[ep.interface as usize];
                        self.bus.endpoint_set_enabled(
                            ep.ep_address,
                            iface.current_alt_setting == ep.interface_alt,
                        );
                    })
                    .unwrap();

                    // Notify handler.
                    if let Some(h) = &self.handler {
                        h.configured(true);
                    }

                    self.control.accept(stage)
                }
                (Request::SET_CONFIGURATION, CONFIGURATION_NONE_U16) => match self.device_state {
                    UsbDeviceState::Default => self.control.accept(stage),
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

                        self.control.accept(stage)
                    }
                },
                _ => self.control.reject(),
            },
            (RequestType::Standard, Recipient::Interface) => {
                let iface = match self.interfaces.get_mut(req.index as usize) {
                    Some(iface) => iface,
                    None => return self.control.reject(),
                };

                match req.request {
                    Request::SET_INTERFACE => {
                        let new_altsetting = req.value as u8;

                        if new_altsetting >= iface.num_alt_settings {
                            warn!("SET_INTERFACE: trying to select alt setting out of range.");
                            return self.control.reject();
                        }

                        iface.current_alt_setting = new_altsetting;

                        // Enable/disable EPs of this interface as needed.
                        foreach_endpoint(self.config_descriptor, |ep| {
                            if ep.interface == req.index as u8 {
                                self.bus.endpoint_set_enabled(
                                    ep.ep_address,
                                    iface.current_alt_setting == ep.interface_alt,
                                );
                            }
                        })
                        .unwrap();

                        // TODO check it is valid (not out of range)
                        // TODO actually enable/disable endpoints.

                        if let Some(handler) = &mut iface.handler {
                            handler.set_alternate_setting(new_altsetting);
                        }
                        self.control.accept(stage)
                    }
                    _ => self.control.reject(),
                }
            }
            (RequestType::Standard, Recipient::Endpoint) => match (req.request, req.value) {
                (Request::SET_FEATURE, Request::FEATURE_ENDPOINT_HALT) => {
                    let ep_addr = ((req.index as u8) & 0x8f).into();
                    self.bus.endpoint_set_stalled(ep_addr, true);
                    self.control.accept(stage)
                }
                (Request::CLEAR_FEATURE, Request::FEATURE_ENDPOINT_HALT) => {
                    let ep_addr = ((req.index as u8) & 0x8f).into();
                    self.bus.endpoint_set_stalled(ep_addr, false);
                    self.control.accept(stage)
                }
                _ => self.control.reject(),
            },
            (RequestType::Class, Recipient::Interface) => {
                let iface = match self.interfaces.get_mut(req.index as usize) {
                    Some(iface) => iface,
                    None => return self.control.reject(),
                };
                match &mut iface.handler {
                    Some(handler) => match handler.control_out(req, data) {
                        OutResponse::Accepted => self.control.accept(stage),
                        OutResponse::Rejected => self.control.reject(),
                    },
                    None => self.control.reject(),
                }
            }
            _ => self.control.reject(),
        }
    }

    async fn handle_control_in(&mut self, req: Request, mut stage: DataInStage) {
        // If we don't have an address yet, respond with max 1 packet.
        // The host doesn't know our EP0 max packet size yet, and might assume
        // a full-length packet is a short packet, thinking we're done sending data.
        // See https://github.com/hathach/tinyusb/issues/184
        const DEVICE_DESCRIPTOR_LEN: u8 = 18;
        if self.pending_address == 0
            && self.config.max_packet_size_0 < DEVICE_DESCRIPTOR_LEN
            && (self.config.max_packet_size_0 as usize) < stage.length
        {
            trace!("received control req while not addressed: capping response to 1 packet.");
            stage.length = self.config.max_packet_size_0 as _;
        }

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
                    self.control.accept_in(&status.to_le_bytes(), stage).await
                }
                Request::GET_DESCRIPTOR => self.handle_get_descriptor(req, stage).await,
                Request::GET_CONFIGURATION => {
                    let status = match self.device_state {
                        UsbDeviceState::Configured => CONFIGURATION_VALUE,
                        _ => CONFIGURATION_NONE,
                    };
                    self.control.accept_in(&status.to_le_bytes(), stage).await
                }
                _ => self.control.reject(),
            },
            (RequestType::Standard, Recipient::Interface) => {
                let iface = match self.interfaces.get_mut(req.index as usize) {
                    Some(iface) => iface,
                    None => return self.control.reject(),
                };

                match req.request {
                    Request::GET_STATUS => {
                        let status: u16 = 0;
                        self.control.accept_in(&status.to_le_bytes(), stage).await
                    }
                    Request::GET_INTERFACE => {
                        self.control
                            .accept_in(&[iface.current_alt_setting], stage)
                            .await;
                    }
                    Request::GET_DESCRIPTOR => match &mut iface.handler {
                        Some(handler) => match handler.get_descriptor(req, self.control_buf) {
                            InResponse::Accepted(data) => self.control.accept_in(data, stage).await,
                            InResponse::Rejected => self.control.reject(),
                        },
                        None => self.control.reject(),
                    },
                    _ => self.control.reject(),
                }
            }
            (RequestType::Standard, Recipient::Endpoint) => match req.request {
                Request::GET_STATUS => {
                    let ep_addr: EndpointAddress = ((req.index as u8) & 0x8f).into();
                    let mut status: u16 = 0x0000;
                    if self.bus.endpoint_is_stalled(ep_addr) {
                        status |= 0x0001;
                    }
                    self.control.accept_in(&status.to_le_bytes(), stage).await
                }
                _ => self.control.reject(),
            },
            (RequestType::Class, Recipient::Interface) => {
                let iface = match self.interfaces.get_mut(req.index as usize) {
                    Some(iface) => iface,
                    None => return self.control.reject(),
                };

                match &mut iface.handler {
                    Some(handler) => match handler.control_in(req, self.control_buf) {
                        InResponse::Accepted(data) => self.control.accept_in(data, stage).await,
                        InResponse::Rejected => self.control.reject(),
                    },
                    None => self.control.reject(),
                }
            }
            _ => self.control.reject(),
        }
    }

    async fn handle_get_descriptor(&mut self, req: Request, stage: DataInStage) {
        let (dtype, index) = req.descriptor_type_index();

        match dtype {
            descriptor_type::BOS => self.control.accept_in(self.bos_descriptor, stage).await,
            descriptor_type::DEVICE => self.control.accept_in(self.device_descriptor, stage).await,
            descriptor_type::CONFIGURATION => {
                self.control.accept_in(self.config_descriptor, stage).await
            }
            descriptor_type::STRING => {
                if index == 0 {
                    self.control
                        .accept_in_writer(req, stage, |w| {
                            w.write(descriptor_type::STRING, &lang_id::ENGLISH_US.to_le_bytes());
                        })
                        .await
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
                                    handler.get_string(index, lang_id, self.control_buf)
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
                        self.control
                            .accept_in_writer(req, stage, |w| w.string(s))
                            .await
                    } else {
                        self.control.reject()
                    }
                }
            }
            _ => self.control.reject(),
        }
    }
}
