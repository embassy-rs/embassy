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
// Suggested physical setup on STM32L073RZ Nucleo board:
// - Connect a 1000pF capacitor between pin PA0 (label A0) and GND. This is the sampling capacitor for TSC
//   group 1.
// - Connect one end of a 1K resistor to pin PA1 (label A1) and leave the other end loose.
//   The loose end will act as a touch sensor.
//
// - Connect a 1000pF capacitor between pin PB3 (label D3) and GND. This is the sampling capacitor for TSC
//   group 5.
// - Connect one end of another 1K resistor to pin PB4 and leave the other end loose.
//   The loose end will act as a touch sensor.
// - Connect one end of another 1K resistor to pin PB6 and leave the other end loose.
//   The loose end will act as a touch sensor.
//
// The example uses pins from two TSC groups.
// - PA0 as sampling capacitor, TSC group 1 IO1 (label A0)
// - PA1 as channel, TSC group 1 IO2 (label A1)
// - PB3 as sampling capacitor, TSC group 5 IO1 (label D3)
// - PB4 as channel, TSC group 5 IO2 (label D10)
// - PB6 as channel, TSC group 5 IO3 (label D5)
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
// Troubleshooting:
// - If touch is not detected, try adjusting the SENSOR_THRESHOLD value.
// - Experiment with different values for ct_pulse_high_length, ct_pulse_low_length,
//   pulse_generator_prescaler, max_count_value, and discharge_delay to optimize sensitivity.
//
// Note: Configuration values and sampling capacitor value have been determined experimentally.
// Optimal values may vary based on your specific hardware setup.
// Pins have been chosen for their convenient locations on the STM32L073RZ board. Refer to the
// official relevant STM32 datasheets and nucleo-board user manuals to find suitable
// alternative pins.
//
// Beware for STM32L073RZ nucleo-board, that PA2 and PA3 is used for the uart connection to
// the programmer chip. If you try to use these two pins for TSC, you will get strange
// readings, unless you somehow reconfigure/re-wire your nucleo-board.
// No errors or warnings will be emitted, they will just silently not work as expected.
// (see nucleo user manual UM1724, Rev 14, page 25)

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

const SENSOR_THRESHOLD: u16 = 35;

async fn acquire_sensors(
    touch_controller: &mut Tsc<'static, peripherals::TSC, mode::Async>,
    tsc_acquisition_bank: &AcquisitionBank,
) {
    touch_controller.set_active_channels_bank(tsc_acquisition_bank);
    touch_controller.start();
    touch_controller.pend_for_acquisition().await;
    touch_controller.discharge_io(true);
    let discharge_delay = 5; // ms
    Timer::after_millis(discharge_delay).await;
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let device_config = embassy_stm32::Config::default();
    let context = embassy_stm32::init(device_config);

    // ---------- initial configuration of TSC ----------
    let mut pin_group1: PinGroupWithRoles<peripherals::TSC, G1> = PinGroupWithRoles::default();
    pin_group1.set_io1::<tsc::pin_roles::Sample>(context.PA0);
    let tsc_sensor0 = pin_group1.set_io2(context.PA1);

    let mut pin_group5: PinGroupWithRoles<peripherals::TSC, G5> = PinGroupWithRoles::default();
    pin_group5.set_io1::<tsc::pin_roles::Sample>(context.PB3);
    let tsc_sensor1 = pin_group5.set_io2(context.PB4);
    let tsc_sensor2 = pin_group5.set_io3(context.PB6);

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
        g1: Some(pin_group1.pin_group),
        g5: Some(pin_group5.pin_group),
        ..Default::default()
    };

    let mut touch_controller = tsc::Tsc::new_async(context.TSC, pin_groups, config, Irqs).unwrap();

    // ---------- setting up acquisition banks ----------
    // sensor0 and sensor1 in this example belong to different TSC-groups,
    // therefore we can acquire and read them both in one go.
    let bank1 = touch_controller.create_acquisition_bank(AcquisitionBankPins {
        g1_pin: Some(tsc_sensor0),
        g5_pin: Some(tsc_sensor1),
        ..Default::default()
    });
    // `sensor1` and `sensor2` belongs to the same TSC-group, therefore we must make sure to
    // acquire them one at the time. Therefore, we organize them into different acquisition banks.
    let bank2 = touch_controller.create_acquisition_bank(AcquisitionBankPins {
        g5_pin: Some(tsc_sensor2),
        ..Default::default()
    });

    // Check if TSC is ready
    if touch_controller.get_state() != State::Ready {
        crate::panic!("TSC not ready!");
    }

    info!("TSC initialized successfully");

    // LED2 on the STM32L073RZ nucleo-board (PA5)
    let mut led = Output::new(context.PA5, Level::High, Speed::Low);

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
