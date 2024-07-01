#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::tsc::{self, *};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::exception]
unsafe fn HardFault(_: &cortex_m_rt::ExceptionFrame) -> ! {
    cortex_m::peripheral::SCB::sys_reset();
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let device_config = embassy_stm32::Config::default();
    let context = embassy_stm32::init(device_config);

    let config = tsc::Config {
        ct_pulse_high_length: ChargeTransferPulseCycle::_2,
        ct_pulse_low_length: ChargeTransferPulseCycle::_2,
        spread_spectrum: false,
        spread_spectrum_deviation: SSDeviation::new(2).unwrap(),
        spread_spectrum_prescaler: false,
        pulse_generator_prescaler: PGPrescalerDivider::_4,
        max_count_value: MaxCount::_8191,
        io_default_mode: false,
        synchro_pin_polarity: false,
        acquisition_mode: false,
        max_count_interrupt: false,
        channel_ios: TscIOPin::Group2Io2 | TscIOPin::Group7Io3,
        shield_ios: TscIOPin::Group1Io3.into(),
        sampling_ios: TscIOPin::Group1Io2 | TscIOPin::Group2Io1 | TscIOPin::Group7Io2,
    };

    let mut g1: PinGroup<embassy_stm32::peripherals::TSC, G1> = PinGroup::new();
    g1.set_io2(context.PB13, PinType::Sample);
    g1.set_io3(context.PB14, PinType::Shield);

    let mut g2: PinGroup<embassy_stm32::peripherals::TSC, G2> = PinGroup::new();
    g2.set_io1(context.PB4, PinType::Sample);
    g2.set_io2(context.PB5, PinType::Channel);

    let mut g7: PinGroup<embassy_stm32::peripherals::TSC, G7> = PinGroup::new();
    g7.set_io2(context.PE3, PinType::Sample);
    g7.set_io3(context.PE4, PinType::Channel);

    let mut touch_controller = tsc::Tsc::new(
        context.TSC,
        Some(g1),
        Some(g2),
        None,
        None,
        None,
        None,
        Some(g7),
        None,
        config,
    );

    touch_controller.discharge_io(true);
    Timer::after_millis(1).await;

    touch_controller.start();

    let mut group_two_val = 0;
    let mut group_seven_val = 0;
    info!("Starting touch_controller interface");
    loop {
        touch_controller.poll_for_acquisition();
        touch_controller.discharge_io(true);
        Timer::after_millis(1).await;

        if touch_controller.group_get_status(Group::Two) == GroupStatus::Complete {
            group_two_val = touch_controller.group_get_value(Group::Two);
        }

        if touch_controller.group_get_status(Group::Seven) == GroupStatus::Complete {
            group_seven_val = touch_controller.group_get_value(Group::Seven);
        }

        info!(
            "Group Two value: {}, Group Seven value: {},",
            group_two_val, group_seven_val
        );

        touch_controller.start();
    }
}
