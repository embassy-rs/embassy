#![no_std]
#![no_main]

use defmt::*;

use embassy_executor::Spawner;
use embassy_stm32::hrtim::{AdvancedPwm, BridgeConverter, ComplementaryPwmPin, FullBridgeConverter, PwmPin};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    info!("Hello World!");

    let ch3 = PwmPin::new_chc(p.PB12);
    let ch3n = ComplementaryPwmPin::new_chc(p.PB13);

    let ch4 = PwmPin::new_chd(p.PB14);
    let ch4n = ComplementaryPwmPin::new_chd(p.PB15);

    let mut pwm = AdvancedPwm::new(
        p.HRTIM1,
        None,
        None,
        None,
        None,
        Some(ch3),
        Some(ch3n),
        Some(ch4),
        Some(ch4n),
        None,
        None,
        None,
        None,
    );

    pwm.set_master_frequency(Hertz::mhz(1));

    info!("pwm constructed");

    let mut bridge = FullBridgeConverter::new(&mut pwm.ch_c, &mut pwm.ch_d, Hertz::mhz(1));

    let max_duty_master = bridge.get_max_compare_value_master();

    info!("max compare value master: {}", max_duty_master);

    bridge.set_dead_time(54); // 10ns * 5.376ns step/ns
    bridge.set_duty(max_duty_master / 2);
    bridge.set_minimum_duty(135); // 25ns * 5.376 step/ns

    bridge.start();

    Timer::after_millis(500).await;

    info!("end setup");

    let mut duty = 0;
    loop {
        bridge.set_duty(duty);

        if duty == max_duty_master {
            duty = 0;
        } else {
            duty = duty + 1;
        }

        Timer::after_millis(10).await;
    }
}
