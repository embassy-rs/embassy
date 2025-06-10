// # Example of TSC (Touch Sensing Controller) using multiple pins from the same TSC group
//
// This example demonstrates how to use the Touch Sensing Controller (TSC) with multiple pins, including pins from the same TSC group, on an STM32L4R5ZI-P board.
//
// ## Key Concepts
//
// - Only one TSC pin for each TSC group can be acquired and read at a time.
// - To control which channel pins are acquired and read, we must write a mask before initiating an acquisition.
// - We organize channel pins into acquisition banks to manage this process efficiently.
// - Each acquisition bank can contain exactly one channel pin per TSC group and will contain the relevant mask.
//
// ## This example demonstrates how to:
//
// 1. Configure multiple channel pins within a single TSC group
// 2. Use the set_active_channels_bank method to switch between sets of different channels (acquisition banks)
// 3. Read and interpret touch values from multiple channels in the same group
//
// ## Suggested physical setup on STM32L4R5ZI-P board:
//
// - Connect a 1000pF capacitor between pin PB12 (D19) and GND. This is the sampling capacitor for TSC group 1.
// - Connect one end of a 1K resistor to pin PB13 (D18) and leave the other end loose. This will act as a touch sensor.
// - Connect a 1000pF capacitor between pin PB4 (D25) and GND. This is the sampling capacitor for TSC group 2.
// - Connect one end of a 1K resistor to pin PB5 (D22) and leave the other end loose. This will act as a touch sensor.
// - Connect one end of another 1K resistor to pin PB6 (D71) and leave the other end loose. This will act as a touch sensor.
//
// ## Pin Configuration:
//
// The example uses pins from two TSC groups:
//
// - Group 1:
//   - PB12 (D19) as sampling capacitor (TSC group 1 IO1)
//   - PB13 (D18) as channel (TSC group 1 IO2)
// - Group 2:
//   - PB4 (D25) as sampling capacitor (TSC group 2 IO1)
//   - PB5 (D22) as channel (TSC group 2 IO2)
//   - PB6 (D71) as channel (TSC group 2 IO3)
//
// The pins have been chosen for their convenient locations on the STM32L4R5ZI-P board, making it easy to add capacitors and resistors directly to the board without special connectors, breadboards, or soldering.
//
// ## Program Behavior:
//
// The program reads the designated channel pins and adjusts the LED (connected to PB14) blinking pattern based on which sensor(s) are touched:
//
// - No touch: LED off
// - One sensor touched: Slow blinking
// - Two sensors touched: Fast blinking
// - Three sensors touched: LED constantly on
//
// ## Troubleshooting:
//
// - If touch is not detected, try adjusting the SENSOR_THRESHOLD value (currently set to 20).
// - Experiment with different values for ct_pulse_high_length, ct_pulse_low_length, pulse_generator_prescaler, max_count_value, and discharge_delay to optimize sensitivity.
// - Be aware that for some boards there will be overlapping concerns between some pins, for
//  example UART connection for the programmer to the MCU and a TSC pin. No errors or warning will
//  be emitted if you try to use such a pin for TSC, but you will get strange sensor readings.
//
// Note: Configuration values and sampling capacitor values have been determined experimentally. Optimal values may vary based on your specific hardware setup. Refer to the official STM32L4R5ZI-P datasheet and user manuals for more information on pin configurations and TSC functionality.

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::tsc::{self, *};
use embassy_stm32::{bind_interrupts, mode, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TSC => InterruptHandler<embassy_stm32::peripherals::TSC>;
});

const SENSOR_THRESHOLD: u16 = 20;

async fn acquire_sensors(
    touch_controller: &mut Tsc<'static, peripherals::TSC, mode::Async>,
    tsc_acquisition_bank: &AcquisitionBank,
) {
    touch_controller.set_active_channels_bank(tsc_acquisition_bank);
    touch_controller.start();
    touch_controller.pend_for_acquisition().await;
    touch_controller.discharge_io(true);
    let discharge_delay = 1; // ms
    Timer::after_millis(discharge_delay).await;
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let device_config = embassy_stm32::Config::default();
    let context = embassy_stm32::init(device_config);

    // ---------- initial configuration of TSC ----------
    let mut g1: PinGroupWithRoles<peripherals::TSC, G1> = PinGroupWithRoles::default();
    g1.set_io1::<tsc::pin_roles::Sample>(context.PB12);
    let sensor0 = g1.set_io2::<tsc::pin_roles::Channel>(context.PB13);

    let mut g2: PinGroupWithRoles<peripherals::TSC, G2> = PinGroupWithRoles::default();
    g2.set_io1::<tsc::pin_roles::Sample>(context.PB4);
    let sensor1 = g2.set_io2(context.PB5);
    let sensor2 = g2.set_io3(context.PB6);

    let config = tsc::Config {
        ct_pulse_high_length: ChargeTransferPulseCycle::_16,
        ct_pulse_low_length: ChargeTransferPulseCycle::_16,
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

    let pin_groups: PinGroups<peripherals::TSC> = PinGroups {
        g1: Some(g1.pin_group),
        g2: Some(g2.pin_group),
        ..Default::default()
    };

    let mut touch_controller = tsc::Tsc::new_async(context.TSC, pin_groups, config, Irqs).unwrap();

    // ---------- setting up acquisition banks ----------
    // sensor0 and sensor1 belong to different TSC-groups, therefore we can acquire and
    // read them both in one go.
    let bank1 = touch_controller.create_acquisition_bank(AcquisitionBankPins {
        g1_pin: Some(sensor0),
        g2_pin: Some(sensor1),
        ..Default::default()
    });
    // `sensor1` and `sensor2` belongs to the same TSC-group, therefore we must make sure to
    // acquire them one at the time. We do this by organizing them into different acquisition banks.
    let bank2 = touch_controller.create_acquisition_bank(AcquisitionBankPins {
        g2_pin: Some(sensor2),
        ..Default::default()
    });

    // Check if TSC is ready
    if touch_controller.get_state() != State::Ready {
        crate::panic!("TSC not ready!");
    }

    info!("TSC initialized successfully");

    let mut led = Output::new(context.PB14, Level::High, Speed::Low);

    let mut led_state = false;

    loop {
        acquire_sensors(&mut touch_controller, &bank1).await;
        let readings1 = touch_controller.get_acquisition_bank_values(&bank1);
        acquire_sensors(&mut touch_controller, &bank2).await;
        let readings2 = touch_controller.get_acquisition_bank_values(&bank2);

        let mut touched_sensors_count = 0;
        for reading in readings1.iter().chain(readings2.iter()) {
            info!("{}", reading);
            if reading.sensor_value < SENSOR_THRESHOLD {
                touched_sensors_count += 1;
            }
        }

        match touched_sensors_count {
            0 => {
                // No sensors touched, turn off the LED
                led.set_low();
                led_state = false;
            }
            1 => {
                // One sensor touched, blink slowly
                led_state = !led_state;
                if led_state {
                    led.set_high();
                } else {
                    led.set_low();
                }
                Timer::after_millis(200).await;
            }
            2 => {
                // Two sensors touched, blink faster
                led_state = !led_state;
                if led_state {
                    led.set_high();
                } else {
                    led.set_low();
                }
                Timer::after_millis(50).await;
            }
            3 => {
                // All three sensors touched, LED constantly on
                led.set_high();
                led_state = true;
            }
            _ => crate::unreachable!(), // This case should never occur with 3 sensors
        }
    }
}
