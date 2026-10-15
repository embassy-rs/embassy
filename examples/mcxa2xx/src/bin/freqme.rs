#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_embedded_hal::SetConfig;
use embassy_executor::Spawner;
use embassy_mcxa::clocks::periph_helpers::FreqmeTargetClockSel;
use embassy_mcxa::freqme::{Config, Freqme, InterruptHandler, ReferenceScale};
use embassy_mcxa::{bind_interrupts, peripherals};
use embassy_time::{Duration, Timer, with_timeout};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    FREQME0 => InterruptHandler<peripherals::FREQME0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_mcxa::config::Config::default();
    config.clock_cfg.sirc.fro_12m_enabled = true;
    let p = embassy_mcxa::init(config);
    let mut config = Config::default();
    config.target = FreqmeTargetClockSel::Fro12m;
    config.reference_scale = ReferenceScale::new(16).unwrap();
    let mut freqme = Freqme::new_async(p.FREQME0, Irqs, config).unwrap();
    for _ in 0..8 {
        let result = with_timeout(Duration::from_millis(100), freqme.measure())
            .await
            .unwrap()
            .unwrap();
        defmt::info!(
            "FRO12M: raw={}, SDK frequency={} Hz",
            result.raw_count(),
            result.frequency_hz().unwrap()
        );
    }
    // Change the reference window without reconstructing the driver.
    config.reference_scale = ReferenceScale::new(14).unwrap();
    freqme.set_config(&config).unwrap();

    // These nonblocking controls are shared by Blocking and Async drivers.
    freqme.oneshot();
    Timer::after_millis(10).await;
    let result = freqme.result().expect("one-shot measurement not ready");
    defmt::info!("Polled one-shot: {} Hz", result.frequency_hz().unwrap());

    // The configured 16,384-cycle window is repeated automatically. Read snapshots
    // of the latest completed result without restarting the measurement.
    freqme.continuous();
    for _ in 0..4 {
        Timer::after_millis(10).await;
        let result = freqme.result().expect("continuous measurement not ready");
        defmt::info!(
            "Continuous: raw={}, {} Hz",
            result.raw_count(),
            result.frequency_hz().unwrap()
        );
    }
    freqme.stop();
    defmt::info!("FREQME example complete");
    cortex_m::asm::bkpt();
}
