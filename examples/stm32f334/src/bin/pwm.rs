#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::hrtim::*;
use embassy_stm32::time::{khz, mhz};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: mhz(8),
            mode: HseMode::Bypass,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;

        config.rcc.mux.hrtim1sw = embassy_stm32::rcc::mux::Timsw::PLL1_P;
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let ch1 = PwmPin::new_cha(p.PA8);
    let ch1n = ComplementaryPwmPin::new_cha(p.PA9);
    let pwm = AdvancedPwm::new(
        p.HRTIM1,
        Some(ch1),
        Some(ch1n),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    info!("pwm constructed");

    let mut buck_converter = BridgeConverter::new(pwm.ch_a, khz(5));

    //    embassy_stm32::pac::HRTIM1
    //        .tim(0)
    //        .setr(0)
    //        .modify(|w| w.set_sst(true));
    //
    //    Timer::after_millis(500).await;
    //
    //    embassy_stm32::pac::HRTIM1
    //        .tim(0)
    //        .rstr(0)
    //        .modify(|w| w.set_srt(true));

    let max_duty = buck_converter.get_max_compare_value();

    info!("max compare value: {}", max_duty);

    buck_converter.set_dead_time(max_duty / 20);
    buck_converter.set_primary_duty(max_duty / 2);
    buck_converter.set_secondary_duty(3 * max_duty / 4);

    buck_converter.start();

    Timer::after_millis(500).await;

    info!("end program");

    cortex_m::asm::bkpt();
}
