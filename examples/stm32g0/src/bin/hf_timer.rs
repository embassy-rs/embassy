#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::Config as PeripheralConfig;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::khz;
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin};
use embassy_stm32::timer::simple_pwm::PwmPin;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = PeripheralConfig::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(Hsi {
            sys_div: HsiSysDiv::DIV1,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL16,
            divp: None,
            divq: Some(PllQDiv::DIV2), // 16 / 1 * 16 / 2 = 128 Mhz
            divr: Some(PllRDiv::DIV4), // 16 / 1 * 16 / 4 = 64 Mhz
        });
        config.rcc.sys = Sysclk::PLL1_R;

        // configure TIM1 mux to select PLLQ as clock source
        // https://www.st.com/resource/en/reference_manual/rm0444-stm32g0x1-advanced-armbased-32bit-mcus-stmicroelectronics.pdf
        // RM0444 page 210
        // RCC - Peripherals Independent Clock Control Register - bit 22 -> 1
        config.rcc.mux.tim1sel = embassy_stm32::rcc::mux::Tim1sel::PLL1_Q;
    }
    let p = embassy_stm32::init(config);

    let ch1 = PwmPin::new(p.PA8, OutputType::PushPull);
    let ch1n = ComplementaryPwmPin::new(p.PA7, OutputType::PushPull);

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
        khz(512),
        Default::default(),
    );

    let max = pwm.get_max_duty();
    info!("Max duty: {}", max);

    pwm.set_duty(Channel::Ch1, max / 2);
    pwm.enable(Channel::Ch1);

    loop {}
}
