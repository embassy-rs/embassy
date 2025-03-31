#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::flash::{Flash, InterruptHandler};
use embassy_stm32::gpio::{AnyPin, Level, Output, Speed};
use embassy_stm32::{bind_interrupts, Peri};
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

    // Led should blink uninterrupted during ~2sec erase operation
    spawner.spawn(blinky(p.PB7.into())).unwrap();

    // Test on bank 2 in order not to stall CPU.
    test_flash(&mut f, 1024 * 1024, 128 * 1024).await;
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

async fn test_flash<'a>(f: &mut Flash<'a>, offset: u32, size: u32) {
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
}
