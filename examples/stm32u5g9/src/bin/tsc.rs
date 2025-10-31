#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::tsc::{self, *};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TSC => InterruptHandler<embassy_stm32::peripherals::TSC>;
});

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
    };

    let mut g1: PinGroupWithRoles<peripherals::TSC, G1> = PinGroupWithRoles::default();
    g1.set_io2::<tsc::pin_roles::Sample>(context.PB13);
    g1.set_io3::<tsc::pin_roles::Shield>(context.PB14);

    let mut g2: PinGroupWithRoles<peripherals::TSC, G2> = PinGroupWithRoles::default();
    g2.set_io1::<tsc::pin_roles::Sample>(context.PB4);
    let sensor0 = g2.set_io2(context.PB5);

    let mut g7: PinGroupWithRoles<peripherals::TSC, G7> = PinGroupWithRoles::default();
    g7.set_io2::<tsc::pin_roles::Sample>(context.PE3);
    let sensor1 = g7.set_io3(context.PE4);

    let pin_groups: PinGroups<peripherals::TSC> = PinGroups {
        g1: Some(g1.pin_group),
        g2: Some(g2.pin_group),
        g7: Some(g7.pin_group),
        ..Default::default()
    };

    let mut touch_controller = tsc::Tsc::new_async(context.TSC, pin_groups, config, Irqs).unwrap();

    let acquisition_bank = touch_controller.create_acquisition_bank(AcquisitionBankPins {
        g2_pin: Some(sensor0),
        g7_pin: Some(sensor1),
        ..Default::default()
    });

    touch_controller.set_active_channels_bank(&acquisition_bank);

    info!("Starting touch_controller interface");
    loop {
        touch_controller.start();
        touch_controller.pend_for_acquisition().await;
        touch_controller.discharge_io(true);
        Timer::after_millis(1).await;

        let status = touch_controller.get_acquisition_bank_status(&acquisition_bank);

        if status.all_complete() {
            let read_values = touch_controller.get_acquisition_bank_values(&acquisition_bank);
            let group2_reading = read_values.get_group_reading(Group::Two).unwrap();
            let group7_reading = read_values.get_group_reading(Group::Seven).unwrap();
            info!("group 2 value: {}", group2_reading.sensor_value);
            info!("group 7 value: {}", group7_reading.sensor_value);
        }
    }
}
