#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod builder;
pub mod control;
pub mod descriptor;
pub mod driver;
pub mod types;
mod util;

use driver::Unsupported;
use embassy::blocking_mutex::raw::{NoopRawMutex, RawMutex};
use embassy::channel::Channel;
use heapless::Vec;

use self::control::*;
use self::descriptor::*;
use self::driver::{Bus, Driver, Event};
use self::types::*;
use self::util::*;

pub use self::builder::Config;
pub use self::builder::UsbDeviceBuilder;

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

/// The bConfiguration value for the not configured state.
pub const CONFIGURATION_NONE: u8 = 0;

/// The bConfiguration value for the single configuration supported by this device.
pub const CONFIGURATION_VALUE: u8 = 1;

/// The default value for bAlternateSetting for all interfaces.
pub const DEFAULT_ALTERNATE_SETTING: u8 = 0;

pub const MAX_INTERFACE_COUNT: usize = 4;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DeviceCommand {
    Enable,
    Disable,
    RemoteWakeup,
}

/// A handler trait for changes in the device state of the [UsbDevice].
pub trait DeviceStateHandler {
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

    /// Called when the USB device has been disabled.
    fn disabled(&self) {}
}

pub struct UsbDevice<'d, D: Driver<'d>, M: RawMutex = NoopRawMutex> {
    bus: D::Bus,
    handler: Option<&'d dyn DeviceStateHandler>,
    commands: Option<&'d Channel<M, DeviceCommand, 1>>,
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

