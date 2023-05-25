#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[cfg(feature = "defmt-rtt")]
use defmt_rtt::*;
use embassy_boot_stm32::{AlignedBuffer, FirmwareUpdater};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::flash::{Flash, WRITE_SIZE};
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use panic_reset as _;

static APP_B: &[u8] = include_bytes!("../../b.bin");

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let flash = Flash::new_blocking_only(p.FLASH);
    let mut flash = BlockingAsync::new(flash);

    let button = Input::new(p.PA0, Pull::Up);
    let mut button = ExtiInput::new(button, p.EXTI0);

    let mut led = Output::new(p.PB9, Level::Low, Speed::Low);
    led.set_high();

    let mut updater = FirmwareUpdater::default();
    button.wait_for_falling_edge().await;
    //defmt::info!("Starting update");
    let mut offset = 0;
    for chunk in APP_B.chunks(2048) {
        let mut buf: [u8; 2048] = [0; 2048];
        buf[..chunk.len()].copy_from_slice(chunk);
        //        defmt::info!("Writing chunk at 0x{:x}", offset);
        updater.write_firmware(offset, &buf, &mut flash, 2048).await.unwrap();
        offset += chunk.len();
    }
    let mut magic = AlignedBuffer([0; WRITE_SIZE]);
    updater.mark_updated(&mut flash, magic.as_mut()).await.unwrap();
    //defmt::info!("Marked as updated");
    led.set_low();
    cortex_m::peripheral::SCB::sys_reset();
}
