#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::i2c::{self, I2c, Master};
use embassy_stm32::peripherals::I2C1;
use embassy_stm32::{Config, bind_interrupts, time, usb};
/*
 * Board NUCLEO-F302R8 with STM32F302R8
 * I2C1_SCL: PB8
 * I2C1_SDA: PB9
 *
 * MAX30100_I2C_WRITE_ADDR: 0xAE
 * MAX30100_I2C_READ_ADDR: 0xAF
 *
 *
 */

const DEVICE_ADDRESS: u8 = 0x57; // 7-bit I2C Address (Confirmed)
const MODE_CONF_REG_ADDR: u8 = 0x09; // CORRECTED Mode Config Register Address
const LED_CONF_REG_ADDR: u8 = 0x0A; // SpO2 Configuration Register (used to set LED pulse width/rate)
const IR_LED_CONF_REG_ADDR: u8 = 0x0C; // IR LED Current Register
const RED_LED_CONF_REG_ADDR: u8 = 0x0D; // Red LED Current Register
const HR_ONLY_MODE: u8 = 1 << 1;
const RESET_COMMAND: u8 = 0b0100_0000;
// const FIFO_WR_PTR_ADDR: u8 = 0x04;
// const FIFO_RD_PTR_ADDR: u8 = 0x06;
const FIFO_DATA_ADDR: u8 = 0x07;
const IR_SAMPLE_SIZE: usize = 6;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    bind_interrupts!(struct Irqs {
        I2C1_EV => i2c::EventInterruptHandler<I2C1>;
        I2C1_ER => i2c::ErrorInterruptHandler<I2C1>;
    });

    let mut i2c_myconfig = i2c::Config::default();
    i2c_myconfig.scl_pullup = true;
    i2c_myconfig.sda_pullup = true;
    let mut i2c_bus = i2c::I2c::new(p.I2C1, p.PB8, p.PB9, Irqs, p.DMA1_CH6, p.DMA1_CH7, i2c_myconfig);

    // let write_buffer = [PART_ID_REG_ADDR]; // The register address we want to read

    // --- Trigger the Reset ---
    i2c_bus
        .write(DEVICE_ADDRESS, &[MODE_CONF_REG_ADDR, RESET_COMMAND])
        .await
        .ok();
    Timer::after_millis(50).await; // Wait for reset to complete

    // Set IR LED Current (0x1F to 0x0C)
    // This is crucial for the LED to light up and provide a signal.
    i2c_bus.write(DEVICE_ADDRESS, &[IR_LED_CONF_REG_ADDR, 0xFF]).await.ok();

    i2c_bus.write(DEVICE_ADDRESS, &[LED_CONF_REG_ADDR, 0xE3]).await.ok(); // increasing pulse width

    // Set Mode to Heart Rate Only (0x02 to 0x09)
    i2c_bus
        .write(DEVICE_ADDRESS, &[MODE_CONF_REG_ADDR, HR_ONLY_MODE])
        .await
        .ok();

    let mut raw_data_buffer = [0u8; IR_SAMPLE_SIZE];
    loop {
        // Read 3 bytes from the FIFO data register (0x07)
        if i2c_bus
            .write_read(DEVICE_ADDRESS, &[FIFO_DATA_ADDR], &mut raw_data_buffer)
            .await
            .is_ok()
        {
            // Convert the 3 bytes into a single 24-bit value (u32)
            let ir_value: u32 = ((raw_data_buffer[0] as u32 & 0x0F) << 16) | // Masking/shifting
                    ((raw_data_buffer[1] as u32) << 8) |
                    (raw_data_buffer[2] as u32);

            defmt::info!("IR Data: {}", ir_value);
        } else {
            defmt::error!("FIFO read failed.");
        }

        // Wait for the next sample (adjust based on your desired sample rate)
        Timer::after_millis(10).await;
    }
}
