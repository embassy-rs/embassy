#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::{info, panic, unwrap};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{SPI0, USB};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_time::{Duration, Instant, Ticker, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::UsbDevice;
use heapless::String;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[path = "../tmc/mod.rs"]
mod tmc;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello there!");

    let p = embassy_rp::init(Default::default());

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, Irqs);

    // Create embassy-usb Config
    let config = {
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Embassy");
        config.product = Some("USB-serial example");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;

        // Required for windows compatibility.
        config.device_class = 0xEF;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config.composite_with_iads = true;
        config
    };

    // Create embassy-usb DeviceBuilder using the driver and config.
    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        let builder = embassy_usb::Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [], // no msos descriptors
            CONTROL_BUF.init([0; 64]),
        );
        builder
    };

    // Create classes on the builder.
    let class = {
        static STATE: StaticCell<State> = StaticCell::new();
        let state = STATE.init(State::new());
        CdcAcmClass::new(&mut builder, state, 64)
    };

    // Build the builder.
    let usb = builder.build();

    let miso = p.PIN_16;
    let mosi = p.PIN_19;
    let clk = p.PIN_18;
    let adc_cs = Output::new(p.PIN_17, Level::High);
    let dac_cs = Output::new(p.PIN_20, Level::High);

    let mut config = Config::default();
    config.polarity = embassy_rp::spi::Polarity::IdleHigh;
    config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    config.frequency = 100_000;

    let spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, config);

    // Run the USB device.
    unwrap!(spawner.spawn(usb_task(usb)));
    unwrap!(spawner.spawn(spi_task(class, spi, adc_cs, dac_cs)));
}

type MyUsbDriver = Driver<'static, USB>;
type MyUsbDevice = UsbDevice<'static, MyUsbDriver>;

#[embassy_executor::task]
async fn usb_task(mut usb: MyUsbDevice) -> ! {
    usb.run().await
}

#[embassy_executor::task]
async fn spi_task(
    mut class: CdcAcmClass<'static, Driver<'static, USB>>,
    mut spi: Spi<'static, SPI0, Async>,
    mut adc_cs: Output<'static>,
    mut dac_cs: Output<'static>,
) -> ! {
    let mut ticker = Ticker::every(Duration::from_millis(500));

    loop {
        let now = Instant::now().as_micros();

        let delta = Instant::now().as_micros() - now;

        // Buffer to hold the formatted output
        let mut buf = String::<64>::new();

        // Format the data and voltage as a string
        let _ = write!(buf, "us: {}\r\n", delta);

        // Send the data over USB
        class.write_packet(buf.as_bytes()).await.ok();

        ticker.next().await;
    }
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
