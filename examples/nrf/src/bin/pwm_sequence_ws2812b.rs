#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::NoPin;
use embassy_nrf::pwm::{
    Config, Prescaler, SequenceConfig, SequenceLoad, SequenceMode, SequencePwm,
};
use embassy_nrf::Peripherals;

// WS2812B LED light demonstration. Drives just one light.
// The following reference on WS2812B may be of use:
// https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf

// In the following declarations, setting the high bit tells the PWM
// to reverse polarity, which is what the WS2812B expects.

const T1H: u16 = 0x8000 | 13; // Duty = 13/20 ticks (0.8us/1.25us) for a 1
const T0H: u16 = 0x8000 | 7; // Duty 7/20 ticks (0.4us/1.25us) for a 0
const RES: u16 = 0x8000;

// Provides data to a WS2812b (Neopixel) LED and makes it go blue. The data
// line is assumed to be P1_05.
#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    // Declare the bits of 24 bits
    let mut blue_seq: [u16; 8 * 3] = [
        T0H, T0H, T0H, T0H, T0H, T0H, T0H, T0H, // G
        T0H, T0H, T0H, T0H, T0H, T0H, T0H, T0H, // R
        T1H, T1H, T1H, T1H, T1H, T1H, T1H, T1H, // B
    ];
    let reset_seq = [RES; 1];

    let mut config = Config::default();
    config.sequence_load = SequenceLoad::Common;
    config.prescaler = Prescaler::Div1;
    config.max_duty = 20; // 1.25us (1s / 16Mhz * 20)
    let mut pwm = unwrap!(SequencePwm::new(
        p.PWM0, p.P1_05, NoPin, NoPin, NoPin, config,
    ));

    let blue_seq_config = SequenceConfig::default();
    let mut reset_seq_config = SequenceConfig::default();
    reset_seq_config.end_delay = 799; // 50us (20 ticks * 40) - 1 tick because we've already got one RES
    unwrap!(pwm.start(
        &blue_seq,
        blue_seq_config,
        Some(&reset_seq),
        Some(reset_seq_config),
        SequenceMode::Times(2)
    ));

    Timer::after(Duration::from_millis(20000)).await;
    info!("Program stopped");
}
