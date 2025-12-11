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

const MAX30100_I2C_ADDR: u8 = 0x57;
const MAX30100_I2C_WRITE_ADDR: u8 = 0xAE;
const MAX30100_I2C_READ_ADDR: u8 = 0xAF;
const MAX30100_PART_ID_REG_ADDR: u8 = 0xFF;
const MAX30100_MODE_CONFIG_REG_ADDR: u8 = 0x06;

const DEVICE_ADDRESS: u8 = 0x57; // 7-bit I2C Address (Confirmed)
const MODE_CONF_REG_ADDR: u8 = 0x09; // CORRECTED Mode Config Register Address
const LED_CONF_REG_ADDR: u8 = 0x0A; // SpO2 Configuration Register (used to set LED pulse width/rate)
const IR_LED_CONF_REG_ADDR: u8 = 0x0C; // IR LED Current Register
const RED_LED_CONF_REG_ADDR: u8 = 0x0D; // Red LED Current Register
const HR_ONLY_MODE: u8 = 1 << 1;
const RESET_COMMAND: u8 = 0b0100_0000;
const FIFO_WR_PTR_ADDR: u8 = 0x04;
const FIFO_RD_PTR_ADDR: u8 = 0x06;
const FIFO_DATA_ADDR: u8 = 0x07;

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
    defmt::info!("Triggering MAX30100 software reset (0x40 to 0x06)...");

    if let Err(e) = i2c_bus
        .write(DEVICE_ADDRESS, &[MODE_CONF_REG_ADDR, RESET_COMMAND])
        .await
    {
        defmt::error!("I2C Reset failed: {:?}", e);
    } else {
        defmt::info!("Reset initiated.");
    }

    // ⚠️ IMPORTANT: Add a short delay after reset to ensure the chip is ready
    Timer::after_millis(50).await;

    // Step 2: Configure LED Pulse Width and Sample Rate (Register 0x0A)
    // Setting: 400us pulse width, 100 sps (0b0010_0011)
    // This value helps the LEDs light up reliably.
    const SPO2_CONF_VALUE: u8 = 0b0010_0011; // 0x23

    defmt::info!("Setting LED pulse width and sample rate (0x23 to 0x0A)...");
    if let Err(e) = i2c_bus
        .write(DEVICE_ADDRESS, &[LED_CONF_REG_ADDR, SPO2_CONF_VALUE])
        .await
    {
        defmt::error!("I2C Config 0x0A failed: {:?}", e);
        return;
    }

    // Step 3: Set Red LED Current (Register 0x0D)
    // Value 0x15 = ~6.7mA (Visible, won't draw excessive power)
    const LED_CURRENT_VALUE: u8 = 0x15;

    defmt::info!("Setting Red LED current to 0x15 (0x15 to 0x0D)...");
    if let Err(e) = i2c_bus
        .write(DEVICE_ADDRESS, &[RED_LED_CONF_REG_ADDR, LED_CURRENT_VALUE])
        .await
    {
        defmt::error!("I2C Config 0x0D failed: {:?}", e);
        return;
    }
    // Set IR LED current to the same value (Register 0x0C)
    if let Err(e) = i2c_bus
        .write(DEVICE_ADDRESS, &[IR_LED_CONF_REG_ADDR, LED_CURRENT_VALUE])
        .await
    {
        defmt::error!("I2C Config 0x0C failed: {:?}", e);
        return;
    }

    // Step 4: Set Mode Configuration Register (0x09)
    // Value 0x02 = Heart Rate (HR) Only Mode (Bits 2:0 = 010)
    const HR_MODE_SET: u8 = 0b0000_0010; // 0x02

    defmt::info!("Setting Mode to HR Only (0x02 to 0x09)...");
    if let Err(e) = i2c_bus.write(DEVICE_ADDRESS, &[MODE_CONF_REG_ADDR, HR_MODE_SET]).await {
        defmt::error!("I2C Config 0x09 failed: {:?}", e);
        return;
    } else {
        defmt::info!("Initialization COMPLETE. Sensor should be pulsing.");
    }

    let mut raw_data_buffer = [0u8; 3];
    loop {
        if let Err(e) = i2c_bus
            .write_read(
                DEVICE_ADDRESS,
                &[FIFO_DATA_ADDR], // 1. Start by telling the chip to read from the FIFO Data register
                &mut raw_data_buffer, // 2. Read 6 bytes
            )
            .await
        {
            defmt::error!("I2C FIFO Read failed: {:?}", e);
        } else {
            let red_value: u32 = ((raw_data_buffer[0] as u32 & 0x03) << 16) | // Masking the two most significant bits
                ((raw_data_buffer[1] as u32) << 8) |
                (raw_data_buffer[2] as u32);
            defmt::info!("RAW FIFO Data: {} = {:?}", red_value, raw_data_buffer);
        }
        // Timer::after_millis(1000).await;

        // let mut read_buffer = [0u8];
        // if let Err(e) = i2c_bus
        //     .write_read(MAX30100_I2C_ADDR, &[MAX30100_PART_ID_REG_ADDR], &mut read_buffer)
        //     .await
        // {
        //     defmt::warn!("MAX30100 I2C read failed: {:?}", e);
        //     // return;
        // }
        // println!("read_buffer = {:?}", read_buffer);

        // MAX30100 info:

        // Read part ID register:
        // if let Err(e) = i2c_bus
        //     .write_read(DEVICE_ADDRESS, &[PART_ID_REG_ADDR], &mut read_buffer)
        //     .await
        // {
        //     // Handle error (e.g., Nack, Timeout)
        //     defmt::error!("I2C failed: {:?}", e);
        // } else {
        //     // read_buffer[0] now contains the PART ID (should be 0x11)
        //     defmt::info!("PART_ID_REG: 0x{:x}", read_buffer);
        // }

        // // Reading MODE register:
        // let mut mode_reg = [0u8];
        // if let Err(e) = i2c_bus
        //     .write_read(DEVICE_ADDRESS, &[MODE_CONF_REG_ADDR], &mut mode_reg)
        //     .await
        // {
        //     // Handle error (e.g., Nack, Timeout)
        //     defmt::error!("I2C failed: {:?}", e);
        // } else {
        //     // read_buffer[0] now contains the PART ID (should be 0x11)
        //     defmt::info!("MODE_CONF_REG: 0x{:x}", mode_reg);
        // }

        // // Setting HR mode:
        // let new_mode = mode_reg[0] | HR_ONLY_MODE;
        // println!("Setting MODE_REG = 0x{:x}", new_mode);
        // if let Err(e) = i2c_bus.write(DEVICE_ADDRESS, &[MODE_CONF_REG_ADDR, new_mode]).await {
        //     // Handle error (e.g., Nack, Timeout)
        //     defmt::error!("I2C failed: {:?}", e);
        // }
        // Timer::after_millis(1000).await;
        // // else {
        // //     // read_buffer[0] now contains the PART ID (should be 0x11)
        // //     defmt::info!("MODE_CONF_REG: 0x{:x}", read_buffer[0]);
        // // }

        // if let Err(e) = i2c_bus
        //     .write_read(DEVICE_ADDRESS, &[MODE_CONF_REG_ADDR], &mut mode_reg)
        //     .await
        // {
        //     // Handle error (e.g., Nack, Timeout)
        //     defmt::error!("I2C failed: {:?}", e);
        // } else {
        //     // read_buffer[0] now contains the PART ID (should be 0x11)
        //     defmt::info!("MODE_CONF_REG: 0x{:x}", mode_reg);
        // }
    }
}
