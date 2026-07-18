// required-features: tamp

// Scoped to what's testable without extra wiring: backup register
// read/write, and the monotonic counter. External/internal tamper detection
// needs either external wiring or triggering a chip-specific, possibly
// destructive condition (e.g. some internal tampers erase debug access), so
// they're left to the manual hardware test in examples/stm32wba6/src/bin/tamp.rs.

#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::tamp::{self, Tamp};

bind_interrupts!(struct Irqs{
    TAMP => tamp::InterruptHandler;
});

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let p = init();

    let mut tamp = Tamp::new(p.TAMP, Irqs);

    // Out-of-range register.
    defmt::assert_eq!(tamp.read_backup_register(32), None);

    // Read/write round-trip, and independence between registers.
    tamp.write_backup_register(0, 0xdead_beef);
    tamp.write_backup_register(1, 0x1234_5678);
    defmt::assert_eq!(tamp.read_backup_register(0), Some(0xdead_beef));
    defmt::assert_eq!(tamp.read_backup_register(1), Some(0x1234_5678));

    tamp.write_backup_register(0, 0);
    defmt::assert_eq!(tamp.read_backup_register(0), Some(0));
    defmt::assert_eq!(tamp.read_backup_register(1), Some(0x1234_5678));

    // Each read of COUNTR performs an internal write access that increments it.
    let c0 = tamp.monotonic_counter();
    let c1 = tamp.monotonic_counter();
    let c2 = tamp.monotonic_counter();
    defmt::assert_eq!(c1, c0 + 1);
    defmt::assert_eq!(c2, c1 + 1);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
