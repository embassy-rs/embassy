//! This examples shows how you can use buffered or DMA UART to read a DS18B20 temperature sensor on 1-Wire bus.
//! Connect 5k pull-up resistor between PA9 and 3.3V.
#![no_std]
#![no_main]

use cortex_m::singleton;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::{
    BufferedUartRx, BufferedUartTx, Config, ConfigError, OutputConfig, RingBufferedUartRx, UartTx,
};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

/// Create onewire bus using DMA USART
fn create_onewire(p: embassy_stm32::Peripherals) -> OneWire<UartTx<'static, Async>, RingBufferedUartRx<'static>> {
    use embassy_stm32::usart::Uart;
    bind_interrupts!(struct Irqs {
        USART1 => usart::InterruptHandler<peripherals::USART1>;
    });

    let mut config = Config::default();
    config.tx_config = OutputConfig::OpenDrain;

    let usart = Uart::new_half_duplex(
        p.USART1,
        p.PA9,
        Irqs,
        p.DMA1_CH1,
        p.DMA1_CH2,
        config,
        // Enable readback so we can read sensor pulling data low while transmission is in progress
        usart::HalfDuplexReadback::Readback,
    )
    .unwrap();

    const BUFFER_SIZE: usize = 16;
    let (tx, rx) = usart.split();
    let rx_buf: &mut [u8; BUFFER_SIZE] = singleton!(TX_BUF: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();
    let rx = rx.into_ring_buffered(rx_buf);
    OneWire::new(tx, rx)
}

/*
/// Create onewire bus using buffered USART
fn create_onewire(p: embassy_stm32::Peripherals) -> OneWire<BufferedUartTx<'static>, BufferedUartRx<'static>> {
    use embassy_stm32::usart::BufferedUart;
    bind_interrupts!(struct Irqs {
        USART1 => usart::BufferedInterruptHandler<peripherals::USART1>;
    });

    const BUFFER_SIZE: usize = 16;
    let mut config = Confi::default();
    config.tx_config = OutputConfig::OpenDrain;
    let tx_buf: &mut [u8; BUFFER_SIZE] = singleton!(TX_BUF: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();
    let rx_buf: &mut [u8; BUFFER_SIZE] = singleton!(RX_BUF: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();
    let usart = BufferedUart::new_half_duplex(
        p.USART1,
        p.PA9,
        Irqs,
        tx_buf,
        rx_buf,
        config,
        // Enable readback so we can read sensor pulling data low while transmission is in progress
        usart::HalfDuplexReadback::Readback,
    )
    .unwrap();
    let (tx, rx) = usart.split();
    OneWire::new(tx, rx)
}
*/

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let onewire = create_onewire(p);
    let mut sensor = Ds18b20::new(onewire);

    loop {
        // Start a new temperature measurement
        sensor.start().await;
        // Wait for the measurement to finish
        Timer::after(Duration::from_secs(1)).await;
        match sensor.temperature().await {
            Ok(temp) => info!("temp = {:?} deg C", temp),
            _ => error!("sensor error"),
        }
        Timer::after(Duration::from_secs(1)).await;
    }
}

pub trait SetBaudrate {
    fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError>;
}
impl SetBaudrate for BufferedUartTx<'_> {
    fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        BufferedUartTx::set_baudrate(self, baudrate)
    }
}
impl SetBaudrate for BufferedUartRx<'_> {
    fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        BufferedUartRx::set_baudrate(self, baudrate)
    }
}
impl SetBaudrate for RingBufferedUartRx<'_> {
    fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        RingBufferedUartRx::set_baudrate(self, baudrate)
    }
}
impl SetBaudrate for UartTx<'_, Async> {
    fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        UartTx::set_baudrate(self, baudrate)
    }
}

/// Simplified OneWire bus driver
pub struct OneWire<TX, RX>
where
    TX: embedded_io_async::Write + SetBaudrate,
    RX: embedded_io_async::Read + SetBaudrate,
{
    tx: TX,
    rx: RX,
}

