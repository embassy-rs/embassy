//! This example shows how you can use PIO to read one or more `DS18B20`
//! one-wire temperature sensors using parasite power.
//! It applies a strong pullup during conversion, see "Powering the DS18B20" in the datasheet.
//! For externally powered sensors, use the pio_onewire.rs example.

#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::onewire::{PioOneWire, PioOneWireProgram, PioOneWireSearch};
use embassy_time::Duration;
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
                    info!("Found address: {:x}", address);
                    let _ = devices.push(address);
                } else {
                    warn!("Found invalid address: {:x}", address);
                }
            }
        }
    }

    info!("Search done, found {} devices", devices.len());

    loop {
        // Read all devices one by one
        for device in &devices {
            onewire.reset().await;
            onewire.write_bytes(&[0x55]).await; // Match rom
            onewire.write_bytes(&device.to_le_bytes()).await;
            // 750 ms delay required for default 12-bit resolution.
            onewire.write_bytes_pullup(&[0x44], Duration::from_millis(750)).await;

            onewire.reset().await;
            onewire.write_bytes(&[0x55]).await; // Match rom
            onewire.write_bytes(&device.to_le_bytes()).await;
            onewire.write_bytes(&[0xBE]).await; // Read scratchpad

            let mut data = [0; 9];
            onewire.read_bytes(&mut data).await;
            if crc8(&data) == 0 {
                let temp = ((data[1] as i16) << 8 | data[0] as i16) as f32 / 16.;
                info!("Read device {:x}: {} deg C", device, temp);
            } else {
                warn!("Reading device {:x} failed. {:02x}", device, data);
            }
        }
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
