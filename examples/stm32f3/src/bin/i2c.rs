#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use embassy_time::Instant;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::i2c::{self, I2c, Master};
use embassy_stm32::peripherals::I2C1;
use embassy_stm32::{Config, bind_interrupts, time, usb};
/*
 * Board NUCLEO-F302R8 with STM32F302R8
 * I2C1_SCL: PB8
 * I2C1_SDA: PB9
 *
 * MAX30102_I2C_WRITE_ADDR: 0xAE
 * MAX30102_I2C_READ_ADDR: 0xAF
 *
 *
 */

const DEVICE_ADDRESS: u8 = 0x57; // 7-bit I2C Address (Confirmed)
const MODE_CONF_REG_ADDR: u8 = 0x09; // CORRECTED Mode Config Register Address
const LED_CONF_REG_ADDR: u8 = 0x0A; // SpO2 Configuration Register (used to set LED pulse width/rate)
const IR_LED_CONF_REG_ADDR: u8 = 0x0C; // IR LED Current Register
const RED_LED_CONF_REG_ADDR: u8 = 0x0D; // Red LED Current Register
const PART_ID_REG_ADDR: u8 = 0xFF; // Part ID Register
const HR_ONLY_MODE: u8 = 1 << 1;
const RESET_COMMAND: u8 = 0b0100_0000;
// const FIFO_WR_PTR_ADDR: u8 = 0x04;
// const FIFO_RD_PTR_ADDR: u8 = 0x06;
const FIFO_DATA_ADDR: u8 = 0x07;
const IR_SAMPLE_SIZE: usize = 6;

struct HrDetector {
    baseline: f32,
    last_peak_time_ms: u32,
    bpm: f32,
}

fn update_hr(
    det: &mut HrDetector,
    ir: u32,
    now_ms: u32,
) -> Option<f32> {
    let ir = ir as f32;

    // --- 1. Baseline removal (slow LPF)
    det.baseline = 0.99 * det.baseline + 0.01 * ir;
    let ac = ir - det.baseline;

    // --- 2. Simple threshold peak detection
    const THRESHOLD: f32 = 3000.0;     // tune (depends on LED current)
    const REFRACTORY_MS: u32 = 450;    // max 200 BPM

    if ac > THRESHOLD {
        if now_ms - det.last_peak_time_ms > REFRACTORY_MS {
            if det.last_peak_time_ms != 0 {
                let dt_ms = now_ms - det.last_peak_time_ms;
                let bpm = 60_000.0 / (dt_ms as f32);
                det.bpm = 0.8 * det.bpm + 0.2 * bpm; // smooth BPM
            }
            det.last_peak_time_ms = now_ms;
            return Some(det.bpm);
        }
    }
    None
}

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

    let write_buffer = [PART_ID_REG_ADDR]; // The register address we want to read
    let mut read_buffer = [0u8; 1];
    i2c_bus.write_read(DEVICE_ADDRESS, &write_buffer, &mut read_buffer).await.ok();
    defmt::info!("Part ID: {}", read_buffer[0]);
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


    let mut hr = HrDetector {
        baseline: 0.0,
        last_peak_time_ms: 0,
        bpm: 0.0,
    };

    let mut raw_data_buffer = [0u8; IR_SAMPLE_SIZE];
    loop {
        // Read 3 bytes from the FIFO data register (0x07)
        if i2c_bus
            .write_read(DEVICE_ADDRESS, &[FIFO_DATA_ADDR], &mut raw_data_buffer)
            .await
            .is_ok()
        {
            // interpret the 3 bytes as an 18-bit sample (MSB first)
            let ir_value: u32 =
            (((raw_data_buffer[0] as u32) & 0x03) << 16) |
            ((raw_data_buffer[1] as u32) << 8) |
            (raw_data_buffer[2] as u32);
            let now_ms = Instant::now().as_millis() as u32;
            // defmt::info!("IR Data: {}", ir_value);
            if let Some(bpm) = update_hr(&mut hr, ir_value, now_ms) {
                // Round to 1 decimal place using integer arithmetic (no_std compatible)
                let rounded_bpm = ((bpm * 10.0 + 0.5) as u32) as f32 / 10.0;
                info!("HR: {=f32} BPM", rounded_bpm);
            }
        } else {
            defmt::error!("FIFO read failed.");
        }

        // Wait for the next sample (adjust based on your desired sample rate)
        Timer::after_millis(10).await;
    }
}
