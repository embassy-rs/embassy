//! This example measures from the TMP6131 thermistor on the LP-MSPM0G3507 board.
//!
//! J9 must be set to connect the thermistor to PB24 for this example to work.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::adc::{self, Adc, AdcChannel, Conversion};
use embassy_mspm0::{Config, bind_interrupts, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

bind_interrupts!(struct Irqs {
    ADC0 => adc::InterruptHandler<peripherals::ADC0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");
    let mut p = embassy_mspm0::init(Config::default());

    let mut adc = Adc::new_async(p.ADC0, Irqs, Default::default());
    let mut thermistor_pin = p.PB24.reborrow_adc();
    let max_count = adc.resolution().max_count();

    loop {
        let r = adc.blocking_read(&mut thermistor_pin, Conversion::default());
        let (temp, temp_decimal) = tmp61_get_temp(max_count, r);

        info!(
            "Temperature at thermistor: {}.{}°C (blocking read, raw reading: {})",
            temp, temp_decimal, r
        );

        Timer::after_millis(1000).await;

        let r = adc.irq_read(&mut thermistor_pin, Conversion::default()).await;
        let (temp, temp_decimal) = tmp61_get_temp(max_count, r);

        info!(
            "Temperature at thermistor: {}.{}°C (irq read, raw reading: {})",
            temp, temp_decimal, r
        );

        Timer::after_millis(1000).await;
    }
}

/// R56 on LP-MSPM0G3507
const R_BIAS: u32 = 10000;
const VDD_MV: u32 = 3300;

/// Table of resistances for -40°C to 125°C.
///
/// Index 0 is -40°C. The temperature increases +5°C for each index.
///
/// This table comes from https://dr-download.ti.com/software-development/support-software/MD-cYRSrw9Du8/01.00.00.0F/sboc595f.zip.
const TABLE: [u16; 34] = [
    6543, 6761, 6987, 7220, 7460, 7707, 7962, 8225, 8495, 8772, 9057, 9350, 9651, 9959, 10275, 10599, 10931, 11270,
    11618, 11975, 12339, 12712, 13093, 13483, 13881, 14288, 14704, 15129, 15563, 16006, 16459, 16921, 17392, 17874,
];

fn tmp61_get_temp(max_count: u32, reading: u16) -> (i8, u8) {
    let v_temp = (reading as u32 * VDD_MV) / max_count;
    let r_temp = (R_BIAS * v_temp) / (VDD_MV - v_temp);

    let index = TABLE
        .partition_point(|&r| (r as u32) < r_temp)
        // Temperatures outside of the table should saturate.
        .clamp(1, TABLE.len() - 1);

    let r0 = TABLE[index - 1] as u32;
    let r1 = TABLE[index] as u32;

    let t0 = -40 + (index as i32 - 1) * 5;

    // Interpolate the hundredths of °C to the next temperature in table.
    let frac = ((r_temp - r0) * 500) / (r1 - r0);

    // Compute the whole temperature in hundredths of °C since interpolation may add up to 4.999°C recurring.
    let temp_whole = (t0 as i32 * 100) + frac as i32;

    ((temp_whole / 100) as i8, (temp_whole % 100) as u8)
}
