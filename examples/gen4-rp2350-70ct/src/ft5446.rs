//! FT5446 capacitive touch controller (FocalTech FT5x06 family, I2C).
//!
//! The gen4-RP2350-70CT wires the controller to I2C1 (SCL=GPIO39, SDA=GPIO46)
//! with INT=GPIO38 and RST=GPIO47. The panel is mounted in landscape, so the
//! native (portrait) coordinates are swapped to match the 800x480 framebuffer
//! (`LCD_TOUCH_SWAP_XY` in the vendor board header).

use defmt::{info, warn};
use embassy_rp::gpio::{Flex, Output, Pull};
use embassy_time::{Duration, Timer};

use crate::board::{BoardI2c, DISPLAY_HEIGHT, DISPLAY_WIDTH};

/// 7-bit I2C address of the FT5x06/FT5446 family.
pub const I2C_ADDR: u8 = 0x38;

const REG_TD_STATUS: u8 = 0x02;
const REG_CHIP_ID: u8 = 0xA3;

/// Swap X/Y to map the portrait-native panel onto the landscape framebuffer.
const SWAP_XY: bool = true;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
    pub pressed: bool,
    pub i2c_ok: bool,
}

/// Reset the controller and probe its chip id.
pub async fn init(i2c: &mut BoardI2c, rst: &mut Output<'static>, int: &mut Flex<'static>) {
    // Hardware reset (FocalTech datasheet: hold RST low >5 ms, release, wait ~300 ms).
    int.set_pull(Pull::Up);
    int.set_as_input();

    rst.set_high();
    Timer::after(Duration::from_millis(10)).await;
    rst.set_low();
    Timer::after(Duration::from_millis(20)).await;
    rst.set_high();
    Timer::after(Duration::from_millis(300)).await;

    let mut id = [0u8; 1];
    match i2c.blocking_write_read(I2C_ADDR, &[REG_CHIP_ID], &mut id) {
        Ok(()) => info!("FT5446 ready @ 0x{:02x}, chip id 0x{:02x}", I2C_ADDR, id[0]),
        Err(e) => warn!("FT5446 chip-id read failed: {:?}", e),
    }
}

/// Read a single touch point (first contact) from the controller.
pub fn read_touch(i2c: &mut BoardI2c) -> TouchPoint {
    // TD_STATUS + first touch register block (TD_STATUS, P1_XH, P1_XL, P1_YH, P1_YL).
    let mut buf = [0u8; 5];
    if i2c.blocking_write_read(I2C_ADDR, &[REG_TD_STATUS], &mut buf).is_err() {
        return TouchPoint {
            i2c_ok: false,
            ..Default::default()
        };
    }

    let touches = buf[0] & 0x0F;
    if touches == 0 {
        return TouchPoint {
            i2c_ok: true,
            pressed: false,
            ..Default::default()
        };
    }

    let raw_x = (u16::from(buf[1] & 0x0F) << 8) | u16::from(buf[2]);
    let raw_y = (u16::from(buf[3] & 0x0F) << 8) | u16::from(buf[4]);

    let (x, y) = if SWAP_XY { (raw_y, raw_x) } else { (raw_x, raw_y) };

    TouchPoint {
        x: x.min(DISPLAY_WIDTH as u16 - 1),
        y: y.min(DISPLAY_HEIGHT as u16 - 1),
        pressed: true,
        i2c_ok: true,
    }
}
