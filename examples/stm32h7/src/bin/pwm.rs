#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::khz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let ch1_pin = PwmPin::new(p.PA6, OutputType::PushPull);
    let mut pwm = SimplePwm::new(p.TIM3, Some(ch1_pin), None, None, None, khz(10), Default::default());
    let mut ch1 = pwm.ch1();
    ch1.enable();

    info!("PWM initialized");
    info!("PWM max duty {}", ch1.max_duty_cycle());

    loop {
        ch1.set_duty_cycle_fully_off();
        Timer::after_millis(300).await;
        ch1.set_duty_cycle_fraction(1, 4);
        Timer::after_millis(300).await;
        ch1.set_duty_cycle_fraction(1, 2);
        Timer::after_millis(300).await;
        ch1.set_duty_cycle(ch1.max_duty_cycle() - 1);
        Timer::after_millis(300).await;
    }
}
