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

use heapless::Vec;

use self::control::*;
use self::descriptor::*;
use self::driver::*;
use self::types::*;
use self::util::*;

pub use self::builder::Config;
pub use self::builder::UsbDeviceBuilder;

/// The global state of the USB device.
///
/// In general class traffic is only possible in the `Configured` state.
#[repr(u8)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum UsbDeviceState {
    /// The USB device has just been created or reset.
    Default,

    /// The USB device has received an address from the host.
    Addressed,

    /// The USB device has been configured and is fully functional.
    Configured,

    /// The USB device has been suspended by the host or it has been unplugged from the USB bus.
    Suspend,
}

/// The bConfiguration value for the not configured state.
pub const CONFIGURATION_NONE: u8 = 0;

/// The bConfiguration value for the single configuration supported by this device.
pub const CONFIGURATION_VALUE: u8 = 1;

/// The default value for bAlternateSetting for all interfaces.
pub const DEFAULT_ALTERNATE_SETTING: u8 = 0;

pub const MAX_INTERFACE_COUNT: usize = 4;

pub struct UsbDevice<'d, D: Driver<'d>> {
    bus: D::Bus,
    control: ControlPipe<D::ControlPipe>,

    config: Config<'d>,
    device_descriptor: &'d [u8],
    config_descriptor: &'d [u8],
    bos_descriptor: &'d [u8],
    control_buf: &'d mut [u8],

    device_state: UsbDeviceState,
    remote_wakeup_enabled: bool,
    self_powered: bool,
    pending_address: u8,

    interfaces: Vec<(u8, &'d mut dyn ControlHandler), MAX_INTERFACE_COUNT>,
}

impl<'d, D: Driver<'d>> UsbDevice<'d, D> {
    pub(crate) fn build(
        mut driver: D,
        config: Config<'d>,
        device_descriptor: &'d [u8],
        config_descriptor: &'d [u8],
        bos_descriptor: &'d [u8],
        interfaces: Vec<(u8, &'d mut dyn ControlHandler), MAX_INTERFACE_COUNT>,
        control_buf: &'d mut [u8],
    ) -> Self {
        let control = driver
            .alloc_control_pipe(config.max_packet_size_0 as u16)
            .expect("failed to alloc control endpoint");

        // Enable the USB bus.
        // This prevent further allocation by consuming the driver.
        let driver = driver.enable();

        Self {
            bus: driver,
            config,
            control: ControlPipe {
                control,
                request: None,
            },
            device_descriptor,
            config_descriptor,
            bos_descriptor,
            control_buf,
            device_state: UsbDeviceState::Default,
            remote_wakeup_enabled: false,
            self_powered: false,
            pending_address: 0,
            interfaces,
        }
    }

    pub async fn run(&mut self) {
        loop {
            let control_fut = self.control.setup();
            let bus_fut = self.bus.poll();
            match select(bus_fut, control_fut).await {
                Either::Left(evt) => match evt {
                    Event::Reset => {
                        self.bus.reset();

                        self.device_state = UsbDeviceState::Default;
                        self.remote_wakeup_enabled = false;
                        self.pending_address = 0;

                        for (_, h) in self.interfaces.iter_mut() {
                            h.reset();
                        }
                    }
                    Event::Resume => {}
                    Event::Suspend => {
                        self.bus.suspend();
                        self.device_state = UsbDeviceState::Suspend;
                    }
                },
                Either::Right(req) => {
                    debug!("control request: {:x}", req);

                    match req.direction {
                        UsbDirection::In => self.handle_control_in(req).await,
                        UsbDirection::Out => self.handle_control_out(req).await,
                    }
                }
            }
        }
    }

    async fn handle_control_out(&mut self, req: Request) {
        const CONFIGURATION_NONE_U16: u16 = CONFIGURATION_NONE as u16;
        const CONFIGURATION_VALUE_U16: u16 = CONFIGURATION_VALUE as u16;

        // If the request has a data state, we must read it.
        let data = if req.length > 0 {
            match self.control.data_out(self.control_buf).await {
                Ok(data) => data,
                Err(_) => {
                    warn!("usb: failed to read CONTROL OUT data stage.");
                    return;
                }
            }
        } else {
            &[]
        };

        match (req.request_type, req.recipient) {
            (RequestType::Standard, Recipient::Device) => match (req.request, req.value) {
                (Request::CLEAR_FEATURE, Request::FEATURE_DEVICE_REMOTE_WAKEUP) => {
                    self.remote_wakeup_enabled = false;
                    self.control.accept();
                }
                (Request::SET_FEATURE, Request::FEATURE_DEVICE_REMOTE_WAKEUP) => {
                    self.remote_wakeup_enabled = true;
                    self.control.accept();
                }
                (Request::SET_ADDRESS, 1..=127) => {
                    self.pending_address = req.value as u8;
                    self.control.accept();
                }
                (Request::SET_CONFIGURATION, CONFIGURATION_VALUE_U16) => {
                    self.device_state = UsbDeviceState::Configured;
                    self.control.accept();
                }
                (Request::SET_CONFIGURATION, CONFIGURATION_NONE_U16) => match self.device_state {
                    UsbDeviceState::Default => {
                        self.control.accept();
                    }
                    _ => {
                        self.device_state = UsbDeviceState::Addressed;
                        self.control.accept();
                    }
                },
                _ => self.control.reject(),
            },
            (RequestType::Standard, Recipient::Endpoint) => match (req.request, req.value) {
                (Request::SET_FEATURE, Request::FEATURE_ENDPOINT_HALT) => {
                    let ep_addr = ((req.index as u8) & 0x8f).into();
                    self.bus.set_stalled(ep_addr, true);
                    self.control.accept();
                }
                (Request::CLEAR_FEATURE, Request::FEATURE_ENDPOINT_HALT) => {
                    let ep_addr = ((req.index as u8) & 0x8f).into();
                    self.bus.set_stalled(ep_addr, false);
                    self.control.accept();
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
                            OutResponse::Accepted => self.control.accept(),
                            OutResponse::Rejected => self.control.reject(),
                        }
                    }
                    None => self.control.reject(),
                }
            }
            _ => self.control.reject(),
        }
    }

    async fn handle_control_in(&mut self, req: Request) {
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
                    self.control.accept_in(&status.to_le_bytes()).await;
                }
                Request::GET_DESCRIPTOR => {
                    self.handle_get_descriptor(req).await;
                }
                Request::GET_CONFIGURATION => {
                    let status = match self.device_state {
                        UsbDeviceState::Configured => CONFIGURATION_VALUE,
                        _ => CONFIGURATION_NONE,
                    };
                    self.control.accept_in(&status.to_le_bytes()).await;
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
                    self.control.accept_in(&status.to_le_bytes()).await;
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
                            InResponse::Accepted(data) => self.control.accept_in(data).await,
                            InResponse::Rejected => self.control.reject(),
                        }
                    }
                    None => self.control.reject(),
                }
            }
            _ => self.control.reject(),
        }
    }

    async fn handle_get_descriptor(&mut self, req: Request) {
        let (dtype, index) = req.descriptor_type_index();

        match dtype {
            descriptor_type::BOS => self.control.accept_in(self.bos_descriptor).await,
            descriptor_type::DEVICE => self.control.accept_in(self.device_descriptor).await,
            descriptor_type::CONFIGURATION => self.control.accept_in(self.config_descriptor).await,
            descriptor_type::STRING => {
                if index == 0 {
                    self.control
                        .accept_in_writer(req, |w| {
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
                        self.control.accept_in_writer(req, |w| w.string(s)).await;
                    } else {
                        self.control.reject()
                    }
                }
            }
            _ => self.control.reject(),
        }
    }
}

