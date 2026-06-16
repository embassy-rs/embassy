#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp2350_touch_lcd_7_examples::board;
use embassy_rp2350_touch_lcd_7_examples::usb_monitor;
use embassy_time::{Duration, Timer};
use {panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = board::init();
    usb_monitor::spawn(&spawner, p.USB);
    Timer::after_millis(200).await;
    board::log_board_info();

    let mut i2c = board::init_i2c(p.I2C1, p.PIN_7, p.PIN_6);
    let mut touch_pins = board::init_touch_pins(p.PIN_19, p.PIN_18);
    board::init_gt911(&mut i2c, &mut touch_pins).await;

    info!("GT911 touch demo — poll loop");

    loop {
        let t = board::read_touch(&mut i2c);
        if t.pressed {
            info!("touch @ ({}, {})", t.x, t.y);
        }
        Timer::after(Duration::from_millis(20)).await;
    }
}
