//! GT911 capacitive touch controller (I2C).

use defmt::{debug, warn};
use embassy_rp::gpio::Output;
use embassy_time::{Duration, Timer};

use crate::board::{BoardI2c, DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub const I2C_ADDR: u8 = 0x5D;
const REG_PRODUCT_ID: u16 = 0x8140;
const REG_STATUS: u16 = 0x814E;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
    pub pressed: bool,
    pub i2c_ok: bool,
}

pub async fn init(i2c: &mut BoardI2c, rst: &mut Output<'static>) {
    rst.set_high();
    Timer::after(Duration::from_millis(50)).await;
    rst.set_low();
    Timer::after(Duration::from_millis(50)).await;
    rst.set_high();
    Timer::after(Duration::from_millis(250)).await;

    let mut id = [0u8; 4];
    for attempt in 0..10 {
        match i2c.blocking_write_read(I2C_ADDR, &reg16_be(REG_PRODUCT_ID), &mut id) {
            Ok(()) if id[0] == b'9' && id[1] == b'1' && id[2] == b'1' => {
                debug!("GT911 product id: {:a}", &id[..4]);
                return;
            }
            Ok(()) => warn!(
                "GT911 unexpected id attempt {}: {:02x}{:02x}{:02x}{:02x}",
                attempt, id[0], id[1], id[2], id[3]
            ),
            Err(e) => warn!("GT911 id read failed attempt {}: {:?}", attempt, e),
        }
        Timer::after(Duration::from_millis(100)).await;
    }
    warn!("GT911 not detected on I2C @ 0x{:02x}", I2C_ADDR);
}

pub fn read_touch(i2c: &mut BoardI2c) -> TouchPoint {
    let mut status = [0u8; 1];
    if i2c
        .blocking_write_read(I2C_ADDR, &reg16_be(REG_STATUS), &mut status)
        .is_err()
    {
        return TouchPoint {
            i2c_ok: false,
            ..Default::default()
        };
    }

    let _ = i2c.blocking_write(I2C_ADDR, &write_reg16_be(REG_STATUS, 0));

    if status[0] & 0x80 == 0 {
        return TouchPoint {
            i2c_ok: true,
            pressed: false,
            ..Default::default()
        };
    }

    let mut data = [0u8; 8];
    if i2c
        .blocking_write_read(I2C_ADDR, &reg16_be(REG_STATUS + 1), &mut data)
        .is_err()
    {
        return TouchPoint {
            i2c_ok: false,
            ..Default::default()
        };
    }

    let x = (u16::from(data[1]) << 8 | u16::from(data[0])).min(DISPLAY_WIDTH as u16 - 1);
    let y = (u16::from(data[3]) << 8 | u16::from(data[2])).min(DISPLAY_HEIGHT as u16 - 1);

    TouchPoint {
        x,
        y,
        pressed: true,
        i2c_ok: true,
    }
}

fn reg16_be(reg: u16) -> [u8; 2] {
    reg.to_be_bytes()
}

fn write_reg16_be(reg: u16, byte: u8) -> [u8; 3] {
    [reg.to_be_bytes()[0], reg.to_be_bytes()[1], byte]
}
