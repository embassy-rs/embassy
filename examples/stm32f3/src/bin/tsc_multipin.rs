// Example of TSC (Touch Sensing Controller) using multiple pins from the same tsc-group.
//
// What is special about using multiple TSC pins as sensor channels from the same TSC group,
// is that only one TSC pin for each TSC group can be acquired and read at the time.
// To control which channel pins are acquired and read, we must write a mask before initiating an
// acquisition. To help manage and abstract all this business away, we can organize our channel
// pins into acquisition banks. Each acquisition bank can contain exactly one channel pin per TSC
// group and it will contain the relevant mask.
//
// This example demonstrates how to:
// 1. Configure multiple channel pins within a single TSC group
// 2. Use the set_active_channels_bank method to switch between sets of different channels (acquisition banks)
// 3. Read and interpret touch values from multiple channels in the same group
//
// Suggested physical setup on STM32F303ZE Nucleo board:
// - Connect a 1000pF capacitor between pin PA10 and GND. This is the sampling capacitor for TSC
//   group 4.
// - Connect one end of a 1K resistor to pin PA9 and leave the other end loose.
//   The loose end will act as a touch sensor.
//
// - Connect a 1000pF capacitor between pin PA7 and GND. This is the sampling capacitor for TSC
//   group 2.
// - Connect one end of another 1K resistor to pin PA6 and leave the other end loose.
//   The loose end will act as a touch sensor.
// - Connect one end of another 1K resistor to pin PA5 and leave the other end loose.
//   The loose end will act as a touch sensor.
//
// The example uses pins from two TSC groups.
// - PA10 as sampling capacitor, TSC group 4 IO2
// - PA9 as channel, TSC group 4 IO1
// - PA7 as sampling capacitor, TSC group 2 IO4
// - PA6 as channel, TSC group 2 IO3
// - PA5 as channel, TSC group 2 IO2
//
// The pins have been chosen to make it easy to simply add capacitors directly onto the board and
// connect one leg to GND, and to easily add resistors to the board with no special connectors,
// breadboards, special wires or soldering required. All you need is the capacitors and resistors.
//
// The program reads the designated channel pins and adjusts the LED blinking
// pattern based on which sensor(s) are touched:
// - No touch: LED off
// - one sensor touched: Slow blinking
// - two sensors touched: Fast blinking
// - three sensors touched: LED constantly on
//
// ## Troubleshooting:
//
// - If touch is not detected, try adjusting the SENSOR_THRESHOLD value (currently set to 20).
// - Experiment with different values for ct_pulse_high_length, ct_pulse_low_length, pulse_generator_prescaler, max_count_value, and discharge_delay to optimize sensitivity.
// - Be aware that for some boards there will be overlapping concerns between some pins, for
//  example UART connection for the programmer to the MCU and a TSC pin. No errors or warning will
//  be emitted if you try to use such a pin for TSC, but you will get strange sensor readings.
//
// Note: Configuration values and sampling capacitor values have been determined experimentally. Optimal values may vary based on your specific hardware setup. Refer to the official STM32 datasheet and user manuals for more information on pin configurations and TSC functionality.

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::tsc::{self, *};
use embassy_stm32::{mode, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

const SENSOR_THRESHOLD: u16 = 10;

async fn acquire_sensors(
    touch_controller: &mut Tsc<'static, peripherals::TSC, mode::Blocking>,
    tsc_acquisition_bank: &AcquisitionBank,
) {
    touch_controller.set_active_channels_bank(tsc_acquisition_bank);
    touch_controller.start();
    touch_controller.poll_for_acquisition();
    touch_controller.discharge_io(true);
    let discharge_delay = 5; // ms
    Timer::after_millis(discharge_delay).await;
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let device_config = embassy_stm32::Config::default();
    let context = embassy_stm32::init(device_config);

    // ---------- initial configuration of TSC ----------
    //
    let mut pin_group4: PinGroupWithRoles<peripherals::TSC, G4> = PinGroupWithRoles::default();
    // D68 on the STM32F303ZE nucleo-board
    pin_group4.set_io2::<tsc::pin_roles::Sample>(context.PA10);
    // D69 on the STM32F303ZE nucleo-board
    let tsc_sensor0 = pin_group4.set_io1(context.PA9);

    let mut pin_group2: PinGroupWithRoles<peripherals::TSC, G2> = PinGroupWithRoles::default();
    // D11 on the STM32F303ZE nucleo-board
    pin_group2.set_io4::<tsc::pin_roles::Sample>(context.PA7);
    // D12 on the STM32F303ZE nucleo-board
    let tsc_sensor1 = pin_group2.set_io3(context.PA6);
    // D13 on the STM32F303ZE nucleo-board
    let tsc_sensor2 = pin_group2.set_io2(context.PA5);

    let config = Config {
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

    let pin_groups: PinGroups<peripherals::TSC> = PinGroups {
        g4: Some(pin_group4.pin_group),
        g2: Some(pin_group2.pin_group),
        ..Default::default()
    };

    let mut touch_controller = tsc::Tsc::new_blocking(context.TSC, pin_groups, config).unwrap();

    // ---------- setting up acquisition banks ----------
    // sensor0 and sensor1 in this example belong to different TSC-groups,
    // therefore we can acquire and read them both in one go.
    let bank1 = touch_controller.create_acquisition_bank(AcquisitionBankPins {
        g4_pin: Some(tsc_sensor0),
        g2_pin: Some(tsc_sensor1),
        ..Default::default()
    });
    // `sensor1` and `sensor2` belongs to the same TSC-group, therefore we must make sure to
    // acquire them one at the time. Therefore, we organize them into different acquisition banks.
    let bank2 = touch_controller.create_acquisition_bank(AcquisitionBankPins {
        g2_pin: Some(tsc_sensor2),
        ..Default::default()
    });

    // Check if TSC is ready
    if touch_controller.get_state() != State::Ready {
        crate::panic!("TSC not ready!");
    }

    info!("TSC initialized successfully");

    // LED2 on the STM32F303ZE nucleo-board
    let mut led = Output::new(context.PB7, Level::High, Speed::Low);

    let mut led_state = false;

    loop {
        acquire_sensors(&mut touch_controller, &bank1).await;
        let readings1 = touch_controller.get_acquisition_bank_values(&bank1);
        acquire_sensors(&mut touch_controller, &bank2).await;
        let readings2 = touch_controller.get_acquisition_bank_values(&bank2);

        let mut touched_sensors_count = 0;
        for reading in readings1.iter() {
            info!("{}", reading);
            if reading.sensor_value < SENSOR_THRESHOLD {
                touched_sensors_count += 1;
            }
        }
        for reading in readings2.iter() {
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
