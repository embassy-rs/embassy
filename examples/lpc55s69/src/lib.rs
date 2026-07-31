#![no_std]
use defmt::panic;
use embassy_usb::UsbDeviceSpeed;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::{Driver, EndpointError};

pub struct UsbResources<'d> {
    pub(crate) config_descriptor: [u8; 256],
    pub(crate) bos_descriptor: [u8; 256],
    pub(crate) control_buf: [u8; 64],
    pub(crate) state: State<'d>,
}

impl<'d> UsbResources<'d> {
    pub const fn new() -> Self {
        Self {
            config_descriptor: [0; 256],
            bos_descriptor: [0; 256],
            control_buf: [0; 64],
            state: State::new(),
        }
    }
}

pub struct UsbParams {
    pub pid: u16,
    pub product: &'static str,
    pub serial: &'static str,
    pub max_speed: UsbDeviceSpeed,
}

pub(crate) fn cdc<'d, D: Driver<'d>>(
    driver: D,
    resources: &'d mut UsbResources<'d>,
    params: UsbParams,
    mps: u16,
) -> (embassy_usb::UsbDevice<'d, D>, CdcAcmClass<'d, D>) {
    let mut config = embassy_usb::Config::new(0xc0de, params.pid);
    config.manufacturer = Some("Embassy");
    config.product = Some(params.product);
    config.serial_number = Some(params.serial);
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.max_speed = params.max_speed;
    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        &mut resources.config_descriptor,
        &mut resources.bos_descriptor,
        &mut [],
        &mut resources.control_buf,
    );
    let class = CdcAcmClass::new(&mut builder, &mut resources.state, mps);
    (builder.build(), class)
}

pub(crate) struct Disconnected;

impl From<EndpointError> for Disconnected {
    fn from(error: EndpointError) -> Self {
        match error {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected,
        }
    }
}

pub mod serial;
pub mod throughput;
pub mod throughput_device;
