#![no_std]
#![feature(generic_associated_types)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod builder;
mod control;
pub mod descriptor;
pub mod driver;
pub mod types;
mod util;

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

pub struct UsbDevice<'d, D: Driver<'d>> {
    bus: D::Bus,
    control_in: D::EndpointIn,
    control_out: D::EndpointOut,

    config: Config<'d>,
    device_descriptor: &'d [u8],
    config_descriptor: &'d [u8],
    bos_descriptor: &'d [u8],

    device_state: UsbDeviceState,
    remote_wakeup_enabled: bool,
    self_powered: bool,
    pending_address: u8,
}

impl<'d, D: Driver<'d>> UsbDevice<'d, D> {
    pub(crate) fn build(
        mut driver: D,
        config: Config<'d>,
        device_descriptor: &'d [u8],
        config_descriptor: &'d [u8],
        bos_descriptor: &'d [u8],
    ) -> Self {
        let control_out = driver
            .alloc_endpoint_out(
                Some(0x00.into()),
                EndpointType::Control,
                config.max_packet_size_0 as u16,
                0,
            )
            .expect("failed to alloc control endpoint");

        let control_in = driver
            .alloc_endpoint_in(
                Some(0x80.into()),
                EndpointType::Control,
                config.max_packet_size_0 as u16,
                0,
            )
            .expect("failed to alloc control endpoint");

        // Enable the USB bus.
        // This prevent further allocation by consuming the driver.
        let driver = driver.enable();

        Self {
            bus: driver,
            config,
            control_in,
            control_out,
            device_descriptor,
            config_descriptor,
            bos_descriptor,
            device_state: UsbDeviceState::Default,
            remote_wakeup_enabled: false,
            self_powered: false,
            pending_address: 0,
        }
    }

    pub async fn run(&mut self) {
        let mut buf = [0; 8];

        loop {
            let control_fut = self.control_out.read(&mut buf);
            let bus_fut = self.bus.poll();
            match select(bus_fut, control_fut).await {
                Either::Left(evt) => match evt {
                    Event::Reset => {
                        self.bus.reset();

                        self.device_state = UsbDeviceState::Default;
                        self.remote_wakeup_enabled = false;
                        self.pending_address = 0;

                        // TODO
                        //self.control.reset();
                        //for cls in classes {
                        //    cls.reset();
                        //}
                    }
                    Event::Resume => {}
                    Event::Suspend => {
                        self.bus.suspend();
                        self.device_state = UsbDeviceState::Suspend;
                    }
                },
                Either::Right(n) => {
                    let n = n.unwrap();
                    assert_eq!(n, 8);
                    let req = Request::parse(&buf).unwrap();
                    info!("control request: {:x}", req);

                    // Now that we have properly parsed the setup packet, ensure the end-point is no longer in
                    // a stalled state.
                    self.control_out.set_stalled(false);

                    match req.direction {
                        UsbDirection::In => self.handle_control_in(req).await,
                        UsbDirection::Out => self.handle_control_out(req).await,
                    }
                }
            }
        }
    }

    async fn write_chunked(&mut self, data: &[u8]) -> Result<(), driver::WriteError> {
        for c in data.chunks(8) {
            self.control_in.write(c).await?;
        }
        if data.len() % 8 == 0 {
            self.control_in.write(&[]).await?;
        }
        Ok(())
    }

    async fn control_out_accept(&mut self, req: Request) {
        info!("control out accept");
        // status phase
        // todo: cleanup
        self.control_out.read(&mut []).await.unwrap();
    }

    async fn control_in_accept(&mut self, req: Request, data: &[u8]) {
        info!("control accept {:x}", data);

        let len = data.len().min(req.length as _);
        if let Err(e) = self.write_chunked(&data[..len]).await {
            info!("write_chunked failed: {:?}", e);
        }

        // status phase
        // todo: cleanup
        self.control_out.read(&mut []).await.unwrap();
    }

    async fn control_in_accept_writer(
        &mut self,
        req: Request,
        f: impl FnOnce(&mut DescriptorWriter),
    ) {
        let mut buf = [0; 256];
        let mut w = DescriptorWriter::new(&mut buf);
        f(&mut w);
        let pos = w.position();
        self.control_in_accept(req, &buf[..pos]).await;
    }

    fn control_reject(&mut self, req: Request) {
        self.control_out.set_stalled(true);
    }

    async fn handle_control_out(&mut self, req: Request) {
        // TODO actually read the data if there's an OUT data phase.

        const CONFIGURATION_NONE_U16: u16 = CONFIGURATION_NONE as u16;
        const CONFIGURATION_VALUE_U16: u16 = CONFIGURATION_VALUE as u16;
        const DEFAULT_ALTERNATE_SETTING_U16: u16 = DEFAULT_ALTERNATE_SETTING as u16;

        match req.request_type {
            RequestType::Standard => match (req.recipient, req.request, req.value) {
                (
                    Recipient::Device,
                    Request::CLEAR_FEATURE,
                    Request::FEATURE_DEVICE_REMOTE_WAKEUP,
                ) => {
                    self.remote_wakeup_enabled = false;
                    self.control_out_accept(req).await;
                }

                (Recipient::Endpoint, Request::CLEAR_FEATURE, Request::FEATURE_ENDPOINT_HALT) => {
                    //self.bus.set_stalled(((req.index as u8) & 0x8f).into(), false);
                    self.control_out_accept(req).await;
                }

                (
                    Recipient::Device,
                    Request::SET_FEATURE,
                    Request::FEATURE_DEVICE_REMOTE_WAKEUP,
                ) => {
                    self.remote_wakeup_enabled = true;
                    self.control_out_accept(req).await;
                }

                (Recipient::Endpoint, Request::SET_FEATURE, Request::FEATURE_ENDPOINT_HALT) => {
                    self.bus
                        .set_stalled(((req.index as u8) & 0x8f).into(), true);
                    self.control_out_accept(req).await;
                }

                (Recipient::Device, Request::SET_ADDRESS, 1..=127) => {
                    self.pending_address = req.value as u8;

                    // on NRF the hardware auto-handles SET_ADDRESS.
                    self.control_out_accept(req).await;
                }

                (Recipient::Device, Request::SET_CONFIGURATION, CONFIGURATION_VALUE_U16) => {
                    self.device_state = UsbDeviceState::Configured;
                    self.control_out_accept(req).await;
                }

                (Recipient::Device, Request::SET_CONFIGURATION, CONFIGURATION_NONE_U16) => {
                    match self.device_state {
                        UsbDeviceState::Default => {
                            self.control_out_accept(req).await;
                        }
                        _ => {
                            self.device_state = UsbDeviceState::Addressed;
                            self.control_out_accept(req).await;
                        }
                    }
                }

                (Recipient::Interface, Request::SET_INTERFACE, DEFAULT_ALTERNATE_SETTING_U16) => {
                    // TODO: do something when alternate settings are implemented
                    self.control_out_accept(req).await;
                }

                _ => self.control_reject(req),
            },
            _ => self.control_reject(req),
        }
    }

    async fn handle_control_in(&mut self, req: Request) {
        match req.request_type {
            RequestType::Standard => match (req.recipient, req.request) {
                (Recipient::Device, Request::GET_STATUS) => {
                    let mut status: u16 = 0x0000;
                    if self.self_powered {
                        status |= 0x0001;
                    }
                    if self.remote_wakeup_enabled {
                        status |= 0x0002;
                    }
                    self.control_in_accept(req, &status.to_le_bytes()).await;
                }

                (Recipient::Interface, Request::GET_STATUS) => {
                    let status: u16 = 0x0000;
                    self.control_in_accept(req, &status.to_le_bytes()).await;
                }

                (Recipient::Endpoint, Request::GET_STATUS) => {
                    let ep_addr: EndpointAddress = ((req.index as u8) & 0x8f).into();
                    let mut status: u16 = 0x0000;
                    if self.bus.is_stalled(ep_addr) {
                        status |= 0x0001;
                    }
                    self.control_in_accept(req, &status.to_le_bytes()).await;
                }

                (Recipient::Device, Request::GET_DESCRIPTOR) => {
                    self.handle_get_descriptor(req).await;
                }

                (Recipient::Device, Request::GET_CONFIGURATION) => {
                    let status = match self.device_state {
                        UsbDeviceState::Configured => CONFIGURATION_VALUE,
                        _ => CONFIGURATION_NONE,
                    };
                    self.control_in_accept(req, &status.to_le_bytes()).await;
                }

                (Recipient::Interface, Request::GET_INTERFACE) => {
                    // TODO: change when alternate settings are implemented
                    let status = DEFAULT_ALTERNATE_SETTING;
                    self.control_in_accept(req, &status.to_le_bytes()).await;
                }
                _ => self.control_reject(req),
            },
            _ => self.control_reject(req),
        }
    }

    async fn handle_get_descriptor(&mut self, req: Request) {
        let (dtype, index) = req.descriptor_type_index();
        let config = self.config.clone();

        match dtype {
            descriptor_type::BOS => self.control_in_accept(req, self.bos_descriptor).await,
            descriptor_type::DEVICE => self.control_in_accept(req, self.device_descriptor).await,
            descriptor_type::CONFIGURATION => {
                self.control_in_accept(req, self.config_descriptor).await
            }
            descriptor_type::STRING => {
                if index == 0 {
                    self.control_in_accept_writer(req, |w| {
                        w.write(descriptor_type::STRING, &lang_id::ENGLISH_US.to_le_bytes())
                            .unwrap();
                    })
                    .await
                } else {
                    let s = match index {
                        1 => self.config.manufacturer,
                        2 => self.config.product,
                        3 => self.config.serial_number,
                        _ => {
                            let index = StringIndex::new(index);
                            let lang_id = req.index;
                            None
                            //classes
                            //    .iter()
                            //    .filter_map(|cls| cls.get_string(index, lang_id))
                            //    .nth(0)
                        }
                    };

                    if let Some(s) = s {
                        self.control_in_accept_writer(req, |w| w.string(s).unwrap())
                            .await;
                    } else {
                        self.control_reject(req)
                    }
                }
            }
            _ => self.control_reject(req),
        }
    }
}