    interfaces: Vec<(u8, &'d mut dyn ControlHandler), MAX_INTERFACE_COUNT>,
}

impl<'d, D: Driver<'d>, M: RawMutex> UsbDevice<'d, D, M> {
    pub(crate) fn build(
        mut driver: D,
        config: Config<'d>,
        handler: Option<&'d dyn DeviceStateHandler>,
        commands: Option<&'d Channel<M, DeviceCommand, 1>>,
        device_descriptor: &'d [u8],
        config_descriptor: &'d [u8],
        bos_descriptor: &'d [u8],
        interfaces: Vec<(u8, &'d mut dyn ControlHandler), MAX_INTERFACE_COUNT>,
        control_buf: &'d mut [u8],
    ) -> UsbDevice<'d, D, M> {
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
            commands,
            control: ControlPipe::new(control),
            device_descriptor,
            config_descriptor,
            bos_descriptor,
            control_buf,
            device_state: UsbDeviceState::Default,
            suspended: false,
            remote_wakeup_enabled: false,
            self_powered: false,
            pending_address: 0,
            interfaces,
        }
    }

    pub async fn run(&mut self) -> ! {
        if self.config.start_enabled {
            self.bus.enable().await;
        } else {
            self.wait_for_enable().await
        }

        loop {
            let control_fut = self.control.setup();
            let bus_fut = self.bus.poll();
            let commands_fut = recv_or_wait(self.commands);

            match select3(bus_fut, control_fut, commands_fut).await {
                Either3::First(evt) => match evt {
                    Event::Reset => {
                        trace!("usb: reset");
                        self.device_state = UsbDeviceState::Default;
                        self.suspended = false;
                        self.remote_wakeup_enabled = false;
                        self.pending_address = 0;

                        for (_, h) in self.interfaces.iter_mut() {
                            h.reset();
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
                },
                Either3::Second(req) => match req {
                    Setup::DataIn(req, stage) => self.handle_control_in(req, stage).await,
                    Setup::DataOut(req, stage) => self.handle_control_out(req, stage).await,
                },
                Either3::Third(cmd) => match cmd {
                    DeviceCommand::Enable => warn!("usb: Enable command received while enabled."),
                    DeviceCommand::Disable => {
                        trace!("usb: disable");
                        self.bus.disable().await;
                        self.device_state = UsbDeviceState::Disabled;
                        if let Some(h) = &self.handler {
                            h.disabled();
                        }
                        self.wait_for_enable().await;
                    }
                    DeviceCommand::RemoteWakeup => {
                        trace!("usb: remote wakeup");
                        if self.suspended && self.remote_wakeup_enabled {
                            match self.bus.remote_wakeup().await {
                                Ok(()) => {
                                    self.suspended = false;
                                    if let Some(h) = &self.handler {
                                        h.suspended(false);
                                    }
                                }
                                Err(Unsupported) => warn!("Remote wakeup is unsupported!"),
                            }
                        } else {
                            warn!("Remote wakeup requested when not enabled or not suspended.");
                        }
                    }
                },
            }
        }
    }

    async fn wait_for_enable(&mut self) {
        loop {
            // When disabled just wait until we're told to re-enable
            match recv_or_wait(self.commands).await {
                DeviceCommand::Enable => break,
                cmd => warn!("usb: {:?} received while disabled", cmd),
            }
        }

        trace!("usb: enable");
        self.bus.enable().await;
        self.device_state = UsbDeviceState::Default;
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
                    self.bus.set_device_address(self.pending_address);
                    self.device_state = UsbDeviceState::Addressed;
                    if let Some(h) = &self.handler {
                        h.addressed(self.pending_address);
                    }
                    self.control.accept(stage)
                }
                (Request::SET_CONFIGURATION, CONFIGURATION_VALUE_U16) => {
                    self.device_state = UsbDeviceState::Configured;
                    self.bus.set_configured(true);
                    if let Some(h) = &self.handler {
                        h.configured(true);
                    }
                    self.control.accept(stage)
                }
                (Request::SET_CONFIGURATION, CONFIGURATION_NONE_U16) => match self.device_state {
                    UsbDeviceState::Default => self.control.accept(stage),
                    _ => {
                        self.device_state = UsbDeviceState::Addressed;
                        self.bus.set_configured(false);
                        if let Some(h) = &self.handler {
                            h.configured(false);
                        }
                        self.control.accept(stage)
                    }
                },
                _ => self.control.reject(),
            },
            (RequestType::Standard, Recipient::Endpoint) => match (req.request, req.value) {
                (Request::SET_FEATURE, Request::FEATURE_ENDPOINT_HALT) => {
                    let ep_addr = ((req.index as u8) & 0x8f).into();
                    self.bus.set_stalled(ep_addr, true);
                    self.control.accept(stage)
                }
                (Request::CLEAR_FEATURE, Request::FEATURE_ENDPOINT_HALT) => {
                    let ep_addr = ((req.index as u8) & 0x8f).into();
                    self.bus.set_stalled(ep_addr, false);
                    self.control.accept(stage)
                }
                _ => self.control.reject(),
            },
            (_, Recipient::Interface) => {
                let handler = self
                    .interfaces
                    .iter_mut()
                    .find(|(i, _)| req.index == *i as _)
                    .map(|(_, h)| h);

                match handler {
                    Some(handler) => {
                        let response = match (req.request_type, req.request) {
                            (RequestType::Standard, Request::SET_INTERFACE) => {
                                handler.set_interface(req.value)
                            }
                            _ => handler.control_out(req, data),
                        };
                        match response {
                            OutResponse::Accepted => self.control.accept(stage),
                            OutResponse::Rejected => self.control.reject(),
                        }
                    }
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
            (RequestType::Standard, Recipient::Endpoint) => match req.request {
                Request::GET_STATUS => {
                    let ep_addr: EndpointAddress = ((req.index as u8) & 0x8f).into();
                    let mut status: u16 = 0x0000;
                    if self.bus.is_stalled(ep_addr) {
                        status |= 0x0001;
                    }
                    self.control.accept_in(&status.to_le_bytes(), stage).await
                }
                _ => self.control.reject(),
            },
            (_, Recipient::Interface) => {
                let handler = self
                    .interfaces
                    .iter_mut()
                    .find(|(i, _)| req.index == *i as _)
                    .map(|(_, h)| h);

                match handler {
                    Some(handler) => {
                        let response = match (req.request_type, req.request) {
                            (RequestType::Standard, Request::GET_STATUS) => {
                                handler.get_status(self.control_buf)
                            }
                            (RequestType::Standard, Request::GET_INTERFACE) => {
                                handler.get_interface(self.control_buf)
                            }
                            _ => handler.control_in(req, self.control_buf),
                        };

                        match response {
                            InResponse::Accepted(data) => self.control.accept_in(data, stage).await,
                            InResponse::Rejected => self.control.reject(),
                        }
                    }
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
                        1 => self.config.manufacturer,
                        2 => self.config.product,
                        3 => self.config.serial_number,
                        _ => {
                            let _index = StringIndex::new(index);
                            let _lang_id = req.index;
                            // TODO
                            None
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
