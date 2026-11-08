//! This example tests the RP Pico on board LED, but is run from SRAM.
//!
//! It does not blink on the RP Pico W board. See `blinky_wifi.rs`.
//!
//! Same code as `blinky.rs`, plus a check that it is really running from SRAM.
//!
//! Compile, post-process and run with `run-blinky-ram.sh`.
//!
//! The example splits SRAM into two regions, called "FLASH" and "RAM" as per
//! cortex-m-rt's `link.x` convention, via a modified `memory.x` linker script,
//! activated by the linker symbol `__run_from_ram=1`.
//!
//! This causes the binary to be linked to run from SRAM.
//!
//! The `blinky-ram` feature selects the `boot2-ram-memcpy` second stage bootloader.
//!
//! Two post-processing steps then:
//!   1. extract the 256-byte boot2 from the binary,
//!   2. join boot2 and the program into one image, based at the start of flash (`0x10000000`).
//!
//! This causes `probe-rs` to load the image into flash at the correct offset.
//! On reset, the boot ROM runs `boot2-ram-memcpy`, which copies the program to SRAM and runs it.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_time::Timer;
use gpio::{Level, Output};
use panic_probe as _;

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Fail loudly if the program is executing from flash (XIP) instead of SRAM.
    let pc = cortex_m::register::pc::read();
    defmt::assert!(
        (0x2000_0000..0x2004_2000).contains(&pc),
        "not running from SRAM, pc={:#x}",
        pc
    );
    info!("running from SRAM, pc={:#x}", pc);

    let mut led = Output::new(p.PIN_25, Level::Low);

    loop {
        info!("led on!");
        led.set_high();
        Timer::after_millis(500).await;

        info!("led off!");
        led.set_low();
        Timer::after_millis(500).await;
    }
}
