#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap};
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::flash::Flash;
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Speed};
use embassy_stm32::Peripherals;
use embedded_storage_async::nor_flash::{AsyncNorFlash, AsyncReadNorFlash};
use {defmt_rtt as _, panic_probe as _};

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    info!("Hello Flash!");

    let mut f = Flash::unlock(p.FLASH);

    // Led should blink uninterrupted during ~2sec erase operation
    spawner.spawn(blinky(p.PB7.degrade())).unwrap();

    // Test on bank 2 in order not to stall CPU.
    test_flash(&mut f, 1024 * 1024, 128 * 1024).await;
}

#[embassy::task]
async fn blinky(p: AnyPin) {
    let mut led = Output::new(p, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after(Duration::from_millis(300)).await;

        info!("low");
        led.set_low();
        Timer::after(Duration::from_millis(300)).await;
    }
}

async fn test_flash<'a>(f: &mut Flash<'a>, offset: u32, size: u32) {
    info!("Testing offset: {=u32:#X}, size: {=u32:#X}", offset, size);

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.read(offset, &mut buf).await);
    info!("Read: {=[u8]:x}", buf);

    info!("Erasing...");
    unwrap!(f.erase(offset, offset + size).await);

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.read(offset, &mut buf).await);
    info!("Read after erase: {=[u8]:x}", buf);

    info!("Writing...");
    unwrap!(
        f.write(
            offset,
            &[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
                29, 30, 31, 32
            ]
        )
        .await
    );

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.read(offset, &mut buf).await);
    info!("Read: {=[u8]:x}", buf);
    assert_eq!(
        &buf[..],
        &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32
        ]
    );
}
