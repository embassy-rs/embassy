#![no_std]
#![no_main]

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

    defmt::info!("** Code watchdog example **");

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

    defmt::info!("Watchdog initialized");

    // First part of the example is to demonstrate how the secure counter feature of the cdog works.
    watchdog.start(0xFFFFFF, 0);
    watchdog.add(42);
    watchdog.check(42);
    watchdog.sub(2);
    watchdog.check(40);
    watchdog.start(0xFFFFFFFF, 0);
    watchdog.check(0);
    defmt::info!(
        "Next check should generate an interrupt as checked value (=1) is different than the secure counter (=0)"
    );
    watchdog.check(1);

    // Now demonstrating how the instruction timer feature of the cdog works.
    defmt::info!("Start again the code watchdog to generate a timeout interrupt");
    watchdog.start(0xFFFFF, 0);
    while watchdog.get_instruction_timer() != 0 {
        defmt::info!("Instruction timer : {:08x}", watchdog.get_instruction_timer());
        Timer::after_millis(100).await;
    }

    defmt::info!("** End of example **");
}
