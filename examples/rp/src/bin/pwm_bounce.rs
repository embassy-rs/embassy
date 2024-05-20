#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::pwm::{Config, Pwm};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::pre_init]
unsafe fn before_main() {
    embassy_rp::pac::SIO.spinlock(31).write_value(1);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut c: Config = Default::default();
    c.top = u16::MAX;
    c.compare_a = 0;
    c.compare_b = 0;
    let mut counter = 0u16;

    let mut pwm = Pwm::new_output_ab(p.PWM_SLICE0, p.PIN_16, p.PIN_17, c.clone());
    enum Bounce {
        Forward,
        Backward,
    }
    let mut bounce = Bounce::Forward;

    loop {
        Timer::after_micros(3).await;
        if counter >= u16::MAX {
            bounce = Bounce::Backward;
        }
        if counter <= 0 {
            bounce = Bounce::Forward;
        }
        match bounce {
            Bounce::Forward => {
                counter += 1;
            }
            Bounce::Backward => {
                counter -= 1;
            }
        }
        c.compare_a = counter;
        c.compare_b = u16::MAX - counter;
        pwm.set_config(&c);
    }
}
