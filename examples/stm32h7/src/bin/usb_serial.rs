#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{panic, *};
use embassy_executor::Spawner;
use embassy_stm32::time::mhz;
use embassy_stm32::usb_otg::Driver;
use embassy_stm32::{bind_interrupts, peripherals, usb_otg, Config};
use embassy_usb::class::cdc_acm::{SerialPort, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use futures::future::join;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    OTG_FS => usb_otg::InterruptHandler<peripherals::USB_OTG_FS>;
});

const TX_BUF: usize = 256;
const RX_BUF: usize = 256;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut config = Config::default();
    config.rcc.sys_ck = Some(mhz(400));
    config.rcc.hclk = Some(mhz(200));
    config.rcc.pll1.q_ck = Some(mhz(100));
    let p = embassy_stm32::init(config);

    // Create the driver, from the HAL.
    let mut ep_out_buffer = [0u8; 256];
    let driver = Driver::new_fs(p.USB_OTG_FS, Irqs, p.PA12, p.PA11, &mut ep_out_buffer);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::<TX_BUF, RX_BUF>::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
    );

    // Create classes on the builder.
    let (mut serial, serial_runner) = SerialPort::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();
    let serial_fut = serial_runner.run();

    // Do stuff with the class!
    let echo_fut = echo(&mut serial);

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(join(usb_fut, serial_fut), echo_fut).await;
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo(serial: &mut SerialPort<'_, TX_BUF, RX_BUF>) -> ! {
    let mut buf = [0; 256];
    loop {
        let n = serial.read(&mut buf).await;
        let data = &buf[..n];
        info!("data: {:x}", data);
        serial.write(data).await;
    }
}
