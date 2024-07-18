//! this example demonstrates how to determine:
//! 1.) are we connected to usb power
//! 2.) and what is the voltage on vsys
//!
//! we will put the required pins into mutexes because on the Pi Pico W they will be used by the wifi chip, so we need to make
//! sure that we are not using them at the same time. On a board without a wifi chip, this is not necessary and the pins can be
//! used directly.

#![no_std]
#![no_main]

use core::borrow::BorrowMut;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config, InterruptHandler};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{ADC, PIN_24, PIN_25, PIN_29};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{with_timeout, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

pub struct PowerPins {
    vbus_pin: PIN_24, // vbus on a board without a wifi chip, on a board with a wifi chip, this will not work - see below. You will need some other pin and a voltage divider.
    pin_25: PIN_25,
    vsys_pin: PIN_29, // vsys
}

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

pub type PowerPinsType = Mutex<ThreadModeRawMutex, Option<PowerPins>>;
pub static POWER_PINS: PowerPinsType = Mutex::new(None);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // initialize the peripherals
    let p = embassy_rp::init(Default::default());

    // assign the power pins to the struct which we will put into the mutex
    let power_pins = PowerPins {
        vbus_pin: p.PIN_24,
        pin_25: p.PIN_25,
        vsys_pin: p.PIN_29,
    };
    // put the power pins into the mutex
    // inner scope is so that once the mutex is written to, the MutexGuard is dropped, thus the
    // Mutex is released
    {
        *(POWER_PINS.lock().await) = Some(power_pins);
    };

    // spawn the tasks
    spawner.spawn(dummy_wifi_task(spawner)).unwrap();
    spawner.spawn(check_usb_power(spawner)).unwrap();
    spawner.spawn(get_vsys_voltage(spawner, p.ADC)).unwrap();
}

// this task simulates the wifi chip, which will need the pins eventually
// here we just lock the mutex every 20s for 5s and do nothing besides that
#[embassy_executor::task]
async fn dummy_wifi_task(_spawner: Spawner) {
    // get the power pins from the mutex
    loop {
        // inner scope, so that after 5s have passed, the MutexGuard is dropped
        {
            info!("wifi locking power pins");
            let _power_pins_unlocked = POWER_PINS.lock().await;
            Timer::after_secs(5).await;
        }
        // do nothing, give other tasks a chance to run
        Timer::after_secs(20).await;
    }
}

#[embassy_executor::task]
async fn get_vsys_voltage(_spawner: Spawner, adc: ADC) {
    let mut adc_val = Adc::new(adc, Irqs, Config::default());
    loop {
        // inner scope, so that after we did what we came for, the MutexGuard is dropped
        {
            let mut power_pins_guard = POWER_PINS.lock().await;

            if let Some(ref mut pins) = *power_pins_guard {
                // we need to set pin 25 as an output
                let pin25_borrow = pins.pin_25.borrow_mut();
                let mut pin25_output = Output::new(pin25_borrow, Level::Low);

                // we need to set pin 29 as channel for the adc
                let vsys_pin_borrow = pins.vsys_pin.borrow_mut();
                let mut vsys_pin = Channel::new_pin(vsys_pin_borrow, Pull::None);

                // and we need the adc
                let adc = &mut adc_val;
                // read the adc

                // for reading the adc we need to set pin 25 to high. On a board without a wifi chip, this is not necessary.
                pin25_output.set_high();
                Timer::after_millis(50).await; // give the adc some time to settle

                // read the adc
                let level = adc.read(&mut vsys_pin).await.unwrap();
                info!("Pin 29 ADC: {}", level);
                let voltage = (level as f32) * 3.3 * 3.0 / 4_096.0; // reference voltage is 3.3V, there is a voltage divider by 3 and the adc is 12 bit
                info!("Pin 29 Voltage: {}", voltage);

                // set pin 25 to low again, see above: on a board without a wifi chip, this is not necessary
                // if we do not cycle through low, subsequent adc reads will be wrong
                pin25_output.set_low();
            }
        }
        // do nothing and give other tasks a chance to run
        Timer::after_secs(1).await;
    }
}

// this task will check if we are connected to usb power
// on a board without a wifi chip, this task will work with PIN_24
// on a board with a wifi chip, this task will not work with PIN_24.
// Either: Do not use this.
// Or: Wire from vbus to another gpio and use that. Bring down the voltage from 5V to the required 3.3V with a voltage divider made of two resistors.
// The resistors must be one exactly double the resistance of the other to get 3.3V between them, i.e. use 20k and 10k resistors.
#[embassy_executor::task]
async fn check_usb_power(_spawner: Spawner) {
    // get the power pins from the mutex
    loop {
        // inner scope, so that after we did what we came for, the MutexGuard is dropped
        {
            let mut power_pins_guard = POWER_PINS.lock().await;

            if let Some(ref mut pins) = *power_pins_guard {
                // we need to set pin 24 as an input
                // on a board without a wifi chip, this will work
                // on a board with a wifi chip, this will not work, see above
                let vbus_pin_borrow = pins.vbus_pin.borrow_mut();
                let mut vbus_pin_input = Input::new(vbus_pin_borrow, Pull::None);

                // and then we wait for an edge, in this demo we limit the time we wait for an edge to 20s, because we are still locking the mutex
                let result = with_timeout(Duration::from_secs(20), vbus_pin_input.wait_for_any_edge()).await;
                match result {
                    Ok(_) => {
                        info!(
                            "Edge on usb power, now: {:?}",
                            Debug2Format(&vbus_pin_input.get_level())
                        );
                    }
                    Err(_) => {
                        info!("timed out without detecting an edge on usb power");
                    }
                }
            }
        }
        // do nothing and give other tasks a chance to run
        Timer::after_secs(1).await;
    }
}
