//! Complete I2C slave example using blocking operations
//!
//! This example shows how to set up an STM32F4 as an I2C slave device
//! that can handle both read and write transactions from master devices.

#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::i2c::{self, Address, I2c, SlaveAddrConfig, SlaveCommand, SlaveCommandKind};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

pub const I2C_SLAVE_ADDR: u8 = 0x42;
pub const BUFFER_SIZE: usize = 8;
static I2C_BUFFER: Mutex<ThreadModeRawMutex, [u8; BUFFER_SIZE]> = Mutex::new([0; BUFFER_SIZE]);

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // Configure I2C
    let mut i2c_config = i2c::Config::default();
    i2c_config.sda_pullup = false;
    i2c_config.scl_pullup = false;
    i2c_config.frequency = Hertz(100_000);
    i2c_config.timeout = embassy_time::Duration::from_millis(30000);

    // Initialize I2C as master first
    let i2c_master = I2c::new_blocking(
        p.I2C1, p.PB8, // SCL
        p.PB9, // SDA
        i2c_config,
    );

    // Convert to slave+master mode
    let slave_config = SlaveAddrConfig::basic(I2C_SLAVE_ADDR);
    let i2c_slave = i2c_master.into_slave_multimaster(slave_config);

    spawner.spawn(i2c_slave_task(i2c_slave).unwrap());
}

#[embassy_executor::task]
pub async fn i2c_slave_task(mut i2c_slave: I2c<'static, embassy_stm32::mode::Blocking, i2c::mode::MultiMaster>) {
    info!("Blocking I2C slave ready at address 0x{:02X}", I2C_SLAVE_ADDR);

    loop {
        match i2c_slave.blocking_listen() {
            Ok(SlaveCommand {
                kind: SlaveCommandKind::Write,
                address,
            }) => {
                let addr_val = match address {
                    Address::SevenBit(addr) => addr,
                    Address::TenBit(addr) => (addr & 0xFF) as u8,
                };

                info!("I2C: Received write command - Address 0x{:02X}", addr_val);
                let mut data_buffer = I2C_BUFFER.lock().await;

                match i2c_slave.blocking_respond_to_write(&mut *data_buffer) {
                    Ok(bytes_received) => {
                        info!(
                            "I2C: Received {} bytes - Buffer now contains: 0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}",
                            bytes_received,
                            data_buffer[0],
                            data_buffer[1],
                            data_buffer[2],
                            data_buffer[3],
                            data_buffer[4],
                            data_buffer[5],
                            data_buffer[6],
                            data_buffer[7]
                        );
                    }
                    Err(e) => {
                        error!("I2C: Write error: {}", format_i2c_error(&e));
                    }
                }
            }

            Ok(SlaveCommand {
                kind: SlaveCommandKind::Read,
                address,
            }) => {
                let addr_val = match address {
                    Address::SevenBit(addr) => addr,
                    Address::TenBit(addr) => (addr & 0xFF) as u8, // Show low byte for 10-bit
                };

                info!("I2C: Received read command - Address 0x{:02X}", addr_val);
                let data_buffer = I2C_BUFFER.lock().await;

                match i2c_slave.blocking_respond_to_read(&data_buffer[..BUFFER_SIZE]) {
                    Ok(bytes_sent) => {
                        info!("I2C: Responded to read - {} bytes sent", bytes_sent);
                    }
                    Err(e) => {
                        error!("I2C: Read error: {}", format_i2c_error(&e));
                    }
                }
            }

            Err(e) => {
                error!("I2C: Listen error: {}", format_i2c_error(&e));
                Timer::after(Duration::from_millis(100)).await;
            }
        }
    }
}

fn format_i2c_error(e: &embassy_stm32::i2c::Error) -> &'static str {
    match e {
        embassy_stm32::i2c::Error::Bus => "Bus",
        embassy_stm32::i2c::Error::Arbitration => "Arbitration",
        embassy_stm32::i2c::Error::Nack => "Nack",
        embassy_stm32::i2c::Error::Timeout => "Timeout",
        embassy_stm32::i2c::Error::Crc => "Crc",
        embassy_stm32::i2c::Error::Overrun => "Overrun",
        embassy_stm32::i2c::Error::ZeroLengthTransfer => "ZeroLengthTransfer",
    }
}
