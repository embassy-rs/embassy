#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::{info, panic, unwrap};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output, Input, Pull};
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
    let cs = Output::new(p.PIN_17, Level::High);
    let en = Output::new(p.PIN_20, Level::High);
    let hall = Input::new(p.PIN_21, Pull::Down);

    let mut config = Config::default();
    config.polarity = embassy_rp::spi::Polarity::IdleHigh;
    config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    config.frequency = 1_000_000;

    let spi = Spi::new(p.SPI0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, config);

    // Run the USB device.
    unwrap!(spawner.spawn(usb_task(usb)));
    unwrap!(spawner.spawn(spi_task(class, spi, cs, en, hall)));
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
    mut cs: Output<'static>,
    mut en: Output<'static>,
    mut hall: Input<'static>,
) -> ! {
    let mut driver = tmc::tmc5130::TMC5130::new();

    let init_params = tmc::params::InitParams {
        gconf: 0x00000000,
        slaveconf: 0x00000000,
        // IHOLD=3, IRUN=20, IHOLDDELAY=7 (RMS current = 0.75A with VSENSE = 1)
        ihold_irun: 0x00071403,
        tpwmthrs: 0x00000000,
        tcoolthrs: 0x00000010,
        thigh: 0x00000010,
        a1: 0x00000000,
        v1: 0x00000000,
        amax: 0x00002710,
        dmax: 0x00002710,
        vmax: 0x00000000,
        d1: 0x00000001,
        vstop: 0x00000009,
        // VSENSE=1, TOFF=5, CHM=1,
        chopconf: 0x00034295,
        coolconf: 0x1040000,
    };

    driver.init(&mut spi, &mut cs, &mut en, init_params).await.ok();

    driver.set_xactual(&mut spi, &mut cs, 0).await.ok();

    let move_to =  tmc::params::MoveToParams {
        position: 800 * tmc::TMC_MICROSTEPS_PER_STEP as i32,
        speed: tmc::Speed::Rpm(50_f32),
        reset: true,
        stop: false
    };

    while driver
        .move_to(&mut spi, &mut cs, &move_to).await
        .is_err()
    {}

    Timer::after_millis(10).await;

    while let Ok(0) = driver.get_vactual(&mut spi, &mut cs).await {}

    while hall.is_low() {
        Timer::after_millis(10).await;
    }

    driver.stop(&mut spi, &mut cs).await.ok();

    while driver.get_vactual(&mut spi, &mut cs).await.unwrap_or(0) != 0 {
        Timer::after_millis(10).await;
    }

    driver.set_xactual(&mut spi, &mut cs, 0).await.ok();

    let backoff =  tmc::params::MoveToParams {
        position: -90 * tmc::TMC_MICROSTEPS_PER_STEP as i32,
        speed: tmc::Speed::Rpm(50_f32),
        reset: false,
        stop: false
    };

    while driver
        .move_to(&mut spi, &mut cs, &backoff).await
        .is_err()
    {}

    Timer::after_millis(10).await;

    while driver.get_vactual(&mut spi, &mut cs).await.unwrap_or(0) != 0 {
        Timer::after_millis(10).await;
    }

    driver.stop(&mut spi, &mut cs).await.ok();
    driver.set_xactual(&mut spi, &mut cs, 0).await.ok();

    Timer::after_millis(2000).await;

    let mut ticker = Ticker::every(Duration::from_millis(4000));

    loop {
        let params = tmc::params::MoveToParams {
            speed: tmc::Speed::Rpm(50_f32),
            position: -50000,
            reset: false,
            stop: false,
        };

        driver.move_to(&mut spi, &mut cs, &params).await.ok();

        Timer::after_millis(10).await;

        while driver.get_vactual(&mut spi, &mut cs).await.unwrap_or(0) != 0 {
            Timer::after_millis(10).await;
        }

        let params = tmc::params::MoveToParams {
            speed: tmc::Speed::Rpm(50_f32),
            position: 0,
            reset: false,
            stop: false,
        };

        driver.move_to(&mut spi, &mut cs, &params).await.ok();

        Timer::after_millis(10).await;

        while driver.get_vactual(&mut spi, &mut cs).await.unwrap_or(0) != 0 {
            Timer::after_millis(10).await;
        }

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
