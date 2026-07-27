//! Tamper detection example.
//!
//! Wire PC13 (the Nucleo user button, B1) as an external tamper input: pressing
//! it grounds the pin, which is detected as a tamper event.
//!
//! Also demonstrates that TAMP backup registers survive a reset: the value
//! written on the previous boot is read back and logged before being
//! incremented, so power-cycling the board (without touching the tamper
//! button) should show the counter go up by one each time.
//!
//! Notes from hardware testing:
//! - By hardware default (`CR2.BKERASE`), *any* tamper detection erases all
//!   backup registers as a security measure — so pressing the tamper button
//!   will also reset the backup counter back to 0 on top of logging the
//!   event. This driver doesn't expose control over that bit in v1.
//! - Resetting the board *while a debugger stays attached* (e.g.
//!   `probe-rs reset`) was observed to also clear the backup registers on
//!   this chip, even though `RCC.BDCR` (LSEON/RTCSEL) itself was not reset —
//!   consistent with STM32WBA's documented behavior of treating debug access
//!   as a security-relevant event, separate from this driver's own logic.
//!   Power-cycling the board with the debugger disconnected is the reliable
//!   way to observe plain reset persistence.
#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::gpio::Pull;
use embassy_stm32::tamp::{self, ExternalTamperConfig, Filter, Tamp, Trigger};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TAMP => tamp::InterruptHandler;
});

// 0-based external channel index. PC13 is wired to TAMP4 (IN4) on this chip.
const BUTTON_CHANNEL: u8 = 3;
const BACKUP_REGISTER: usize = 0;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut tamp = Tamp::new(p.TAMP, Irqs);

    let boot_count = tamp.read_backup_register(BACKUP_REGISTER).unwrap_or(0);
    info!("boot count (from backup register, survives reset): {}", boot_count);
    tamp.write_backup_register(BACKUP_REGISTER, boot_count + 1);

    let _button = tamp.configure_external_channel(
        BUTTON_CHANNEL,
        p.PC13,
        ExternalTamperConfig {
            trigger: Trigger::ActiveLow,
            filter: Filter::Filter8,
            pull: Pull::Up,
        },
    );

    info!("Press the USER button to trigger a tamper event...");

    loop {
        let status = tamp.wait_for_tamper().await;
        info!(
            "Tamper detected! external channel {}: {}, monotonic counter: {}",
            BUTTON_CHANNEL,
            status.is_external(BUTTON_CHANNEL),
            tamp.monotonic_counter()
        );
        tamp.clear_tamper_flags(status);
    }
}
