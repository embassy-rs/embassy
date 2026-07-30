//! USB1 high-speed enumeration test. Requires a host cable on **P9**.

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, pac, peripherals};
use embassy_time::Timer;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::{Handler, UsbDeviceSpeed};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB1 => InterruptHandler<peripherals::USBHSD>;
});

static CONFIGURED: AtomicBool = AtomicBool::new(false);

struct ConfiguredHandler;

impl Handler for ConfiguredHandler {
    fn configured(&mut self, configured: bool) {
        if configured {
            CONFIGURED.store(true, Ordering::Relaxed);
        }
    }
}

/// Polls [`CONFIGURED`] for up to 10 s. Returns whether the host configured us.
async fn wait_configured() -> bool {
    for _ in 0..100 {
        if CONFIGURED.load(Ordering::Relaxed) {
            return true;
        }
        Timer::after_millis(100).await;
    }
    false
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut hal_config = embassy_nxp::config::Config::default();
    // The high-speed controller requires a system clock of at least 96 MHz.
    hal_config.main_clock = MainClock::FroHf96;
    let p = embassy_nxp::init(hal_config);

    let driver = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-HS enumeration test");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.max_speed = UsbDeviceSpeed::High;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();
    let mut handler = ConfiguredHandler;

    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    let _class = CdcAcmClass::new(&mut builder, &mut state, 512);
    builder.handler(&mut handler);

    let mut usb = builder.build();

    match select(usb.run(), wait_configured()).await {
        Either::First(never) => never,
        Either::Second(true) => {
            // 10b = high speed, per the DEVCMDSTAT.SPEED field docs.
            defmt::assert_eq!(
                pac::USBHSD.devcmdstat().read().speed(),
                0b10,
                "device did not negotiate high speed"
            );
            defmt::assert!(
                pac::USBHSD.devcmdstat().read().dev_addr() != 0,
                "host configured us but DEVCMDSTAT.DEV_ADDR is still 0"
            );
            defmt::info!("Test OK");
            cortex_m::asm::bkpt();
        }
        Either::Second(false) => defmt::panic!("not configured within 10 s"),
    }
}
