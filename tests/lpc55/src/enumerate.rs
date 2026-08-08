use core::sync::atomic::{AtomicBool, Ordering};

use embassy_futures::select::{Either, select};
use embassy_time::Timer;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::Driver;
use embassy_usb::{Handler, UsbDeviceSpeed};

pub struct Params {
    pub product: &'static str,
    pub max_speed: UsbDeviceSpeed,
}

pub struct Resources<'d> {
    config_descriptor: [u8; 256],
    bos_descriptor: [u8; 256],
    control_buf: [u8; 64],
    state: State<'d>,
    handler: ConfiguredHandler,
}

impl<'d> Resources<'d> {
    pub const fn new(configured: &'static AtomicBool) -> Self {
        Self {
            config_descriptor: [0; 256],
            bos_descriptor: [0; 256],
            control_buf: [0; 64],
            state: State::new(),
            handler: ConfiguredHandler(configured),
        }
    }
}

struct ConfiguredHandler(&'static AtomicBool);

impl Handler for ConfiguredHandler {
    fn configured(&mut self, configured: bool) {
        if configured {
            self.0.store(true, Ordering::Relaxed);
        }
    }
}

pub async fn run<'d, D: Driver<'d>, F: FnOnce(), const MPS: usize>(
    driver: D,
    resources: &'d mut Resources<'d>,
    params: Params,
    validate: F,
) {
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some(params.product);
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.max_speed = params.max_speed;

    let configured = resources.handler.0;
    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        &mut resources.config_descriptor,
        &mut resources.bos_descriptor,
        &mut [],
        &mut resources.control_buf,
    );
    let _class = CdcAcmClass::new(&mut builder, &mut resources.state, MPS as u16);
    builder.handler(&mut resources.handler);
    let mut usb = builder.build();

    match select(usb.run(), wait_configured(configured)).await {
        Either::First(never) => never,
        Either::Second(true) => {
            validate();
            defmt::info!("Test OK");
            cortex_m::asm::bkpt();
        }
        Either::Second(false) => defmt::panic!("not configured within 10 s"),
    }
}

async fn wait_configured(configured: &AtomicBool) -> bool {
    for _ in 0..100 {
        if configured.load(Ordering::Relaxed) {
            return true;
        }
        Timer::after_millis(100).await;
    }
    false
}
