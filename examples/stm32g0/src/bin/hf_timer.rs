#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::pac::rcc::vals::Tim1sel;
use embassy_stm32::rcc::{ClockSrc, Config as RccConfig, PllConfig, PllSource, Pllm, Plln, Pllq, Pllr};
use embassy_stm32::time::khz;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin};
use embassy_stm32::timer::simple_pwm::PwmPin;
use embassy_stm32::timer::Channel;
use embassy_stm32::{pac, Config as PeripheralConfig};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut rcc_config = RccConfig::default();
    rcc_config.mux = ClockSrc::PLL(PllConfig {
        source: PllSource::HSI,
        m: Pllm::DIV1,
        n: Plln::MUL16,
        r: Pllr::DIV4,       // CPU clock comes from PLLR (HSI (16MHz) / 1 * 16 / 4 = 64MHz)
        q: Some(Pllq::DIV2), // TIM1 or TIM15 can be sourced from PLLQ (HSI (16MHz) / 1 * 16 / 2 = 128MHz)
        p: None,
    });

    let mut peripheral_config = PeripheralConfig::default();
    peripheral_config.rcc = rcc_config;

    let p = embassy_stm32::init(peripheral_config);

    // configure TIM1 mux to select PLLQ as clock source
    // https://www.st.com/resource/en/reference_manual/rm0444-stm32g0x1-advanced-armbased-32bit-mcus-stmicroelectronics.pdf
    // RM0444 page 210
    // RCC - Peripherals Independent Clock Control Register - bit 22 -> 1
    pac::RCC.ccipr().modify(|w| w.set_tim1sel(Tim1sel::PLL1_Q));

    let ch1 = PwmPin::new_ch1(p.PA8, OutputType::PushPull);
    let ch1n = ComplementaryPwmPin::new_ch1(p.PA7, OutputType::PushPull);

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
