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
    Config, Prescaler, Sequence, SequenceConfig, SequenceLoad, SequenceMode, SequencePwm,
};
use embassy_nrf::Peripherals;

// WS2812B LED light demonstration. Drives just one light.
// The following reference on WS2812B may be of use:
// https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf.
// This demo lights up a single LED in blue. It then proceeds
// to pulsate the LED rapidly.

// In the following declarations, setting the high bit tells the PWM
// to reverse polarity, which is what the WS2812B expects.

const T1H: u16 = 0x8000 | 13; // Duty = 13/20 ticks (0.8us/1.25us) for a 1
const T0H: u16 = 0x8000 | 7; // Duty 7/20 ticks (0.4us/1.25us) for a 0
const RES: u16 = 0x8000;

// Provides data to a WS2812b (Neopixel) LED and makes it go blue. The data
// line is assumed to be P1_05.
#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut config = Config::default();
    config.sequence_load = SequenceLoad::Common;
    config.prescaler = Prescaler::Div1;
    config.max_duty = 20; // 1.25us (1s / 16Mhz * 20)
    let mut pwm = unwrap!(SequencePwm::new(
        p.PWM0, p.P1_05, NoPin, NoPin, NoPin, config,
    ));

    // Declare the bits of 24 bits
    let color_seq = Sequence::new(
        [
            T0H, T0H, T0H, T0H, T0H, T0H, T0H, T0H, // G
            T0H, T0H, T0H, T0H, T0H, T0H, T0H, T0H, // R
            T1H, T1H, T1H, T1H, T1H, T1H, T1H, T1H, // B
        ],
        SequenceConfig::default(),
    );

    let mut reset_seq_config = SequenceConfig::default();
    reset_seq_config.end_delay = 799; // 50us (20 ticks * 40) - 1 tick because we've already got one RES;
    let reset_seq = Sequence::new([RES], reset_seq_config);

    unwrap!(pwm.start(color_seq, reset_seq, SequenceMode::Times(2)));

    Timer::after(Duration::from_millis(1000)).await;

    let mut color_bit = 16;
    let mut bit_value = T0H;

    loop {
        if let (Some(mut color_seq), Some(reset_seq)) = pwm.stop() {
            color_seq.words[color_bit] = bit_value;
            unwrap!(pwm.start(color_seq, reset_seq, SequenceMode::Times(2)));
        }

        Timer::after(Duration::from_millis(50)).await;

        if bit_value == T0H {
            if color_bit == 20 {
                bit_value = T1H;
            } else {
                color_bit += 1;
            }
        } else {
            if color_bit == 16 {
                bit_value = T0H;
            } else {
                color_bit -= 1;
            }
        }
    }
}
