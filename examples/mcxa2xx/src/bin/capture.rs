#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::ctimer::CTimer;
use hal::ctimer::capture::{self, Capture, Edge, InterruptHandler};
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use hal::peripherals::CTIMER2;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    CTIMER2 => InterruptHandler<CTIMER2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("Capture example");

    let pin = Output::new(p.P3_12, Level::Low, DriveStrength::Normal, SlewRate::Fast);

    let ctimer = CTimer::new(p.CTIMER2, Default::default()).unwrap();
    let mut config = capture::Config::default();
    config.edge = Edge::RisingEdge;
    let mut capture = Capture::new_with_input_pin(ctimer, p.CTIMER2_CH0, p.P3_14, Irqs, config).unwrap();

    spawner.spawn(pin_task(pin).unwrap());

    loop {
        let one = capture.capture().await.unwrap();
        let two = capture.capture().await.unwrap();
        let diff = two - one;
        defmt::info!(
            "{} s {} Hz",
            diff.to_period(capture.frequency()),
            diff.to_frequency(capture.frequency())
        );
    }
}

#[embassy_executor::task]
async fn pin_task(mut pin: Output<'static>) {
    Timer::after_secs(1).await;

    loop {
        pin.set_high();
        pin.set_low();
        Timer::after_secs(1).await;
    }
}
