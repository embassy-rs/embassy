// Example of async TSC (Touch Sensing Controller) that lights an LED when touch is detected.
//
// Suggested physical setup on STM32L073RZ Nucleo board:
// - Connect a 1000pF capacitor between pin A0 and GND. This is your sampling capacitor.
// - Connect one end of a 1K resistor to pin A1 and leave the other end loose.
//   The loose end will act as touch sensor which will register your touch.
//
// Troubleshooting the setup:
// - If no touch seems to be registered, then try to disconnect the sampling capacitor from GND momentarily,
//   now the led should light up. Next try using a different value for the sampling capacitor.
//   Also experiment with increasing the values for `ct_pulse_high_length`, `ct_pulse_low_length`, `pulse_generator_prescaler`, `max_count_value` and `discharge_delay`.
//
// All configuration values and sampling capacitor value have been determined experimentally.
// Suitable configuration and discharge delay values are highly dependent on the value of the sample capacitor. For example, a shorter discharge delay can be used with smaller capacitor values.
//
#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::bind_interrupts;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::tsc::{self, *};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TSC => InterruptHandler<embassy_stm32::peripherals::TSC>;
});

#[cortex_m_rt::exception]
unsafe fn HardFault(_: &cortex_m_rt::ExceptionFrame) -> ! {
    cortex_m::peripheral::SCB::sys_reset();
}

/// This example is written for the nucleo-stm32l073rz, with a stm32l073rz chip.
///
/// Make sure you check/update the following (whether you use the L073RZ or another board):
///
/// * [ ] Update .cargo/config.toml with the correct `probe-rs run --chip STM32L073RZTx`chip name.
/// * [ ] Update Cargo.toml to have the correct `embassy-stm32` feature, for L073RZ it should be `stm32l073rz`.
/// * [ ] If your board has a special clock or power configuration, make sure that it is
///       set up appropriately.
/// * [ ] If your board has different pin mapping, update any pin numbers or peripherals
///       to match your schematic
///
/// If you are unsure, please drop by the Embassy Matrix chat for support, and let us know:
///
/// * Which example you are trying to run
/// * Which chip and board you are using
///
/// Embassy Chat: https://matrix.to/#/#embassy-rs:matrix.org
#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let device_config = embassy_stm32::Config::default();
    let context = embassy_stm32::init(device_config);

    let config = tsc::Config {
        ct_pulse_high_length: ChargeTransferPulseCycle::_4,
        ct_pulse_low_length: ChargeTransferPulseCycle::_4,
        spread_spectrum: false,
        spread_spectrum_deviation: SSDeviation::new(2).unwrap(),
        spread_spectrum_prescaler: false,
        pulse_generator_prescaler: PGPrescalerDivider::_16,
        max_count_value: MaxCount::_255,
        io_default_mode: false,
        synchro_pin_polarity: false,
        acquisition_mode: false,
        max_count_interrupt: false,
        channel_ios: TscIOPin::Group1Io1.into(),
        shield_ios: 0, // no shield
        sampling_ios: TscIOPin::Group1Io2.into(),
    };

    let mut g1: PinGroup<embassy_stm32::peripherals::TSC, G1> = PinGroup::new();
    g1.set_io1(context.PA0, PinType::Sample);
    g1.set_io2(context.PA1, PinType::Channel);

    let mut touch_controller = tsc::Tsc::new_async(
        context.TSC,
        Some(g1),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        config,
        Irqs,
    );

    // Check if TSC is ready
    if touch_controller.get_state() != State::Ready {
        info!("TSC not ready!");
        loop {} // Halt execution
    }
    info!("TSC initialized successfully");

    // LED2 on the STM32L073RZ nucleo-board (PA5)
    let mut led = Output::new(context.PA5, Level::High, Speed::Low);

    // smaller sample capacitor discharge faster and can be used with shorter delay.
    let discharge_delay = 5; // ms

    info!("Starting touch_controller interface");
    loop {
        touch_controller.start();
        touch_controller.pend_for_acquisition().await;
        touch_controller.discharge_io(true);
        Timer::after_millis(discharge_delay).await;

        let grp1_status = touch_controller.group_get_status(Group::One);
        match grp1_status {
            GroupStatus::Complete => {
                let group_one_val = touch_controller.group_get_value(Group::One);
                info!("{}", group_one_val);
                led.set_high();
            }
            GroupStatus::Ongoing => led.set_low(),
        }
    }
}