impl<TX, RX> OneWire<TX, RX>
where
    TX: embedded_io_async::Write + SetBaudrate,
    RX: embedded_io_async::Read + SetBaudrate,
{
    // bitrate with one bit taking ~104 us
    const RESET_BUADRATE: u32 = 9600;
    // bitrate with one bit taking ~8.7 us
    const BAUDRATE: u32 = 115200;

    // startbit + 8 low bits = 9 * 1/115200 = 78 us low pulse
    const LOGIC_1_CHAR: u8 = 0xFF;
    // startbit only = 1/115200 = 8.7 us low pulse
    const LOGIC_0_CHAR: u8 = 0x00;

    // Address all devices on the bus
    const COMMAND_SKIP_ROM: u8 = 0xCC;

    pub fn new(tx: TX, rx: RX) -> Self {
        Self { tx, rx }
    }

    fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        self.tx.set_baudrate(baudrate)?;
        self.rx.set_baudrate(baudrate)
    }

    /// Reset the bus by at least 480 us low pulse.
    pub async fn reset(&mut self) {
        // Switch to 9600 baudrate, so one bit takes ~104 us
        self.set_baudrate(Self::RESET_BUADRATE).expect("set_baudrate failed");
        // Low USART start bit + 4x low bits = 5 * 104 us = 520 us low pulse
        self.tx.write(&[0xF0]).await.expect("write failed");

        // Read the value on the bus
        let mut buffer = [0; 1];
        self.rx.read_exact(&mut buffer).await.expect("read failed");

        // Switch back to 115200 baudrate, so one bit takes ~8.7 us
        self.set_baudrate(Self::BAUDRATE).expect("set_baudrate failed");

        // read and expect sensor pulled some high bits to low (device present)
        if buffer[0] & 0xF != 0 || buffer[0] & 0xF0 == 0xF0 {
            warn!("No device present");
        }
    }

    /// Send byte and read response on the bus.
    pub async fn write_read_byte(&mut self, byte: u8) -> u8 {
        // One byte is sent as 8 UART characters
        let mut tx = [0; 8];
        for (pos, char) in tx.iter_mut().enumerate() {
            *char = if (byte >> pos) & 0x1 == 0x1 {
                Self::LOGIC_1_CHAR
            } else {
                Self::LOGIC_0_CHAR
            };
        }
        self.tx.write_all(&tx).await.expect("write failed");

        // Readback the value on the bus, sensors can pull logic 1 to 0
        let mut rx = [0; 8];
        self.rx.read_exact(&mut rx).await.expect("read failed");
        let mut bus_byte = 0;
        for (pos, char) in rx.iter().enumerate() {
            // if its 0xFF, sensor didnt pull the bus to low level
            if *char == 0xFF {
                bus_byte |= 1 << pos;
            }
        }

        bus_byte
    }

    /// Read a byte from the bus.
    pub async fn read_byte(&mut self) -> u8 {
        self.write_read_byte(0xFF).await
    }
}

/// DS18B20 temperature sensor driver
pub struct Ds18b20<TX, RX>
where
    TX: embedded_io_async::Write + SetBaudrate,
    RX: embedded_io_async::Read + SetBaudrate,
{
    bus: OneWire<TX, RX>,
}

impl<TX, RX> Ds18b20<TX, RX>
where
    TX: embedded_io_async::Write + SetBaudrate,
    RX: embedded_io_async::Read + SetBaudrate,
{
    /// Start a temperature conversion.
    const FN_CONVERT_T: u8 = 0x44;
    /// Read contents of the scratchpad containing the temperature.
    const FN_READ_SCRATCHPAD: u8 = 0xBE;

    pub fn new(bus: OneWire<TX, RX>) -> Self {
        Self { bus }
    }

    /// Start a new measurement. Allow at least 1000ms before getting `temperature`.
    pub async fn start(&mut self) {
        self.bus.reset().await;
        self.bus.write_read_byte(OneWire::<TX, RX>::COMMAND_SKIP_ROM).await;
        self.bus.write_read_byte(Self::FN_CONVERT_T).await;
    }

    /// Calculate CRC8 of the data
    fn crc8(data: &[u8]) -> u8 {
        let mut temp;
        let mut data_byte;
        let mut crc = 0;
        for b in data {
            data_byte = *b;
            for _ in 0..8 {
                temp = (crc ^ data_byte) & 0x01;
                crc >>= 1;
                if temp != 0 {
                    crc ^= 0x8C;
                }
                data_byte >>= 1;
            }
        }
        crc
    }

    /// Read the temperature. Ensure >1000ms has passed since `start` before calling this.
    pub async fn temperature(&mut self) -> Result<f32, ()> {
        self.bus.reset().await;
        self.bus.write_read_byte(OneWire::<TX, RX>::COMMAND_SKIP_ROM).await;
        self.bus.write_read_byte(Self::FN_READ_SCRATCHPAD).await;

        let mut data = [0; 9];
        for byte in data.iter_mut() {
            *byte = self.bus.read_byte().await;
        }

        match Self::crc8(&data) == 0 {
            true => Ok(((data[1] as u16) << 8 | data[0] as u16) as f32 / 16.),
            false => Err(()),
        }
    }
}
