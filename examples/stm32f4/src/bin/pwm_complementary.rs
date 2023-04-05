#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::pwm::complementary_pwm::{Ckd, ComplementaryPwm, ComplementaryPwmPin};
use embassy_stm32::pwm::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::pwm::Channel;
use embassy_stm32::time::khz;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let ch1 = PwmPin::new_ch1(p.PE9);
    let ch1n = ComplementaryPwmPin::new_ch1(p.PA7);
    let mut pwm = ComplementaryPwm::new(
        p.TIM1,
        Some(ch1),
        Some(ch1n),
        None,
        None,
        None,
        None,
        None,
        None,
        khz(10),
    );

    /*
        Dead-time = T_clk * T_dts * T_dtg

        T_dts:
        This bit-field indicates the division ratio between the timer clock (CK_INT) frequency and the
        dead-time and sampling clock (tDTS)used by the dead-time generators and the digital filters
        (ETR, TIx),
        00: tDTS=tCK_INT
        01: tDTS=2*tCK_INT
        10: tDTS=4*tCK_INT

        T_dtg:
        This bit-field defines the duration of the dead-time inserted between the complementary
        outputs. DT correspond to this duration.
        DTG[7:5]=0xx => DT=DTG[7:0]x tdtg with tdtg=tDTS.
        DTG[7:5]=10x => DT=(64+DTG[5:0])xtdtg with Tdtg=2xtDTS.
        DTG[7:5]=110 => DT=(32+DTG[4:0])xtdtg with Tdtg=8xtDTS.
        DTG[7:5]=111 => DT=(32+DTG[4:0])xtdtg with Tdtg=16xtDTS.
        Example if TDTS=125ns (8MHz), dead-time possible values are:
        0 to 15875 ns by 125 ns steps,
        16 us to 31750 ns by 250 ns steps,
        32 us to 63us by 1 us steps,
        64 us to 126 us by 2 us steps
    */
    pwm.set_dead_time_clock_division(Ckd::DIV1);
    pwm.set_dead_time_value(0);

    let max = pwm.get_max_duty();
    pwm.enable(Channel::Ch1);

    info!("PWM initialized");
    info!("PWM max duty {}", max);

    loop {
        pwm.set_duty(Channel::Ch1, 0);
        Timer::after(Duration::from_millis(300)).await;
        pwm.set_duty(Channel::Ch1, max / 4);
        Timer::after(Duration::from_millis(300)).await;
        pwm.set_duty(Channel::Ch1, max / 2);
        Timer::after(Duration::from_millis(300)).await;
        pwm.set_duty(Channel::Ch1, max - 1);
        Timer::after(Duration::from_millis(300)).await;
    }
}
