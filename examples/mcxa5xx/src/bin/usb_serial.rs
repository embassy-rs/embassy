//! USB CDC-ACM serial echo example for the FRDM-MCXA577 (MCX-A577).
//!
//! Enumerates the board as a full-speed USB serial device and echoes every
//! packet received from the host. Connect the board's USB device port to a host.
//!
//! Build/run:
//! ```text
//! cargo run --release --bin usb_serial
//! ```

#![no_std]
#![no_main]

use defmt::{info, panic};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_mcxa::clocks::config::{SoscConfig, SoscMode};
use embassy_mcxa::clocks::{PoweredClock, VddLevel};
use embassy_mcxa::usb::{Config as UsbDriverConfig, Driver, InterruptHandler};
use embassy_mcxa::{bind_interrupts, peripherals};
use embassy_usb::Builder;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB1_HS => InterruptHandler<peripherals::USB1>;
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
    info!("=== USB CDC-ACM serial example ===");

    let driver = Driver::new(p.USB1, Irqs, UsbDriverConfig::default());

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafd);
    config.manufacturer = Some("Embassy");
    config.product = Some("MCXA577 USB serial");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);
    let mut usb = builder.build();
    let usb_fut = usb.run();

    let echo_fut = async {
        loop {
            class.wait_connection().await;
            info!("serial connected");
            let _ = echo(&mut class).await;
            info!("serial disconnected");
        }
    };

    join(usb_fut, echo_fut).await;
}

struct Disconnected;

impl From<EndpointError> for Disconnected {
    fn from(value: EndpointError) -> Self {
        match value {
            EndpointError::BufferOverflow => panic!("USB serial buffer overflow"),
            EndpointError::Disabled => Disconnected,
        }
    }
}

async fn echo<'d>(class: &mut CdcAcmClass<'d, Driver<'d>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("serial rx: {:x}", data);
        class.write_packet(data).await?;
    }
}
