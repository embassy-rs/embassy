//! This example test the flash connected to the RP2040 chip.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::flash::{Async, ERASE_SIZE, FLASH_BASE};
use embassy_rp::peripherals::FLASH;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

const ADDR_OFFSET: u32 = 0x100000;
const FLASH_SIZE: usize = 2 * 1024 * 1024;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    // add some delay to give an attached debug probe time to parse the
    // defmt RTT header. Reading that header might touch flash memory, which
    // interferes with flash write operations.
    // https://github.com/knurling-rs/defmt/pull/683
    Timer::after_millis(10).await;

    let mut flash = embassy_rp::flash::Flash::<_, Async, FLASH_SIZE>::new(p.FLASH, p.DMA_CH0);

    // Get JEDEC id
    let jedec = flash.blocking_jedec_id().unwrap();
    info!("jedec id: 0x{:x}", jedec);

    // Get unique id
    let mut uid = [0; 8];
    flash.blocking_unique_id(&mut uid).unwrap();
    info!("unique id: {:?}", uid);

    erase_write_sector(&mut flash, 0x00);

    multiwrite_bytes(&mut flash, ERASE_SIZE as u32);

    background_read(&mut flash, (ERASE_SIZE * 2) as u32).await;

    loop {}
}

fn multiwrite_bytes(flash: &mut embassy_rp::flash::Flash<'_, FLASH, Async, FLASH_SIZE>, offset: u32) {
    info!(">>>> [multiwrite_bytes]");
    let mut read_buf = [0u8; ERASE_SIZE];
    defmt::unwrap!(flash.blocking_read(ADDR_OFFSET + offset, &mut read_buf));

    info!("Addr of flash block is {:x}", ADDR_OFFSET + offset + FLASH_BASE as u32);
    info!("Contents start with {=[u8]}", read_buf[0..4]);

    defmt::unwrap!(flash.blocking_erase(ADDR_OFFSET + offset, ADDR_OFFSET + offset + ERASE_SIZE as u32));

    defmt::unwrap!(flash.blocking_read(ADDR_OFFSET + offset, &mut read_buf));
    info!("Contents after erase starts with {=[u8]}", read_buf[0..4]);
    if read_buf.iter().any(|x| *x != 0xFF) {
        defmt::panic!("unexpected");
    }

    defmt::unwrap!(flash.blocking_write(ADDR_OFFSET + offset, &[0x01]));
    defmt::unwrap!(flash.blocking_write(ADDR_OFFSET + offset + 1, &[0x02]));
    defmt::unwrap!(flash.blocking_write(ADDR_OFFSET + offset + 2, &[0x03]));
    defmt::unwrap!(flash.blocking_write(ADDR_OFFSET + offset + 3, &[0x04]));

    defmt::unwrap!(flash.blocking_read(ADDR_OFFSET + offset, &mut read_buf));
    info!("Contents after write starts with {=[u8]}", read_buf[0..4]);
    if &read_buf[0..4] != &[0x01, 0x02, 0x03, 0x04] {
        defmt::panic!("unexpected");
    }
}

fn erase_write_sector(flash: &mut embassy_rp::flash::Flash<'_, FLASH, Async, FLASH_SIZE>, offset: u32) {
    info!(">>>> [erase_write_sector]");
    let mut buf = [0u8; ERASE_SIZE];
    defmt::unwrap!(flash.blocking_read(ADDR_OFFSET + offset, &mut buf));

    info!("Addr of flash block is {:x}", ADDR_OFFSET + offset + FLASH_BASE as u32);
    info!("Contents start with {=[u8]}", buf[0..4]);

    defmt::unwrap!(flash.blocking_erase(ADDR_OFFSET + offset, ADDR_OFFSET + offset + ERASE_SIZE as u32));

    defmt::unwrap!(flash.blocking_read(ADDR_OFFSET + offset, &mut buf));
    info!("Contents after erase starts with {=[u8]}", buf[0..4]);
    if buf.iter().any(|x| *x != 0xFF) {
        defmt::panic!("unexpected");
    }

    for b in buf.iter_mut() {
        *b = 0xDA;
    }

    defmt::unwrap!(flash.blocking_write(ADDR_OFFSET + offset, &buf));

    defmt::unwrap!(flash.blocking_read(ADDR_OFFSET + offset, &mut buf));
    info!("Contents after write starts with {=[u8]}", buf[0..4]);
    if buf.iter().any(|x| *x != 0xDA) {
        defmt::panic!("unexpected");
    }
}

async fn background_read(flash: &mut embassy_rp::flash::Flash<'_, FLASH, Async, FLASH_SIZE>, offset: u32) {
    info!(">>>> [background_read]");

    let mut buf = [0u32; 8];
    defmt::unwrap!(flash.background_read(ADDR_OFFSET + offset, &mut buf)).await;

    info!("Addr of flash block is {:x}", ADDR_OFFSET + offset + FLASH_BASE as u32);
    info!("Contents start with {=u32:x}", buf[0]);

    defmt::unwrap!(flash.blocking_erase(ADDR_OFFSET + offset, ADDR_OFFSET + offset + ERASE_SIZE as u32));

    defmt::unwrap!(flash.background_read(ADDR_OFFSET + offset, &mut buf)).await;
    info!("Contents after erase starts with {=u32:x}", buf[0]);
    if buf.iter().any(|x| *x != 0xFFFFFFFF) {
        defmt::panic!("unexpected");
    }

    for b in buf.iter_mut() {
        *b = 0xDABA1234;
    }

    defmt::unwrap!(flash.blocking_write(ADDR_OFFSET + offset, unsafe {
        core::slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len() * 4)
    }));

    defmt::unwrap!(flash.background_read(ADDR_OFFSET + offset, &mut buf)).await;
    info!("Contents after write starts with {=u32:x}", buf[0]);
    if buf.iter().any(|x| *x != 0xDABA1234) {
        defmt::panic!("unexpected");
    }
}
