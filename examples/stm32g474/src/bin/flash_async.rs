#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::flash::{Flash, InterruptHandler};
use embassy_stm32::gpio::{AnyPin, Level, Output, Speed};
use embassy_stm32::{Peri, bind_interrupts};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    FLASH => InterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello Flash!");

    let mut f = Flash::new(p.FLASH, Irqs);

    // Led should blink uninterrupted during erase operation
    spawner.spawn(unwrap!(blinky(p.PA5.into())));

    // Test on bank 2 so the CPU doesn't stall (code runs from bank 1).
    // G474RE in dual-bank mode: bank 2 starts at 256KB offset.
    // Erase 4KB (2 pages at 2KB each in dual-bank mode).
    test_flash(&mut f, 256 * 1024, 4 * 1024).await;
}

#[embassy_executor::task]
async fn blinky(p: Peri<'static, AnyPin>) {
    let mut led = Output::new(p, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}

async fn test_flash(f: &mut Flash<'_>, offset: u32, size: u32) {
    info!("Testing offset: {=u32:#X}, size: {=u32:#X}", offset, size);

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(offset, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Erasing...");
    unwrap!(f.erase(offset, offset + size).await);

    info!("Reading...");
    let mut buf = [0u8; 32];
    unwrap!(f.blocking_read(offset, &mut buf));
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
    unwrap!(f.blocking_read(offset, &mut buf));
    info!("Read: {=[u8]:x}", buf);
    assert_eq!(
        &buf[..],
        &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32
        ]
    );

    info!("Flash async test passed!");
}
