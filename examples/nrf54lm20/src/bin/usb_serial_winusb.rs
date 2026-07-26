#![no_std]
#![no_main]

use defmt::{info, panic};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::config::{ClockSpeed, Config as NrfConfig, HfclkSource};
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::usb::{self, Driver};
use embassy_nrf::{Peri, bind_interrupts, peripherals};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::msos::{self, windows_version};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Config, UsbDeviceSpeed};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

const DEVICE_INTERFACE_GUIDS: &[&str] = &["{EAA9A5DC-30BA-44BC-9232-606CDC875321}"];

static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUFFER: StaticCell<[u8; 64]> = StaticCell::new();
static STATE: StaticCell<State> = StaticCell::new();
static EP_OUT_BUFFER: StaticCell<[u8; 2048]> = StaticCell::new();

bind_interrupts!(pub struct Irqs {
    USBHS => usb::InterruptHandler<peripherals::USBHS>;
    VREGUSB => usb::vbus_detect::InterruptHandler;
});

type UsbDriver<'d> = Driver<'d, HardwareVbusDetect>;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init_board();

    let driver = usb_driver(p.USBHS, &mut EP_OUT_BUFFER.init([0; 2048])[..]);

    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.max_speed = UsbDeviceSpeed::High;

    let mut builder = Builder::new(
        driver,
        config,
        &mut CONFIG_DESCRIPTOR.init([0; 256])[..],
        &mut BOS_DESCRIPTOR.init([0; 256])[..],
        &mut MSOS_DESCRIPTOR.init([0; 256])[..],
        &mut CONTROL_BUFFER.init([0; 64])[..],
    );
    builder.msos_descriptor(windows_version::WIN8_1, 2);

    let mut class = CdcAcmClass::new(&mut builder, STATE.init(State::new()), 64);

    let msos_writer = builder.msos_writer();
    msos_writer.configuration(0);
    msos_writer.function(InterfaceNumber(0));
    msos_writer.function_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
    msos_writer.function_feature(msos::RegistryPropertyFeatureDescriptor::new(
        "DeviceInterfaceGUIDs",
        msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
    ));

    let mut usb = builder.build();
    let usb_fut = usb.run();
    let echo_fut = async {
        loop {
            class.wait_connection().await;
            info!("Connected");
            let _ = echo(&mut class).await;
            info!("Disconnected");
        }
    };

    join(usb_fut, echo_fut).await;
}

struct Disconnected;

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected,
        }
    }
}

async fn echo<'d>(class: &mut CdcAcmClass<'d, UsbDriver<'d>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}

fn init_board() -> embassy_nrf::Peripherals {
    let mut config = NrfConfig::default();
    config.hfclk_source = HfclkSource::ExternalXtal;
    config.clock_speed = ClockSpeed::CK64;
    embassy_nrf::init(config)
}

fn usb_driver(usb: Peri<'static, peripherals::USBHS>, ep_out_buffer: &'static mut [u8]) -> UsbDriver<'static> {
    Driver::new(
        usb,
        Irqs,
        HardwareVbusDetect::new(Irqs),
        ep_out_buffer,
        usb::Config::default(),
    )
}
