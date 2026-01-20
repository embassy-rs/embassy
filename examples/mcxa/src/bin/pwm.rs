#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::ctimer::pwm::{Pwm, SetDutyCycle};
use hal::ctimer::{CTimer, InterruptHandler};
use hal::peripherals::CTIMER2;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        CTIMER2 => InterruptHandler<CTIMER2>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("Pwm example");

    let ctimer = CTimer::new(p.CTIMER2, Irqs, Default::default()).unwrap();
    let channels = ctimer.split();
    let (ch0, _ch1, _ch2, ch3) = channels.split();
    let mut pwm = Pwm::new_single_output(ch3, ch0, p.P3_18, Default::default()).unwrap();

    let mut duty: u8 = 0;
    let mut delta: i8 = 1;

    loop {
        // Fade LED in and out
        pwm.set_duty_cycle_percent(duty).unwrap();
        duty = ((duty as i8) + delta) as u8;

        if duty == 100 || duty == 0 {
            delta *= -1;
        }

        Timer::after_millis(10).await;
    }
}
