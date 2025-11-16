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
use {defmt_rtt as _, panic_probe as _};

/// Connect PA0 and PC13 with a 1k Ohm resistor

async fn _example(
    tim: Peri<'_, crate::peripherals::TIM1>,
    dma: Peri<'_, impl UpDma<crate::peripherals::TIM1>>,
    trigger_pin: crate::peripherals::PA8,
    out_pin: Peri<'_, crate::peripherals::PA9>,
    capture_pin: crate::peripherals::PA10,
) {
    let out_pin = PwmPin::new(out_pin, OutputType::PushPull);

    let mut tim = CustomPwmBuilder::new(tim)
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

    tim.set_compare_value(150, Channel::Ch2);
    tim.waveform_up(dma, Channel::Ch1, &[100, 400, 800, 1100, 1200]).await;
    let _capture = tim.wait_for_configured_edge(Channel::Ch3).await;
}

async fn _example2(tim: Peri<'_, crate::peripherals::TIM1>, trigger_pin: crate::peripherals::PA12) {
    let mut tim = CustomPwmBuilder::<_, _, _, _, _, _, _>::new(tim)
        //.frequency(Hertz(123))
        .prescaler_and_period(0, 1337)
        .etr::<AfioRemap<0>>(trigger_pin, FilterValue::FDTS_DIV32_N8, Etp::NOT_INVERTED, Etps::DIV1)
        .trigger_from_etr(TriggerMode::TriggerMode)
        .ch1_internal(1234)
        .one_pulse_mode()
        .finalize();

    // Should trigger 1234 ticks after PA12 goes high
    let _capture = tim.wait_for_configured_edge(Channel::Ch1).await;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_stm32::init(Default::default());
    info!("Hello World!");
}
