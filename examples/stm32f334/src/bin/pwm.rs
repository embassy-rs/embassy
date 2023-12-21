#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::hrtim::*;
use embassy_stm32::rcc::HrtimClockSource;
use embassy_stm32::time::{khz, mhz};
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config: Config = Default::default();
    config.rcc.sysclk = Some(mhz(64));
    config.rcc.hclk = Some(mhz(64));
    config.rcc.pclk1 = Some(mhz(32));
    config.rcc.pclk2 = Some(mhz(64));
    config.rcc.hrtim = HrtimClockSource::PllClk;

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
    //        .modify(|w| w.set_sst(Activeeffect::SETACTIVE));
    //
    //    Timer::after_millis(500).await;
    //
    //    embassy_stm32::pac::HRTIM1
    //        .tim(0)
    //        .rstr(0)
    //        .modify(|w| w.set_srt(Inactiveeffect::SETINACTIVE));

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
