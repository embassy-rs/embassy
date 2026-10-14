#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::{Duration, Instant, Timer};
use panic_probe as _;

#[cfg(not(feature = "time-driver-lptmr"))]
hal::bind_interrupts!(struct Irqs {
    LPTMR0 => hal::lptmr::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());
    #[cfg(not(feature = "time-driver-lptmr"))]
    {
        let mut timer = hal::lptmr::Lptmr::new(p.LPTMR0, Irqs, Default::default()).unwrap();
        defmt::info!("LPTMR counter clock: {} Hz", timer.frequency());
        for _ in 0..3 {
            let start = Instant::now();
            timer.delay(Duration::from_millis(20)).await.unwrap();
            assert!(start.elapsed() >= Duration::from_millis(20));
        }
        timer.blocking_wait_ticks(1_000).unwrap();
        defmt::info!("LPTMR polling and async waits passed");
    }
    #[cfg(feature = "time-driver-lptmr")]
    {
        let _ = p;
        let start = Instant::now();
        Timer::after_millis(20).await;
        assert!(start.elapsed() >= Duration::from_millis(20));
        defmt::info!("LPTMR time backend passed");
    }
    #[cfg(not(feature = "time-driver-lptmr"))]
    Timer::after_millis(1).await;
    defmt::info!("Example complete");
    cortex_m::asm::bkpt();
}
