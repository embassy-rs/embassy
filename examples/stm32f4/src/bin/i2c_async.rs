#![no_std]
#![no_main]

// Example originally designed for stm32f411ceu6 reading an A1454 hall effect sensor on I2C1
// DMA peripherals changed to compile for stm32f429zi, for the CI.

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::I2c;
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use {defmt_rtt as _, panic_probe as _};

const ADDRESS: u8 = 96;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

/// This example is written for the nucleo-stm32f429zi, with a stm32f429zi chip.
///
/// If you are using a different board or chip, make sure you update the following:
///
/// * [ ] Update .cargo/config.toml with the correct `probe-rs run --chip STM32F429ZITx`chip name.
/// * [ ] Update Cargo.toml to have the correct `embassy-stm32` feature, it is
///       currently `stm32f429zi`.
/// * [ ] If your board has a special clock or power configuration, make sure that it is
///       set up appropriately.
/// * [ ] If your board has different pin mapping, update any pin numbers or peripherals
///       to match your schematic
///
/// If you are unsure, please drop by the Embassy Matrix chat for support, and let us know:
///
/// * Which example you are trying to run
/// * Which chip and board you are using
///
/// Embassy Chat: https://matrix.to/#/#embassy-rs:matrix.org
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello world!");
    let p = embassy_stm32::init(Default::default());

    let mut i2c = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB7,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH0,
        Hertz(100_000),
        Default::default(),
    );

    loop {
        let a1454_read_sensor_command = [0x1F];
        let mut sensor_data_buffer: [u8; 4] = [0, 0, 0, 0];

        match i2c
            .write_read(ADDRESS, &a1454_read_sensor_command, &mut sensor_data_buffer)
            .await
        {
            Ok(()) => {
                // Convert 12-bit signed integer into 16-bit signed integer.
                // Is the 12 bit number negative?
                if (sensor_data_buffer[2] & 0b00001000) == 0b0001000 {
                    sensor_data_buffer[2] = sensor_data_buffer[2] | 0b11110000;
                }

                let mut sensor_value_raw: u16 = sensor_data_buffer[3].into();
                sensor_value_raw |= (sensor_data_buffer[2] as u16) << 8;
                let sensor_value: u16 = sensor_value_raw.into();
                let sensor_value = sensor_value as i16;
                info!("Data: {}", sensor_value);
            }
            Err(e) => error!("I2C Error during read: {:?}", e),
        }
    }
}
