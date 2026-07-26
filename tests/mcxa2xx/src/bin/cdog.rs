#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::bind_interrupts;
use hal::cdog::{FaultControl, InterruptHandler, LockControl, PauseControl, Watchdog};
use hal::config::Config;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        CDOG0 => InterruptHandler;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = hal::init(config);

    let cdog_config = hal::cdog::Config {
        timeout: FaultControl::EnableInterrupt,
        miscompare: FaultControl::EnableInterrupt,
        sequence: FaultControl::EnableInterrupt,
        state: FaultControl::EnableInterrupt,
        address: FaultControl::EnableInterrupt,
        irq_pause: PauseControl::PauseTimer,
        debug_halt: PauseControl::PauseTimer,
        lock: LockControl::Unlocked,
    };

    let mut watchdog = Watchdog::new(p.CDOG0, Irqs, cdog_config).unwrap();

    // First part of the example is to demonstrate how the secure counter feature of the cdog works.
    watchdog.start(0xFFFFFF, 0);
    watchdog.add(42);
    watchdog.check(42);
    watchdog.sub(2);
    watchdog.check(40);
    watchdog.start(0xFFFFFFFF, 0);
    watchdog.check(0);

    // Next check should generate an interrupt as checked value (=1) is different than the secure counter (=0).
    watchdog.check(1);

    // Now demonstrating how the instruction timer feature of the cdog works.
    watchdog.start(0xFFF, 0);
    assert_ne!(watchdog.get_instruction_timer(), 0);

    while watchdog.get_instruction_timer() != 0 {
        Timer::after_millis(1).await;
    }

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
