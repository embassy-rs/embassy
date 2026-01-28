#![no_std]

mod filter;
mod hr_detector;

pub use filter::PpgFilter;
pub use hr_detector::HrDetector;

use embassy_time::Timer;

/// MAX30102 I2C device address (7-bit)
pub const DEVICE_ADDRESS: u8 = 0x57;

/// Register addresses
mod registers {
    pub const MODE_CONF: u8 = 0x09;
    pub const LED_CONF: u8 = 0x0A;
    pub const IR_LED_CURRENT: u8 = 0x0C;
    pub const PART_ID: u8 = 0xFF;
    pub const FIFO_WR_PTR: u8 = 0x04;
    pub const FIFO_RD_PTR: u8 = 0x06;
    pub const FIFO_DATA: u8 = 0x07;
    pub const FIFO_OVF: u8 = 0x05;
}

/// Mode configuration values
mod modes {
    pub const HR_ONLY: u8 = 1 << 1;
    pub const RESET: u8 = 0b0100_0000;
}

/// FIFO sample size (18-bit, 3 bytes)
const IR_SAMPLE_SIZE: usize = 3;

/// MAX30102 driver
pub struct Max30102<I2C> {
    i2c: I2C,
    min_ir_threshold: u32,
    no_finger_threshold_count: u32,
}

impl<I2C> Max30102<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    /// Create a new MAX30102 driver instance
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            min_ir_threshold: 5000,
            no_finger_threshold_count: 50,
        }
    }

    /// Set the minimum IR threshold for finger detection
    pub fn set_min_ir_threshold(&mut self, threshold: u32) {
        self.min_ir_threshold = threshold;
    }

    /// Set the number of consecutive low readings to confirm no finger
    pub fn set_no_finger_threshold_count(&mut self, count: u32) {
        self.no_finger_threshold_count = count;
    }

    /// Initialize the sensor
    pub async fn init(&mut self) -> Result<u8, I2C::Error> {
        // Read Part ID
        let mut read_buffer = [0u8; 1];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[registers::PART_ID], &mut read_buffer)
            .await?;
        let part_id = read_buffer[0];

        // Reset the sensor
        self.i2c
            .write(DEVICE_ADDRESS, &[registers::MODE_CONF, modes::RESET])
            .await?;
        Timer::after_millis(50).await; // Wait for reset to complete

        // Set IR LED Current (0xFF = maximum)
        self.i2c
            .write(DEVICE_ADDRESS, &[registers::IR_LED_CURRENT, 0xFF])
            .await?;

        // Set LED configuration (pulse width and sample rate)
        self.i2c
            .write(DEVICE_ADDRESS, &[registers::LED_CONF, 0xE3])
            .await?;

        // Set Mode to Heart Rate Only
        self.i2c
            .write(DEVICE_ADDRESS, &[registers::MODE_CONF, modes::HR_ONLY])
            .await?;

        Ok(part_id)
    }

    /// Read the number of samples available in FIFO
    pub async fn samples_available(&mut self) -> Result<u8, I2C::Error> {
        let mut wr_ptr = [0u8; 1];
        let mut rd_ptr = [0u8; 1];

        self.i2c
            .write_read(DEVICE_ADDRESS, &[registers::FIFO_WR_PTR], &mut wr_ptr)
            .await?;
        self.i2c
            .write_read(DEVICE_ADDRESS, &[registers::FIFO_RD_PTR], &mut rd_ptr)
            .await?;

        let wr = wr_ptr[0] & 0x1F; // 5-bit pointer
        let rd = rd_ptr[0] & 0x1F;

        let samples = if wr >= rd {
            (wr - rd) as u8
        } else {
            (32 - rd + wr) as u8
        };

        Ok(samples)
    }

    /// Check for FIFO overflow and clear it if needed
    pub async fn check_overflow(&mut self) -> Result<bool, I2C::Error> {
        let mut ovf = [0u8; 1];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[registers::FIFO_OVF], &mut ovf)
            .await?;

        if ovf[0] != 0 {
            // Clear overflow by reading FIFO_RD_PTR
            let mut rd_ptr = [0u8; 1];
            self.i2c
                .write_read(DEVICE_ADDRESS, &[registers::FIFO_RD_PTR], &mut rd_ptr)
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Read one IR sample from FIFO
    pub async fn read_sample(&mut self) -> Result<u32, I2C::Error> {
        let mut buffer = [0u8; IR_SAMPLE_SIZE];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[registers::FIFO_DATA], &mut buffer)
            .await?;

        // Interpret the 3 bytes as an 18-bit sample (MSB first)
        let ir_value: u32 = (((buffer[0] as u32) & 0x03) << 16)
            | ((buffer[1] as u32) << 8)
            | (buffer[2] as u32);

        Ok(ir_value)
    }

    /// Drain all available samples from FIFO
    pub async fn drain_fifo(&mut self, max_samples: u8) -> Result<u8, I2C::Error> {
        let mut drained = 0;
        let samples = self.samples_available().await?;

        for _ in 0..samples.min(max_samples) {
            let mut dummy = [0u8; IR_SAMPLE_SIZE];
            if self
                .i2c
                .write_read(DEVICE_ADDRESS, &[registers::FIFO_DATA], &mut dummy)
                .await
                .is_ok()
            {
                drained += 1;
            } else {
                break;
            }
        }

        Ok(drained)
    }

    /// Check if finger is present based on IR value
    pub fn is_finger_present(&self, ir_value: u32) -> bool {
        ir_value >= self.min_ir_threshold
    }

    /// Get the minimum IR threshold
    pub fn min_ir_threshold(&self) -> u32 {
        self.min_ir_threshold
    }

    /// Get the no-finger threshold count
    pub fn no_finger_threshold_count(&self) -> u32 {
        self.no_finger_threshold_count
    }
}
