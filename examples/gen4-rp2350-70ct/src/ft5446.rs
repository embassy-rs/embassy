//! FocalTech FT5446 capacitive touch (I2C), from the gen4 Graphics4D port.

use defmt::info;
use embassy_rp::gpio::{Flex, Output, Pull};
use embassy_time::{Duration, Timer};

use crate::board::{BoardI2c, DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub const I2C_ADDR: u8 = 0x38;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
    pub pressed: bool,
    pub i2c_ok: bool,
}

pub async fn init(i2c: &mut BoardI2c, rst: &mut Output<'static>, int: &mut Flex<'static>) {
    int.set_pull(Pull::Up);
    int.set_as_input();

    rst.set_low();
    Timer::after(Duration::from_millis(100)).await;
    rst.set_high();
    Timer::after(Duration::from_millis(100)).await;

    // Keep active mode even when no touch event (Graphics4D FT5446 init).
    let _ = i2c.blocking_write(I2C_ADDR, &[0x86]);
    let _ = i2c.blocking_write(I2C_ADDR, &[0x00]);

    info!("FT5446 ready @ 0x{:02x}", I2C_ADDR);
}

pub fn read_touch(i2c: &mut BoardI2c) -> TouchPoint {
    let mut count = [0u8; 1];
    if i2c.blocking_write_read(I2C_ADDR, &[0x02], &mut count).is_err() {
        return TouchPoint {
            i2c_ok: false,
            ..Default::default()
        };
    }

    let mut points = count[0];
    if points == 0xff {
        points = 0;
    }
    points &= 0x0f;

    if points == 0 {
        return TouchPoint {
            i2c_ok: true,
            pressed: false,
            ..Default::default()
        };
    }

    let mut data = [0u8; 6];
    if i2c.blocking_write_read(I2C_ADDR, &[0x03], &mut data).is_err() {
        return TouchPoint {
            i2c_ok: false,
            ..Default::default()
        };
    }

    let mut x = u16::from(data[1]) << 8 | u16::from(data[0]);
    let mut y = u16::from(data[3]) << 8 | u16::from(data[2]);

    // LCD_TOUCH_SWAP_XY from gen4 board header.
    core::mem::swap(&mut x, &mut y);

    TouchPoint {
        x: x.min(DISPLAY_WIDTH as u16 - 1),
        y: y.min(DISPLAY_HEIGHT as u16 - 1),
        pressed: true,
        i2c_ok: true,
    }
}
