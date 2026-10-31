//! This example tests the RP Pico on board LED, but is run from SRAM.
//!
//! It does not blink on the RP Pico W board. See `blinky_wifi.rs`.
//!
//! Same code as `blinky.rs`, plus a check that it is really running from SRAM.
//!
//! Compile, post-process and run with `run-blinky-ram.sh`.
//!
//! Picotool is required to be installed and in the system PATH.
//! A compiled copy of picotool can be obtained at
//! https://github.com/raspberrypi/pico-sdk-tools/releases.
//!
//! The example works by splitting SRAM into equally-sized regions, labelled "FLASH" and "RAM"
//! as per cortex-m-rt's `link.x` convention, via a modified `memory.x` linker script,
//! activated by the linker symbol `__run_from_ram=1`.
//!
//! This causes the binary to be linked to run from SRAM.
//!
//! Two picotool post-processing steps then:
//!   1. add RP2350 boot load map metadata to the binary, telling the boot ROM what
//!      to copy to SRAM and where
//!   2. "re-address" the image to the start of the real chip flash (`0x10000000`).
//!
//! This causes `probe-rs` to load the binary into flash at the correct offset.
//! On reset, the boot ROM will automatically copy it to SRAM and run it there.
//!
//! See the modifications to `build.rs` for this binary and the `memory-ram-data.x`
//! linker script, which:
//!   1. links the `.data` section at its load address, so the boot ROM
//!      copies it with the rest of the image.
//!   2. prevents `cortex-m-rt`'s existing initialization mechanism from overwriting `.data`,
//!      by setting `__sidata` (the address `cortex-m-rt` copies `.data` from) to `__sdata`
//!      (where it copies to) so the copy is a no-op.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_time::Timer;
use gpio::{Level, Output};
use panic_probe as _;

// Program metadata for `picotool info`.
// This isn't needed, but it's recommended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Blinky Example"),
    embassy_rp::binary_info::rp_program_description!(
        c"This example tests the RP Pico on board LED, connected to gpio 25"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Fail loudly if the program is executing from flash (XIP) instead of SRAM.
    let pc = cortex_m::register::pc::read();
    defmt::assert!(
        (0x2000_0000..0x2008_2000).contains(&pc),
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
