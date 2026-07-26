#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use hal::bind_interrupts;
use hal::config::Config;
use hal::interrupt::typelevel::{self, Handler};
use hal::peripherals::WWDT0;
use hal::wwdt::{Instance, InterruptHandler, Watchdog};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        WWDT0 => InterruptHandler<WWDT0>, TestInterruptHandler<WWDT0>;
    }
);

static INTERRUPT_TRIGGERED: AtomicBool = AtomicBool::new(false);

pub struct TestInterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<typelevel::WWDT0> for TestInterruptHandler<T> {
    unsafe fn on_interrupt() {
        INTERRUPT_TRIGGERED.store(true, Ordering::Relaxed);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = hal::init(config);

    let wwdt_config = hal::wwdt::Config {
        timeout: Duration::from_millis(50),
        warning: Some(Duration::from_micros(4000)),
    };

    let mut watchdog = Watchdog::new(p.WWDT0, Irqs, wwdt_config).unwrap();

    assert!(!INTERRUPT_TRIGGERED.load(Ordering::Relaxed));

    // Set to watchdog to generate interrupt if it's not fed within 50ms, and start it.
    // The warning interrupt will trigger 4ms before the timeout.
    watchdog.start();

    for _ in 1..=5 {
        assert!(!INTERRUPT_TRIGGERED.load(Ordering::Relaxed));
        Timer::after_millis(10).await;
        watchdog.feed();
    }

    for _ in 1..=5 {
        assert!(!INTERRUPT_TRIGGERED.load(Ordering::Relaxed));
        Timer::after_millis(10).await;
    }

    assert!(INTERRUPT_TRIGGERED.load(Ordering::Relaxed));
    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
