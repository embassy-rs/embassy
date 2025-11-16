#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::AfioRemap;
use embassy_stm32::pac::timer::vals::{Etp, Etps};
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::custom_timer::{CustomPwmBuilder, TriggerMode};
use embassy_stm32::timer::low_level::FilterValue;

use {defmt_rtt as _, panic_probe as _};

///            0               CCR              ARR          0               CCR
///            .                *                .           .                *
///            .                |                .           .                |
///            .                |                .           .                |
///  wait for  .                |                .           .                |
///  cfgd edge .                |                .           .                |
///            .                |                .           .                |
///            .                |                .           .                |
/// ----------------------------*---------------------------------------------*-------
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

    let trigger_pin = p.PA12;

    let mut tim = CustomPwmBuilder::<_, _, _, _, _, _, _>::new(p.TIM1)
        //.frequency(Hertz(123))
        .prescaler_and_period(0, 1337)
        .etr::<AfioRemap<0>>(trigger_pin, FilterValue::FDTS_DIV32_N8, Etp::NOT_INVERTED, Etps::DIV1)
        .trigger_from_etr(TriggerMode::TriggerMode)
        .ch1_internal(1234)
        .one_pulse_mode()
        .finalize();

    loop {
        // Should trigger 1234 ticks after PA12 goes high
        tim.wait_for_configured_edge(Channel::Ch1).await;

        info!("Edge detected 1234 ticks ago!");
    }
}
