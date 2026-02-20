#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::bind_interrupts;
use hal::cdog::{Cdog, FaultControl, InterruptHandler, LockControl, PauseControl};
use hal::config::Config;
use hal::peripherals::CDOG0;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        CDOG0 => InterruptHandler<CDOG0>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = hal::init(config);

    defmt::info!("** Code watchdog example **");

    let mut config = hal::cdog::Config::default();
    config.timeout = FaultControl::EnableInterrupt;
    config.miscompare = FaultControl::EnableInterrupt;
    config.sequence = FaultControl::EnableInterrupt;
    config.state = FaultControl::EnableInterrupt;
    config.address = FaultControl::EnableInterrupt;
    config.irq_pause = PauseControl::PauseTimer;
    config.debug_halt = PauseControl::PauseTimer;
    config.lock = LockControl::Unlocked;

    let mut cdog = Cdog::new(p.CDOG0, Irqs, config).unwrap();

    defmt::info!("Watchdog initialized");

    // First part of the example is to demonstrate how the secure counter feature of the cdog works.
    cdog.start(0xffffff, 0).unwrap();
    let mut secure_counter = cdog.secure_counter();

    secure_counter += 42;
    secure_counter.validate(42).unwrap();

    secure_counter -= 2;
    secure_counter.validate(40).unwrap();

    cdog.stop(40).unwrap();
    cdog.start(0xffffffff, 0).unwrap();
    secure_counter.validate(0).unwrap();

    defmt::info!(
        "Next check should generate an interrupt as checked value (=1) is different than the secure counter (=0)"
    );
    secure_counter.validate(1).unwrap();

    // Write a value to persistent storage and read it back
    cdog.persistent_value().write(42);
    assert_eq!(cdog.persistent_value().read(), 42);

    // Now demonstrating how the instruction timer feature of the cdog works.
    defmt::info!("Start again the code cdog to generate a timeout interrupt");
    cdog.start(0xffff, 0).unwrap();

    loop {
        let timer = cdog.instruction_timer().read();
        if timer != 0 {
            defmt::info!("Waiting for interrupt: {}", timer);
            Timer::after_millis(250).await;
        } else {
            break;
        }
    }
}
