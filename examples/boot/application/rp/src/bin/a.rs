#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[cfg(feature = "defmt-rtt")]
use defmt_rtt::*;
use embassy_boot_rp::{AlignedBuffer, FirmwareUpdater};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_executor::Spawner;
use embassy_rp::flash::{Flash, ERASE_SIZE, WRITE_SIZE};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use panic_reset as _;

static APP_B: &[u8] = include_bytes!("../../b.bin");

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let flash = Flash::<_, { 2 * 1024 * 1024 }>::new(p.FLASH);
    let mut flash = BlockingAsync::new(flash);

    let mut button = Input::new(p.PIN_28, Pull::Up);
    let mut led = Output::new(p.PIN_25, Level::Low);
    led.set_high();

    let mut updater = FirmwareUpdater::default();
    button.wait_for_falling_edge().await;
    let mut offset = 0;
    for chunk in APP_B.chunks(ERASE_SIZE) {
        let mut buf: [u8; ERASE_SIZE] = [0; ERASE_SIZE];
        buf[..chunk.len()].copy_from_slice(chunk);
        updater
            .write_firmware(offset, &buf, &mut flash, ERASE_SIZE)
            .await
            .unwrap();
        offset += chunk.len();
    }
    let mut magic = AlignedBuffer([0; WRITE_SIZE]);
    updater.mark_updated(&mut flash, magic.as_mut()).await.unwrap();
    led.set_low();
    cortex_m::peripheral::SCB::sys_reset();
}