struct ControlPipe<C: driver::ControlPipe> {
    control: C,
    request: Option<Request>,
}

impl<C: driver::ControlPipe> ControlPipe<C> {
    async fn setup(&mut self) -> Request {
        assert!(self.request.is_none());
        let req = self.control.setup().await;
        self.request = Some(req);
        req
    }

    async fn data_out<'a>(&mut self, buf: &'a mut [u8]) -> Result<&'a [u8], ReadError> {
        let req = self.request.unwrap();
        assert_eq!(req.direction, UsbDirection::Out);
        assert!(req.length > 0);
        let req_length = usize::from(req.length);

        let max_packet_size = self.control.max_packet_size();
        let mut total = 0;

        for chunk in buf.chunks_mut(max_packet_size) {
            let size = self.control.data_out(chunk).await?;
            total += size;
            if size < max_packet_size || total == req_length {
                break;
            }
        }

        Ok(&buf[0..total])
    }

    async fn accept_in(&mut self, buf: &[u8]) -> () {
        #[cfg(feature = "defmt")]
        debug!("control in accept {:x}", buf);
        #[cfg(not(feature = "defmt"))]
        debug!("control in accept {:x?}", buf);
        let req = unwrap!(self.request);
        assert!(req.direction == UsbDirection::In);

        let req_len = usize::from(req.length);
        let len = buf.len().min(req_len);
        let max_packet_size = self.control.max_packet_size();
        let need_zlp = len != req_len && (len % usize::from(max_packet_size)) == 0;

        let mut chunks = buf[0..len]
            .chunks(max_packet_size)
            .chain(need_zlp.then(|| -> &[u8] { &[] }));

        while let Some(chunk) = chunks.next() {
            self.control.data_in(chunk, chunks.size_hint().0 == 0).await;
        }

        self.request = None;
    }

    async fn accept_in_writer(&mut self, req: Request, f: impl FnOnce(&mut DescriptorWriter)) {
        let mut buf = [0; 256];
        let mut w = DescriptorWriter::new(&mut buf);
        f(&mut w);
        let pos = w.position().min(usize::from(req.length));
        self.accept_in(&buf[..pos]).await;
    }

    fn accept(&mut self) {
        assert!(self.request.is_some());
        self.control.accept();
        self.request = None;
    }

    fn reject(&mut self) {
        assert!(self.request.is_some());
        self.control.reject();
        self.request = None;
    }
}
