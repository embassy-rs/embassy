#![no_std]
#![no_main]

mod max30102;

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_time::Instant;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::peripherals::I2C1;
use embassy_stm32::{Config, bind_interrupts};
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
const PART_ID_REG_ADDR: u8 = 0xFF; // Part ID Register
const HR_ONLY_MODE: u8 = 1 << 1;
const RESET_COMMAND: u8 = 0b0100_0000;
const FIFO_WR_PTR_ADDR: u8 = 0x04;
const FIFO_RD_PTR_ADDR: u8 = 0x06;
const FIFO_DATA_ADDR: u8 = 0x07;
const FIFO_OVF_ADDR: u8 = 0x05; // FIFO Overflow register
const IR_SAMPLE_SIZE: usize = 3; // Each sample is 3 bytes (18-bit)
use max30102::*;

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

    let mut filter = PpgFilter::new(100.0); // 100 Hz sampling
    let mut hr_detector = HrDetector::new();
    let mut no_finger_count: u32 = 0; // Count consecutive low IR readings
    let mut finger_present = false;
    let mut no_finger_display_count: u32 = 0; // Counter for displaying BPM when no finger

    // Let filter stabilize (high-pass needs time to remove DC)
    defmt::info!("Stabilizing filter...");
    for _ in 0..100 {
        let mut dummy_buffer = [0u8; IR_SAMPLE_SIZE];
        if i2c_bus
            .write_read(DEVICE_ADDRESS, &[FIFO_DATA_ADDR], &mut dummy_buffer)
            .await
            .is_ok()
        {
            let dummy_ir: u32 =
                (((dummy_buffer[0] as u32) & 0x03) << 16) |
                ((dummy_buffer[1] as u32) << 8) |
                (dummy_buffer[2] as u32);
            filter.process(dummy_ir);
        }
        Timer::after_millis(10).await;
    }
    defmt::info!("Filter stabilized, starting HR detection...");

    let mut raw_data_buffer = [0u8; IR_SAMPLE_SIZE];
    let mut sample_count: u32 = 0;
    
    loop {
        // Check FIFO pointers to see how many samples are available
        let mut wr_ptr = [0u8; 1];
        let mut rd_ptr = [0u8; 1];
        let mut ovf = [0u8; 1];
        
        // Read FIFO pointers
        if i2c_bus.write_read(DEVICE_ADDRESS, &[FIFO_WR_PTR_ADDR], &mut wr_ptr).await.is_ok() &&
           i2c_bus.write_read(DEVICE_ADDRESS, &[FIFO_RD_PTR_ADDR], &mut rd_ptr).await.is_ok() &&
           i2c_bus.write_read(DEVICE_ADDRESS, &[FIFO_OVF_ADDR], &mut ovf).await.is_ok()
        {
            let wr = wr_ptr[0] & 0x1F; // 5-bit pointer
            let rd = rd_ptr[0] & 0x1F;
            let mut samples_available = if wr >= rd { (wr - rd) as u8 } else { (32 - rd + wr) as u8 };
            
            // Check for overflow
            if ovf[0] != 0 {
                defmt::warn!("FIFO overflow detected! Clearing...");
                // Clear overflow by reading FIFO_RD_PTR
                let _ = i2c_bus.write_read(DEVICE_ADDRESS, &[FIFO_RD_PTR_ADDR], &mut rd_ptr).await;
            }
            
            // Skip reading FIFO if no finger is detected (more efficient)
            const MIN_IR_THRESHOLD: u32 = 5000; // Adjust based on your sensor/lighting
            const NO_FINGER_THRESHOLD_COUNT: u32 = 50; // Need 50 consecutive low readings to confirm no finger
            
            if !finger_present && samples_available > 0 {
                // Finger was removed - drain FIFO once to clear stale samples, then check if finger returned
                let mut dummy = [0u8; IR_SAMPLE_SIZE];
                if i2c_bus.write_read(DEVICE_ADDRESS, &[FIFO_DATA_ADDR], &mut dummy).await.is_ok() {
                    let check_ir: u32 =
                        (((dummy[0] as u32) & 0x03) << 16) |
                        ((dummy[1] as u32) << 8) |
                        (dummy[2] as u32);
                    
                    if check_ir >= MIN_IR_THRESHOLD {
                        // Finger returned - drain remaining stale samples once, then start processing
                        defmt::info!("Finger detected (ir={=u32}), clearing stale samples", check_ir);
                        let mut drained = 0;
                        while samples_available > 1 && drained < 31 {
                            let mut dummy2 = [0u8; IR_SAMPLE_SIZE];
                            if i2c_bus.write_read(DEVICE_ADDRESS, &[FIFO_DATA_ADDR], &mut dummy2).await.is_ok() {
                                drained += 1;
                                samples_available -= 1;
                            } else {
                                break;
                            }
                        }
                        hr_detector.reset();
                        finger_present = true;
                        no_finger_count = 0;
                        no_finger_display_count = 0; // Reset counter when finger returns
                    } else {
                        // Still no finger - display 0 BPM periodically
                        no_finger_display_count += 1;
                        if no_finger_display_count % 50 == 0 {
                            defmt::info!("HR: 0.0 BPM (no finger)");
                        }
                        Timer::after_millis(10).await;
                        continue;
                    }
                }
            }
            
            // Read one sample if available and finger is present
            if samples_available > 0 && finger_present {
                if i2c_bus
                    .write_read(DEVICE_ADDRESS, &[FIFO_DATA_ADDR], &mut raw_data_buffer)
                    .await
                    .is_ok()
                {
                    // Interpret the 3 bytes as an 18-bit sample (MSB first)
                    let ir_value: u32 =
                        (((raw_data_buffer[0] as u32) & 0x03) << 16) |
                        ((raw_data_buffer[1] as u32) << 8) |
                        (raw_data_buffer[2] as u32);
                    
                    // Check if finger is still present
                    if ir_value < MIN_IR_THRESHOLD {
                        no_finger_count += 1;
                        
                        // Only reset after sustained low readings (debounce)
                        if no_finger_count >= NO_FINGER_THRESHOLD_COUNT {
                            defmt::warn!("Finger removed (ir={=u32}), resetting detector", ir_value);
                            hr_detector.reset();
                            finger_present = false;
                        }
                        
                        // Don't process this sample - continue to next iteration
                        Timer::after_millis(10).await;
                        continue;
                    } else {
                        // Finger is present - process normally
                        no_finger_count = 0; // Reset counter when finger is present
                        
                        // Process this sample normally
                        let filtered = filter.process(ir_value);
                        let now_ms = Instant::now().as_millis() as u32;
                        sample_count += 1;
                                               
                        // Update HR detector (always process, even if no heartbeat detected)
                        hr_detector.update(filtered, now_ms);
                        
                        // Always display current BPM (even if 0) - update every 50 samples (~0.5 seconds at 100Hz)
                        if sample_count % 50 == 0 {
                            let current_bpm = hr_detector.current_bpm();
                            defmt::info!("HR: {=f32} BPM", current_bpm);
                        }
                    }
                } else {
                    defmt::error!("FIFO read failed.");
                }
            } else {
                // No samples available, wait a bit
                Timer::after_millis(1).await;
            }
        } else {
            defmt::error!("Failed to read FIFO pointers.");
            Timer::after_millis(10).await;
        }

        // Small delay to prevent tight loop
        Timer::after_millis(1).await;
    }
}
