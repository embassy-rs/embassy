// Example of async TSC (Touch Sensing Controller) that lights an LED when touch is detected.
//
// This example demonstrates:
// 1. Configuring a single TSC channel pin
// 2. Using the async TSC interface
// 3. Waiting for acquisition completion using `pend_for_acquisition`
// 4. Reading touch values and controlling an LED based on the results
//
// Suggested physical setup on STM32L4R5ZI-P board:
// - Connect a 1000pF capacitor between pin PB4 (D25) and GND. This is your sampling capacitor.
// - Connect one end of a 1K resistor to pin PB5 (D21) and leave the other end loose.
//   The loose end will act as the touch sensor which will register your touch.
//
// The example uses two pins from Group 2 of the TSC:
// - PB4 (D25) as the sampling capacitor, TSC group 2 IO1
// - PB5 (D21) as the channel pin, TSC group 2 IO2
//
// The program continuously reads the touch sensor value:
// - It starts acquisition, waits for completion using `pend_for_acquisition`, and reads the value.
// - The LED (connected to PB14) is turned on when touch is detected (sensor value < SENSOR_THRESHOLD).
// - Touch values are logged to the console.
//
// Troubleshooting:
// - If touch is not detected, try adjusting the SENSOR_THRESHOLD value.
// - Experiment with different values for ct_pulse_high_length, ct_pulse_low_length,
//   pulse_generator_prescaler, max_count_value, and discharge_delay to optimize sensitivity.
//
// Note: Configuration values and sampling capacitor value have been determined experimentally.
// Optimal values may vary based on your specific hardware setup.

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::tsc::{self, *};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TSC => InterruptHandler<embassy_stm32::peripherals::TSC>;
});
const SENSOR_THRESHOLD: u16 = 25; // Adjust this value based on your setup

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let device_config = embassy_stm32::Config::default();
    let context = embassy_stm32::init(device_config);

    let mut pin_group: PinGroupWithRoles<peripherals::TSC, G2> = PinGroupWithRoles::default();
    // D25
    pin_group.set_io1::<tsc::pin_roles::Sample>(context.PB4);
    // D21
    let tsc_sensor = pin_group.set_io2::<tsc::pin_roles::Channel>(context.PB5);

    let pin_groups: PinGroups<peripherals::TSC> = PinGroups {
        g2: Some(pin_group.pin_group),
        ..Default::default()
    };

    let tsc_conf = Config {
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
    };

    let mut touch_controller = tsc::Tsc::new_async(context.TSC, pin_groups, tsc_conf, Irqs).unwrap();

    // Check if TSC is ready
    if touch_controller.get_state() != State::Ready {
        info!("TSC not ready!");
        return;
    }
    info!("TSC initialized successfully");

    let mut led = Output::new(context.PB14, Level::High, Speed::Low);

    let discharge_delay = 1; // ms

    info!("Starting touch_controller interface");
    loop {
        touch_controller.set_active_channels_mask(tsc_sensor.pin.into());
        touch_controller.start();
        touch_controller.pend_for_acquisition().await;
        touch_controller.discharge_io(true);
        Timer::after_millis(discharge_delay).await;

        let group_val = touch_controller.group_get_value(tsc_sensor.pin.group());
        info!("Touch value: {}", group_val);

        if group_val < SENSOR_THRESHOLD {
            led.set_high();
        } else {
            led.set_low();
        }

        Timer::after_millis(100).await;
    }
}
