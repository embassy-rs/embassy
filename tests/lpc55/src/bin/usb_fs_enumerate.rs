//! USB0 full-speed enumeration test. Requires a host cable on **P10**.
//!
//! Uses the default HAL config on purpose: the full-speed driver brings up its
//! own 48 MHz clock, and proving that is part of the test.

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, pac, peripherals};
use embassy_time::Timer;
use embassy_usb::Handler;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
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
    let p = embassy_nxp::init(embassy_nxp::config::Config::default());

    let driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, Memory::usb1_sram());

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-FS enumeration test");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

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

    let _class = CdcAcmClass::new(&mut builder, &mut state, 64);
    builder.handler(&mut handler);

    let mut usb = builder.build();

    match select(usb.run(), wait_configured()).await {
        Either::First(never) => never,
        Either::Second(true) => {
            defmt::assert!(
                pac::USB0.devcmdstat().read().dev_addr() != 0,
                "host configured us but DEVCMDSTAT.DEV_ADDR is still 0"
            );
            defmt::info!("Test OK");
            cortex_m::asm::bkpt();
        }
        Either::Second(false) => defmt::panic!("not configured within 10 s"),
    }
}
