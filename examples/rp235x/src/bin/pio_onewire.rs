//! This example shows how you can use PIO to read a `DS18B20` one-wire temperature sensor.

#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{self, InterruptHandler, Pio};
use embassy_rp::pio_programs::onewire::{PioOneWire, PioOneWireProgram};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut pio = Pio::new(p.PIO0, Irqs);

    let prg = PioOneWireProgram::new(&mut pio.common);
    let onewire = PioOneWire::new(&mut pio.common, pio.sm0, p.PIN_2, &prg);

    let mut sensor = Ds18b20::new(onewire);

    loop {
        sensor.start().await; // Start a new measurement
        Timer::after_secs(1).await; // Allow 1s for the measurement to finish
        match sensor.temperature().await {
            Ok(temp) => info!("temp = {:?} deg C", temp),
            _ => error!("sensor error"),
        }
        Timer::after_secs(1).await;
    }
}

/// DS18B20 temperature sensor driver
pub struct Ds18b20<'d, PIO: pio::Instance, const SM: usize> {
    wire: PioOneWire<'d, PIO, SM>,
}

impl<'d, PIO: pio::Instance, const SM: usize> Ds18b20<'d, PIO, SM> {
    pub fn new(wire: PioOneWire<'d, PIO, SM>) -> Self {
        Self { wire }
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

    /// Start a new measurement. Allow at least 1000ms before getting `temperature`.
    pub async fn start(&mut self) {
        self.wire.write_bytes(&[0xCC, 0x44]).await;
    }

    /// Read the temperature. Ensure >1000ms has passed since `start` before calling this.
    pub async fn temperature(&mut self) -> Result<f32, ()> {
        self.wire.write_bytes(&[0xCC, 0xBE]).await;
        let mut data = [0; 9];
        self.wire.read_bytes(&mut data).await;
        match Self::crc8(&data) == 0 {
            true => Ok(((data[1] as u32) << 8 | data[0] as u32) as f32 / 16.),
            false => Err(()),
        }
    }
}
