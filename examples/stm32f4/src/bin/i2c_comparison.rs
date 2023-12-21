#![no_std]
#![no_main]

// Example originally designed for stm32f411ceu6 with three A1454 hall effect sensors, connected to I2C1, 2 and 3
// on the pins referenced in the peripheral definitions.
// Pins and DMA peripherals changed to compile for stm32f429zi, to work with the CI.
// MUST be compiled in release mode to see actual performance, otherwise the async transactions take 2x
// as long to complete as the blocking ones!

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::I2c;
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_time::Instant;
use futures::future::try_join3;
use {defmt_rtt as _, panic_probe as _};

const ADDRESS: u8 = 96;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
    I2C3_EV => i2c::EventInterruptHandler<peripherals::I2C3>;
    I2C3_ER => i2c::ErrorInterruptHandler<peripherals::I2C3>;
});

/// Convert 12-bit signed integer within a 4 byte long buffer into 16-bit signed integer.
fn a1454_buf_to_i16(buffer: &[u8; 4]) -> i16 {
    let lower = buffer[3];
    let mut upper = buffer[2];
    // Fill in additional 1s if the 12 bit number is negative.
    if (upper & 0b00001000) == 0b0001000 {
        upper = upper | 0b11110000;
    }

    let mut sensor_value_raw: u16 = lower.into();
    sensor_value_raw |= (upper as u16) << 8;
    let sensor_value: u16 = sensor_value_raw.into();
    let sensor_value = sensor_value as i16;
    sensor_value
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Setting up peripherals.");
    let p = embassy_stm32::init(Default::default());

    let mut i2c1 = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB7,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH0,
        Hertz(100_000),
        Default::default(),
    );

    let mut i2c2 = I2c::new(
        p.I2C2,
        p.PB10,
        p.PB11,
        Irqs,
        p.DMA1_CH7,
        p.DMA1_CH3,
        Hertz(100_000),
        Default::default(),
    );

    let mut i2c3 = I2c::new(
        p.I2C3,
        p.PA8,
        p.PC9,
        Irqs,
        p.DMA1_CH4,
        p.DMA1_CH2,
        Hertz(100_000),
        Default::default(),
    );

    let a1454_read_sensor_command = [0x1F];
    let mut i2c1_buffer: [u8; 4] = [0, 0, 0, 0];
    let mut i2c2_buffer: [u8; 4] = [0, 0, 0, 0];
    let mut i2c3_buffer: [u8; 4] = [0, 0, 0, 0];
    loop {
        // Blocking reads one after the other. Completes in about 2000us.
        let blocking_read_start_us = Instant::now().as_micros();
        match i2c1.blocking_write_read(ADDRESS, &a1454_read_sensor_command, &mut i2c1_buffer) {
            Ok(()) => {}
            Err(e) => error!("I2C Error: {:?}", e),
        }
        match i2c2.blocking_write_read(ADDRESS, &a1454_read_sensor_command, &mut i2c2_buffer) {
            Ok(()) => {}
            Err(e) => error!("I2C Error: {:?}", e),
        }
        match i2c3.blocking_write_read(ADDRESS, &a1454_read_sensor_command, &mut i2c3_buffer) {
            Ok(()) => {}
            Err(e) => error!("I2C Error: {:?}", e),
        }
        let blocking_read_total_us = Instant::now().as_micros() - blocking_read_start_us;
        info!(
            "Blocking reads completed in {}us: i2c1: {} i2c2: {} i2c3: {}",
            blocking_read_total_us,
            a1454_buf_to_i16(&i2c1_buffer),
            a1454_buf_to_i16(&i2c2_buffer),
            a1454_buf_to_i16(&i2c3_buffer)
        );

        // Async reads overlapping. Completes in about 1000us.
        let async_read_start_us = Instant::now().as_micros();

        let i2c1_result = i2c1.write_read(ADDRESS, &a1454_read_sensor_command, &mut i2c1_buffer);
        let i2c2_result = i2c2.write_read(ADDRESS, &a1454_read_sensor_command, &mut i2c2_buffer);
        let i2c3_result = i2c3.write_read(ADDRESS, &a1454_read_sensor_command, &mut i2c3_buffer);

        // Wait for all three transactions to finish, or any one of them to fail.
        match try_join3(i2c1_result, i2c2_result, i2c3_result).await {
            Ok(_) => {
                let async_read_total_us = Instant::now().as_micros() - async_read_start_us;
                info!(
                    "Async reads completed in {}us: i2c1: {} i2c2: {} i2c3: {}",
                    async_read_total_us,
                    a1454_buf_to_i16(&i2c1_buffer),
                    a1454_buf_to_i16(&i2c2_buffer),
                    a1454_buf_to_i16(&i2c3_buffer)
                );
            }
            Err(e) => error!("I2C Error during async write-read: {}", e),
        };
    }
}
