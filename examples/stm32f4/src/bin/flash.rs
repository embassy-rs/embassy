#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::flash::{Blocking, Flash};
use {defmt_rtt as _, panic_probe as _};

/// This example is written for the nucleo-stm32f429zi, with a stm32f429zi chip.
///
/// If you are using a different board or chip, make sure you update the following:
///
/// * [ ] Update .cargo/config.toml with the correct `probe-rs run --chip STM32F429ZITx`chip name.
/// * [ ] Update Cargo.toml to have the correct `embassy-stm32` feature, it is
///       currently `stm32f429zi`.
/// * [ ] If your board has a special clock or power configuration, make sure that it is
///       set up appropriately.
/// * [ ] If your board has different pin mapping, update any pin numbers or peripherals
///       to match your schematic
///
/// If you are unsure, please drop by the Embassy Matrix chat for support, and let us know:
///
/// * Which example you are trying to run
/// * Which chip and board you are using
///
/// Embassy Chat: https://matrix.to/#/#embassy-rs:matrix.org
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello Flash!");

    // Once can also call `into_regions()` to get access to NorFlash implementations
    // for each of the unique characteristics.
    let mut f = Flash::new_blocking(p.FLASH);

    // Sector 5
    test_flash(&mut f, 128 * 1024, 128 * 1024);

    // Sectors 11..=16, across banks (128K, 16K, 16K, 16K, 16K, 64K)
    test_flash(&mut f, (1024 - 128) * 1024, 256 * 1024);

    // Sectors 23, last in bank 2
    test_flash(&mut f, (2048 - 128) * 1024, 128 * 1024);
}

fn test_flash(f: &mut Flash<'_, Blocking>, offset: u32, size: u32) {
    info!("Testing offset: {=u32:#X}, size: {=u32:#X}", offset, size);

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(offset, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Erasing...");
    unwrap!(f.blocking_erase(offset, offset + size));

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(offset, &mut buf));
    info!("Read after erase: {=[u8]:x}", buf);

    info!("Writing...");
    unwrap!(f.blocking_write(
        offset,
        &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32
        ]
    ));

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(offset, &mut buf));
    info!("Read: {=[u8]:x}", buf);
    assert_eq!(
        &buf[..],
        &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32
        ]
    );
}
