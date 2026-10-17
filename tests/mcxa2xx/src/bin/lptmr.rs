#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::{Duration, Instant, Timer};
#[cfg(not(feature = "time-driver-lptmr"))]
use hal::{
    bind_interrupts,
    lptmr::{InterruptHandler, Lptmr},
};
use panic_probe as _;

#[cfg(not(feature = "time-driver-lptmr"))]
bind_interrupts!(struct Irqs {
    LPTMR0 => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    #[cfg(not(feature = "time-driver-lptmr"))]
    test_driver().await;
    #[cfg(feature = "time-driver-lptmr")]
    test_time_driver().await;

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}

#[cfg(not(feature = "time-driver-lptmr"))]
async fn test_driver() {
    let p = hal::init(Default::default());
    let mut peri = p.LPTMR0;
    {
        let mut timer = Lptmr::new(peri.reborrow(), Irqs, Default::default()).unwrap();
        assert_eq!(timer.frequency(), 1_000_000);
        assert!(timer.start(0).is_err());
        assert!(!timer.is_running());

        timer.start(50_000).unwrap();
        Timer::after_millis(5).await;
        assert!(!timer.has_elapsed());
        assert!(timer.counter() > 0);
        timer.stop();
        assert_eq!(timer.counter(), 0);

        let start = Instant::now();
        timer.wait_ticks(20_000).await.unwrap();
        assert!(start.elapsed() >= Duration::from_millis(20));
        assert!(!timer.is_running());

        {
            let wait = timer.wait_ticks(100_000);
            embassy_futures::select::select(wait, Timer::after_millis(5)).await;
        }
        assert!(!timer.is_running());
        timer.wait_ticks(2_000).await.unwrap();

        // The compare may arrive before the wait is first polled. It must
        // complete from the latched status without requiring another IRQ.
        timer.start(1_000).unwrap();
        Timer::after_millis(5).await;
        timer.wait_for_compare().await.unwrap();
        assert!(!timer.is_running());

        // Exercise registration/completion races and reuse after each wait.
        for _ in 0..64 {
            timer.wait_ticks(1).await.unwrap();
            assert!(!timer.is_running());
        }

        timer.start(1_000).unwrap();
        Timer::after_millis(5).await;
        assert!(timer.has_elapsed());
        timer.clear_compare_flag();
        timer.stop();
        assert!(!timer.has_elapsed());
    }

    // Recreate after drop, with both MRCC and counter prescalers applied.
    let mut config = hal::lptmr::Config::default();
    config.div = hal::clocks::periph_helpers::Div4::from_divisor(2).unwrap();
    config.prescaler = hal::lptmr::Prescaler::divide(1).unwrap();
    config.free_running = true;
    {
        let mut timer = Lptmr::new(peri.reborrow(), Irqs, config).unwrap();
        assert_eq!(timer.frequency(), 250_000);
        let start = Instant::now();
        timer.blocking_wait_ticks(5_000).unwrap();
        assert!(start.elapsed() >= Duration::from_millis(20));
        timer.start(250).unwrap();
        Timer::after_millis(5).await;
        assert!(timer.has_elapsed());
        assert!(timer.counter() > 250);
        timer.clear_compare_flag();
        assert!(!timer.has_elapsed());
    }

    // Flooring a fractional clock to 1 Hz would make a 1-second delay finish
    // after one actual 1.907-Hz tick (about 524ms). Reject that configuration.
    let mut fractional = config;
    fractional.div = hal::clocks::periph_helpers::Div4::from_divisor(16).unwrap();
    fractional.prescaler = hal::lptmr::Prescaler::divide(15).unwrap();
    assert!(Lptmr::new(peri.reborrow(), Irqs, fractional).is_err());

    // Direct FRO16K bypasses MRCC; it must not silently honor its divider.
    config.source = hal::clocks::periph_helpers::LptmrClockSel::Clk16K;
    assert!(Lptmr::new(peri.reborrow(), Irqs, config).is_err());
    config.div = hal::clocks::periph_helpers::Div4::no_div();
    config.prescaler = hal::lptmr::Prescaler::bypass();
    let mut timer = Lptmr::new(peri, Irqs, config).unwrap();
    assert_eq!(timer.frequency(), 16_384);
    timer.delay(Duration::from_millis(10)).await.unwrap();
    assert!(!timer.is_running());
}

#[cfg(feature = "time-driver-lptmr")]
async fn test_time_driver() {
    hal::init(Default::default());
    let frequency = hal::clocks::with_clocks(|c| c.cpu_system_clk.as_ref().unwrap().frequency).unwrap();
    let start = Instant::now();
    Timer::after_millis(20).await;
    assert!(start.elapsed() >= Duration::from_millis(20));

    let (a, b) = embassy_futures::join::join(
        async {
            Timer::after_millis(2).await;
            Instant::now()
        },
        async {
            Timer::after_millis(8).await;
            Instant::now()
        },
    )
    .await;
    assert!(a <= b);

    // Delay servicing compare IRQs without stopping the counter. The backend
    // must account for actual elapsed ticks, not count serviced interrupts.
    let before = Instant::now();
    critical_section::with(|_| cortex_m::asm::delay(frequency / 100));
    assert!(before.elapsed() >= Duration::from_millis(10));
    Timer::after_millis(2).await;

    let mut previous = Instant::now();
    for _ in 0..10_000 {
        let now = Instant::now();
        assert!(now >= previous);
        previous = now;
    }
    let start = Instant::now();
    Timer::after_micros(10).await;
    assert!(start.elapsed() >= Duration::from_micros(10));

    // Cancel a long alarm, then enqueue an earlier one on the same task.
    embassy_futures::select::select(Timer::after_millis(100), Timer::after_millis(2)).await;
    let start = Instant::now();
    Timer::after_millis(2).await;
    assert!(start.elapsed() >= Duration::from_millis(2));
}
