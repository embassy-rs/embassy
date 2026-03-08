#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::Speed;
use embassy_stm32::hrtim::bridge_converter::BridgeConverter;
use embassy_stm32::hrtim::*;
use embassy_stm32::time::{khz, mhz};
use embassy_stm32::{Config, hrtim};
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

    let ch1 = hrtim::Pin {
        pin: p.PA8,
        speed: Speed::Low,
    };
    let ch1n = hrtim::Pin {
        pin: p.PA9,
        speed: Speed::Low,
    };

    // ...with a prescaler of 4 this gives us a HrTimer with a tick rate of 1152MHz
    // With max the max period set, this would be 1152MHz/2^16 ~= 17.6kHz...
    let prescaler = hrtim::Pscl4;

    let Parts { control, tima, .. } = p.HRTIM1.hr_control();
    let (control, ..) = control.wait_for_calibration();
    let mut control = control.constrain();

    info!("pwm constructed");

    let mut buck_converter = BridgeConverter::new(tima, ch1, ch1n, khz(5), prescaler, &mut control);

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

    buck_converter.start(&mut control.control);

    Timer::after_millis(500).await;

    info!("end program");

    cortex_m::asm::bkpt();
}
