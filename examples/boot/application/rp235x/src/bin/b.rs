#![no_std]
#![no_main]

use core::cell::RefCell;

use embassy_boot_rp::{AlignedBuffer, BlockingFirmwareUpdater, FirmwareUpdaterConfig, State};
use embassy_executor::Spawner;
use embassy_rp::flash::{self, Flash};
use embassy_rp::gpio;
use embassy_rp::watchdog::Watchdog;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::Timer;
use gpio::{Level, Output};
use {defmt_rtt as _, panic_reset as _};

const FLASH_SIZE: usize = 2 * 1024 * 1024;

#[embassy_executor::main]
async fn main(_s: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut watchdog = Watchdog::new(p.WATCHDOG);
    watchdog.stop();

    let flash = Flash::<_, _, FLASH_SIZE>::new_blocking(p.FLASH);
    let flash = Mutex::new(RefCell::new(flash));

    let config = FirmwareUpdaterConfig::from_linkerfile_blocking(&flash, &flash);
    let mut aligned = AlignedBuffer([0; flash::WRITE_SIZE]);
    let mut updater = BlockingFirmwareUpdater::new(config, &mut aligned.0);
    let blink = if let Ok(State::Boot) = updater.get_state() {
        // Not the first time booting this binary
        500
    } else {
        // Binary was just updated
        100
    };
    updater.mark_booted().unwrap();

    let mut led = Output::new(p.PIN_25, Level::Low);

    loop {
        led.set_high();
        Timer::after_millis(blink).await;

        led.set_low();
        Timer::after_millis(100).await;
    }
}
