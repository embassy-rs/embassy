#![no_std]
#![no_main]

use core::cell::RefCell;

#[cfg(feature = "defmt-rtt")]
use defmt_rtt::*;
use embassy_boot_stm32::{AlignedBuffer, BlockingFirmwareState, FirmwareUpdaterConfig};
use embassy_executor::Spawner;
use embassy_stm32::flash::{Flash, WRITE_SIZE};
use embassy_stm32::rcc::WPAN_DEFAULT;
use embassy_stm32::usb::{self, Driver};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::Duration;
use embassy_usb::Builder;
use embassy_usb_dfu::consts::DfuAttributes;
use embassy_usb_dfu::{usb_dfu, Control, ResetImmediate};
use panic_reset as _;

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc = WPAN_DEFAULT;
    let p = embassy_stm32::init(config);
    let flash = Flash::new_blocking(p.FLASH);
    let flash = Mutex::new(RefCell::new(flash));

    let config = FirmwareUpdaterConfig::from_linkerfile_blocking(&flash, &flash);
    let mut magic = AlignedBuffer([0; WRITE_SIZE]);
    let mut firmware_state = BlockingFirmwareState::from_config(config, &mut magic.0);
    firmware_state.mark_booted().expect("Failed to mark booted");

    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-DFU Runtime example");
    config.serial_number = Some("1235678");

    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut state = Control::new(firmware_state, DfuAttributes::CAN_DOWNLOAD);
    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    usb_dfu::<_, _, ResetImmediate>(&mut builder, &mut state, Duration::from_millis(2500));

    let mut dev = builder.build();
    dev.run().await
}
