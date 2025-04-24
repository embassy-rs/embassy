//! This example shows how you can use PIO to read one or more `DS18B20` one-wire temperature sensors.

#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::onewire::{PioOneWire, PioOneWireProgram, PioOneWireSearch};
use embassy_time::Timer;
use heapless::Vec;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut pio = Pio::new(p.PIO0, Irqs);

    let prg = PioOneWireProgram::new(&mut pio.common);
    let mut onewire = PioOneWire::new(&mut pio.common, pio.sm0, p.PIN_2, &prg);

    info!("Starting onewire search");

    let mut devices = Vec::<u64, 10>::new();
    let mut search = PioOneWireSearch::new();
    for _ in 0..10 {
        if !search.is_finished() {
            if let Some(address) = search.next(&mut onewire).await {
                if crc8(&address.to_le_bytes()) == 0 {
                    info!("Found addres: {:x}", address);
                    let _ = devices.push(address);
                } else {
                    warn!("Found invalid address: {:x}", address);
                }
            }
        }
    }

    info!("Search done, found {} devices", devices.len());

    loop {
        onewire.reset().await;
        // Skip rom and trigger conversion, we can trigger all devices on the bus immediately
        onewire.write_bytes(&[0xCC, 0x44]).await;

        Timer::after_secs(1).await; // Allow 1s for the measurement to finish

        // Read all devices one by one
        for device in &devices {
            onewire.reset().await;
            onewire.write_bytes(&[0x55]).await; // Match rom
            onewire.write_bytes(&device.to_le_bytes()).await;
            onewire.write_bytes(&[0xBE]).await; // Read scratchpad

            let mut data = [0; 9];
            onewire.read_bytes(&mut data).await;
            if crc8(&data) == 0 {
                let temp = ((data[1] as u32) << 8 | data[0] as u32) as f32 / 16.;
                info!("Read device {:x}: {} deg C", device, temp);
            } else {
                warn!("Reading device {:x} failed", device);
            }
        }
        Timer::after_secs(1).await;
    }
}

fn crc8(data: &[u8]) -> u8 {
    let mut crc = 0;
    for b in data {
        let mut data_byte = *b;
        for _ in 0..8 {
            let temp = (crc ^ data_byte) & 0x01;
            crc >>= 1;
            if temp != 0 {
                crc ^= 0x8C;
            }
            data_byte >>= 1;
        }
    }
    crc
}
