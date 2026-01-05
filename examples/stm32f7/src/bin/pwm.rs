#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::{Hertz, mhz};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// If you are trying this and your USB device doesn't connect, the most
// common issues are the RCC config and vbus_detection
//
// See https://embassy.dev/book/#_the_usb_examples_are_not_working_on_my_board_is_there_anything_else_i_need_to_configure
// for more information.
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL200,
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 200 / 2 = 200Mhz
            divq: Some(PllQDiv::DIV4), // 8mhz / 4 * 200 / 4 = 100Mhz
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    let p = embassy_stm32::init(config);
    let ch1_pin = PwmPin::new(p.PE9, OutputType::PushPull);
    let mut pwm = SimplePwm::new(p.TIM1, Some(ch1_pin), None, None, None, mhz(1), Default::default());
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
