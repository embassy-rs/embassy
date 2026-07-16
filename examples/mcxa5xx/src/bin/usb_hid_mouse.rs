//! USB HID mouse example for the FRDM-MCXA577 (MCX-A577).
//!
//! Enumerates the board as a USB full-speed HID mouse and slowly moves the
//! pointer back and forth. Connect the board's USB device port to a host.
//!
//! Build/run:
//! ```text
//! cargo run --release --bin usb_hid_mouse
//! ```

#![no_std]
#![no_main]

use defmt::{info, panic, warn};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::clocks::config::{SoscConfig, SoscMode};
use embassy_mcxa::clocks::{PoweredClock, VddLevel};
use embassy_mcxa::usb::{Config as UsbDriverConfig, Driver, InterruptHandler, PhyConfig};
use embassy_time::Timer;
use embassy_usb::class::hid::{HidBootProtocol, HidSubclass, HidWriter, ReportId, RequestHandler, State};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Config, Handler};
use usbd_hid::descriptor::{MouseReport, SerializedDescriptor};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB1_HS => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut hal_config = hal::config::Config::default();
    // Match the SDK/Zephyr board profile; the USBHS PHY PLL needs this voltage
    // mode to lock reliably on FRDM-MCXA577.
    hal_config.clock_cfg.vdd_power.active_mode.level = VddLevel::OverDriveMode;
    hal_config.clock_cfg.vdd_power.low_power_mode.level = VddLevel::OverDriveMode;
    hal_config.clock_cfg.sosc = Some(SoscConfig {
        mode: SoscMode::CrystalOscillator,
        frequency: 24_000_000,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
    });
    let p = hal::init(hal_config);
    info!("=== USB HID mouse example ===");

    // Create the USB device driver (full speed).
    info!("Creating USB driver");
    let mut driver_config = UsbDriverConfig::default();
    driver_config.phy = PhyConfig::frdm_mcxa577();
    let driver = match Driver::new(p.USB1, Irqs, driver_config) {
        Ok(driver) => driver,
        Err(e) => panic!("USB init failed: {:?}", e),
    };
    info!("USB driver created");

    // Configure the USB device descriptors.
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("MCXA577 HID mouse");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Buffers for the USB device stack.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut request_handler = MyRequestHandler {};
    let mut device_handler = MyDeviceHandler::new();

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );
    builder.handler(&mut device_handler);

    // Create the HID mouse class.
    let config = embassy_usb::class::hid::Config {
        report_descriptor: MouseReport::desc(),
        request_handler: Some(&mut request_handler),
        poll_ms: 60,
        max_packet_size: 8,
        hid_subclass: HidSubclass::No,
        hid_boot_protocol: HidBootProtocol::None,
    };
    let mut writer = HidWriter::<_, 8>::new(&mut builder, &mut state, config);

    // Build the device.
    let mut usb = builder.build();
    let usb_fut = usb.run();

    // Move the mouse pointer back and forth.
    let hid_fut = async {
        let mut dir: i8 = 5;
        loop {
            Timer::after_millis(200).await;

            let report = MouseReport {
                buttons: 0,
                x: dir,
                y: 0,
                wheel: 0,
                pan: 0,
            };
            match writer.write_serialize(&report).await {
                Ok(()) => {}
                Err(e) => warn!("Failed to send report: {:?}", e),
            }
            dir = -dir;
        }
    };

    join(usb_fut, hid_fut).await;
}

struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }
}

struct MyDeviceHandler {}

impl MyDeviceHandler {
    fn new() -> Self {
        MyDeviceHandler {}
    }
}

impl Handler for MyDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        if enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&mut self) {
        info!("Bus reset");
    }

    fn addressed(&mut self, addr: u8) {
        info!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        if configured {
            info!("Device configured, it may now draw up to the configured current from Vbus.")
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }
}
