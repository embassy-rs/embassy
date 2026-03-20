#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::Speed;
use embassy_stm32::hrtim::stm32_hrtim::compare_register::HrCompareRegister;
use embassy_stm32::hrtim::stm32_hrtim::output::{self, HrOutput};
use embassy_stm32::hrtim::stm32_hrtim::timer::{HrSlaveTimer, HrTimer};
use embassy_stm32::hrtim::stm32_hrtim::{HrPwmAdvExt, MasterPreloadSource, PreloadSource};
use embassy_stm32::hrtim::{self, HrControltExt, HrPwmBuilderExt};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
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

    let pin_lh_hs = hrtim::Pin {
        pin: p.PB14,
        speed: Speed::Low,
    };
    let pin_lh_ls = hrtim::Pin {
        pin: p.PB15,
        speed: Speed::Low,
    };
    let pin_rh_hs = hrtim::Pin {
        pin: p.PB12,
        speed: Speed::Low,
    };
    let pin_rh_ls = hrtim::Pin {
        pin: p.PB13,
        speed: Speed::Low,
    };

    let prescaler = hrtim::Pscl4;

    let hrtim::Parts {
        control,
        master,
        timc,
        timd,
        ..
    } = p.HRTIM1.hr_control();
    let (control, ..) = control.wait_for_calibration();
    let mut control = control.constrain();

    let mut master_timer = master
        .pwm_advanced(output::NoPin, output::NoPin)
        .prescaler(prescaler)
        .preload(MasterPreloadSource::OnMasterRepetitionUpdate)
        .period(0xFFFF)
        .finalize(&mut control);

    let mut bridge_left = timd
        .pwm_advanced(pin_lh_hs, pin_lh_ls)
        .prescaler(prescaler)
        .period(0xFFFF)
        .preload(PreloadSource::OnMasterTimerUpdate)
        .finalize(&mut control);

    let mut bridge_right = timc
        .pwm_advanced(pin_rh_hs, pin_rh_ls)
        .prescaler(prescaler)
        .period(0xFFFF)
        .preload(PreloadSource::OnMasterTimerUpdate)
        .finalize(&mut control);

    //            .               .               .               .
    //            .  33%          .               .               .
    //            .----           .----           .----           .----
    //pin_lh_hs   |    |          |    |          |    |          |    |
    //            |    |          |    |          |    |          |    |
    //            -    ------------    ------------    ------------    ----
    //            .     ----------.     ----------.     ----------.     ---
    //pin_lh_ls   .    |          |    |          |    |          |    |
    //            .    |          |    |          |    |          |    |
    //            ------          ------          ------          ------
    //            .     ----------.     ----------.     ----------.     ---
    //pin_rh_hs   .    |          |    |          |    |          |    |
    //            .    |          |    |          |    |          |    |
    //            ------          ------          ------          ------
    //            .----           .----           .----           .----
    //pin_rh_ls   |    |          |    |          |    |          |    |
    //            |    |          |    |          |    |          |    |
    //            -    ------------    ------------    ------------    ----
    //            .               .               .               .
    //            .               .               .               .

    bridge_left.timer.enable_reset_event(&master_timer.timer);
    bridge_right.timer.enable_reset_event(&master_timer.timer);

    bridge_left.out1.enable_set_event(&bridge_left.timer);
    bridge_left.out2.enable_set_event(&bridge_left.cr1);
    bridge_left.out1.enable_rst_event(&bridge_left.cr1);
    bridge_left.out2.enable_rst_event(&bridge_left.timer);

    bridge_right.out1.enable_set_event(&bridge_right.cr1);
    bridge_right.out2.enable_set_event(&bridge_right.timer);
    bridge_right.out1.enable_rst_event(&bridge_right.timer);
    bridge_right.out2.enable_rst_event(&bridge_right.cr1);

    info!("pwm constructed");

    bridge_left.timer.set_period(u16::MAX);
    bridge_left.cr1.set_duty(0);
    bridge_right.timer.set_period(u16::MAX);
    bridge_right.cr1.set_duty(0);

    bridge_left.out1.enable();
    bridge_left.out2.enable();
    bridge_right.out1.enable();
    bridge_right.out2.enable();

    control.control.start_stop_timers(|w| {
        w.start(&mut master_timer.timer)
            .start(&mut bridge_left.timer)
            .start(&mut bridge_right.timer)
    });

    loop {
        // Step duty cycle
        for i in 1..=u16::MAX {
            bridge_left.cr1.set_duty(i);
            bridge_right.cr1.set_duty(i);
        }
        for i in (1..=u16::MAX).rev() {
            bridge_left.cr1.set_duty(i);
            bridge_right.cr1.set_duty(i);
        }
    }
}
