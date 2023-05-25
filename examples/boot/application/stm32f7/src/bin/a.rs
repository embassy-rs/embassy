#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[cfg(feature = "defmt-rtt")]
use defmt_rtt::*;
use embassy_boot_stm32::{AlignedBuffer, FirmwareUpdater};
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::flash::{Flash, WRITE_SIZE};
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use panic_reset as _;

static APP_B: &[u8] = include_bytes!("../../b.bin");

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut flash = Flash::new_blocking_only(p.FLASH);

    let button = Input::new(p.PC13, Pull::Down);
    let mut button = ExtiInput::new(button, p.EXTI13);

    let mut led = Output::new(p.PB7, Level::Low, Speed::Low);
    led.set_high();

    let mut updater = FirmwareUpdater::default();
    let mut writer = updater.prepare_update_blocking(&mut flash).unwrap();
    button.wait_for_rising_edge().await;
    let mut offset = 0;
    let mut buf = AlignedBuffer([0; 4096]);
    for chunk in APP_B.chunks(4096) {
        buf.as_mut()[..chunk.len()].copy_from_slice(chunk);
        writer
            .write_block_blocking(offset, buf.as_ref(), &mut flash, chunk.len())
            .unwrap();
        offset += chunk.len();
    }
    let mut magic = AlignedBuffer([0; WRITE_SIZE]);
    updater.mark_updated_blocking(&mut flash, magic.as_mut()).unwrap();
    led.set_low();
    cortex_m::peripheral::SCB::sys_reset();
}
