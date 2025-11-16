#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AfioRemap, OutputType};
use embassy_stm32::pac::timer::vals::{Etp, Etps};
use embassy_stm32::timer::custom_timer::{CustomPwmBuilder, TriggerMode, TriggerSource};
use embassy_stm32::timer::low_level::{FilterValue, InputCaptureMode, InputTISelection, OutputCompareMode};
use embassy_stm32::timer::{Channel, UpDma};

use embassy_stm32::timer::simple_pwm::PwmPin;
use embassy_stm32::{Peri, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

///            0               CCR              ARR          0               CCR
///            .                *----------------*           .                *-------
///            .                |                |           .                |
///            .                |                |           .                |
///  output    .                |                |           .                |
///            .                |                |           .                |
///            .                |                |           .                |
///            .                |                |           .                |
/// ----------------------------*                *----------------------------*
///            .                                 .           .
///            *---------------------------------------------*
///            |                                 .           |
///            |                                 .           |
/// trigger    |                                 .           |
///            |                                 .           |
///            |                                 .           |
///            |                                 .           |
/// -----------*                                 .           *------------------------
///            .                                 .
///            .                                 *
///            .                         .  *    |                                    .
///            .                   .  *          |                              .  *
///  counter   .             .  *                |                       .   *
///            .        .  *                     |                  .  *
///            .  .  *                           |           .   *
///            *                                 *---------*
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let dma = p.DMA1_CH5;
    let out_pin = PwmPin::new(p.PA9, OutputType::PushPull);
    let trigger_pin = p.PA8;
    let capture_pin = p.PA10;

    let mut tim = CustomPwmBuilder::new(p.TIM1)
        //.frequency(Hertz(123))
        .prescaler_and_period(0, 1337)
        .ch1_input::<AfioRemap<0>>(
            trigger_pin,
            FilterValue::FDTS_DIV32_N8,
            InputCaptureMode::BothEdges,
            InputTISelection::Normal,
            1,
        )
        .trigger_from_ch1(TriggerMode::TriggerMode, TriggerSource::Filtered)
        .ch2::<AfioRemap<0>>(out_pin, OutputCompareMode::PwmMode2, 800)
        .ch3_input::<AfioRemap<0>>(
            capture_pin,
            FilterValue::FCK_INT_N2,
            InputCaptureMode::Rising,
            InputTISelection::Normal,
            0,
        )
        .one_pulse_mode()
        .finalize();

    loop {
        info!("Manually set pulse width");
        tim.set_compare_value(150, Channel::Ch2);
        Timer::after_millis(300).await;

        info!("Send waveform on PA9");
        tim.waveform_up(dma, Channel::Ch1, &[100, 400, 800, 1100, 1200]).await;

        info!("Waiting for rising edge on PA10");
        let capture = tim.wait_for_configured_edge(Channel::Ch3).await;
        info!("Rising edge detected on PA10 {} ticks from trigger on PA8", capture);
    }
}
