#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::pwm::{
    Config, Prescaler, SequenceConfig, SequenceLoad, SequencePwm, SingleSequenceMode, SingleSequencer,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// WS2812B LED light demonstration. Drives just one light.
// The following reference on WS2812B may be of use:
// https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf.
// This demo lights up a single LED in blue. It then proceeds
// to pulsate the LED rapidly.
//
// /!\ NOTE FOR nRF52840-DK users /!\
//
// If you're using the nRF52840-DK, the default "Vdd" power source
// will set the GPIO I/O voltage to 3.0v, using the onboard regulator.
// This can sometimes not be enough to drive the WS2812B signal if you
// are not using an external level shifter. If you set the board to "USB"
// power instead (and provide power via the "nRF USB" connector), the board
// will instead power the I/Os at 3.3v, which is often enough (but still
// out of official spec) for the WS2812Bs to work properly.

// In the following declarations, setting the high bit tells the PWM
// to reverse polarity, which is what the WS2812B expects.

const T1H: u16 = 0x8000 | 13; // Duty = 13/20 ticks (0.8us/1.25us) for a 1
const T0H: u16 = 0x8000 | 7; // Duty 7/20 ticks (0.4us/1.25us) for a 0
const RES: u16 = 0x8000;

// Provides data to a WS2812b (Neopixel) LED and makes it go blue. The data
// line is assumed to be P1_05.
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut config = Config::default();
    config.sequence_load = SequenceLoad::Common;
    config.prescaler = Prescaler::Div1;
    config.max_duty = 20; // 1.25us (1s / 16Mhz * 20)
    let mut pwm = unwrap!(SequencePwm::new_1ch(
        p.PWM0,
        Output::new(p.P1_05, Level::Low, OutputDrive::Standard).into(),
        config
    ));

    // Declare the bits of 24 bits in a buffer we'll be
    // mutating later.
    let mut seq_words = [
        T0H, T0H, T0H, T0H, T0H, T0H, T0H, T0H, // G
        T0H, T0H, T0H, T0H, T0H, T0H, T0H, T0H, // R
        T1H, T1H, T1H, T1H, T1H, T1H, T1H, T1H, // B
        RES,
    ];
    let mut seq_config = SequenceConfig::default();
    seq_config.end_delay = 799; // 50us (20 ticks * 40) - 1 tick because we've already got one RES;

    let mut color_bit = 16;
    let mut bit_value = T0H;

    loop {
        let sequences = SingleSequencer::new(&mut pwm, &seq_words, seq_config.clone());
        unwrap!(sequences.start(SingleSequenceMode::Times(1)));

        Timer::after_millis(50).await;

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

        drop(sequences);

        seq_words[color_bit] = bit_value;
    }
}
