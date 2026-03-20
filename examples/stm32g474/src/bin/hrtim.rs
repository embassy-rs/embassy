#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::Speed;
use embassy_stm32::hrtim::stm32_hrtim::compare_register::HrCompareRegister;
use embassy_stm32::hrtim::stm32_hrtim::output::HrOutput;
use embassy_stm32::hrtim::stm32_hrtim::timer::HrTimer;
use embassy_stm32::hrtim::stm32_hrtim::{HrParts, HrPwmAdvExt, PreloadSource};
use embassy_stm32::hrtim::{HrControltExt, HrPwmBuilderExt, Parts};
use embassy_stm32::{Config, hrtim};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        // Set system frequency to 16MHz * 15/1/2 = 120MHz
        // This would lead to HrTim running at 120MHz * 32 = 3.84GHz...
        use embassy_stm32::rcc::*;
        config.rcc.hsi = true;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2),
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL15,
        });
        config.rcc.sys = Sysclk::PLL1_R;
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let pin1 = hrtim::Pin {
        pin: p.PA8,
        speed: Speed::Low,
    };
    let pin2 = hrtim::Pin {
        pin: p.PA9,
        speed: Speed::Low,
    };

    // ...with a prescaler of 4 this gives us a HrTimer with a tick rate of 960MHz
    // With max the max period set, this would be 960MHz/2^16 ~= 14.6kHz...
    let prescaler = hrtim::Pscl4;

    let Parts { control, tima, .. } = p.HRTIM1.hr_control();
    let (control, ..) = control.wait_for_calibration();
    let mut control = control.constrain();

    //        .               .               .               .
    //        .  30%          .               .               .
    //        .----           .               .----           .
    //pin1    |    |          .               |    |          .
    //        |    |          .               |    |          .
    // --------    ----------------------------    --------------------
    //        .               .----           .               .----
    //pin2    .               |    |          .               |    |
    //        .               |    |          .               |    |
    // ------------------------    ----------------------------    ----
    //        .               .               .               .
    //        .               .               .               .

    let HrParts {
        mut timer,
        mut cr1,
        mut out1,
        mut out2,
        ..
    } = tima
        .pwm_advanced(pin1, pin2)
        .prescaler(prescaler)
        .period(0xFFFF)
        .push_pull_mode(true) // Set push pull mode, out1 and out2 are
        // alternated every period with one being
        // inactive and the other getting to output its wave form
        // as normal
        .preload(PreloadSource::OnRepetitionUpdate)
        .finalize(&mut control);

    out1.enable_rst_event(&cr1); // Set low on compare match with cr1
    out2.enable_rst_event(&cr1);

    out1.enable_set_event(&timer); // Set high at new period
    out2.enable_set_event(&timer);

    info!("pwm constructed");

    out1.enable();
    out2.enable();
    timer.start(&mut control.control);

    // Step frequency from 14.6kHz to about 146kHz(half of that when only looking at one pin)
    for i in 1..=10 {
        let new_period = u16::MAX / i;

        cr1.set_duty(new_period / 3);
        timer.set_period(new_period);

        Timer::after_millis(1).await;
    }

    out1.disable();
    out2.disable();

    info!("end program");

    cortex_m::asm::bkpt();
}
