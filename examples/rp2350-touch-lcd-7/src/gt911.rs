//! GT911 capacitive touch controller (I2C).

use defmt::{info, warn};
use embassy_rp::gpio::{Flex, Output, Pull};
use embassy_time::{Duration, Timer};

use crate::board::{BoardI2c, DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub const I2C_ADDR_5D: u8 = 0x5D;
pub const I2C_ADDR_14: u8 = 0x14;
const REG_PRODUCT_ID: u16 = 0x8140;
const REG_STATUS: u16 = 0x814E;

static mut ACTIVE_ADDR: u8 = I2C_ADDR_5D;

fn active_addr() -> u8 {
    unsafe { ACTIVE_ADDR }
}

fn set_active_addr(addr: u8) {
    unsafe { ACTIVE_ADDR = addr };
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
    pub pressed: bool,
    pub i2c_ok: bool,
}

pub async fn init(i2c: &mut BoardI2c, rst: &mut Output<'static>, int: &mut Flex<'static>) {
    // Match Waveshare `bsp_gt911_init`: INT driven low during reset → address 0x5D.
    int.set_pull(Pull::None);
    int.set_low();
    int.set_as_output();

    rst.set_high();
    Timer::after(Duration::from_millis(50)).await;
    rst.set_low();
    Timer::after(Duration::from_millis(50)).await;
    rst.set_high();
    Timer::after(Duration::from_millis(250)).await;

    if let Some((addr, id)) = probe_product_id(i2c, I2C_ADDR_5D).await {
        set_active_addr(addr);
        int.set_as_input();
        int.set_pull(Pull::Up);
        info!("GT911 ready @ 0x{:02x}, product id: {:a}", addr, &id);
        return;
    }

    warn!("GT911 not at 0x{:02x}, retrying reset with INT high (0x{:02x})", I2C_ADDR_5D, I2C_ADDR_14);
    int.set_pull(Pull::None);
    int.set_high();
    int.set_as_output();

    rst.set_high();
    Timer::after(Duration::from_millis(50)).await;
    rst.set_low();
    Timer::after(Duration::from_millis(50)).await;
    rst.set_high();
    Timer::after(Duration::from_millis(250)).await;

    if let Some((addr, id)) = probe_product_id(i2c, I2C_ADDR_14).await {
        set_active_addr(addr);
        int.set_as_input();
        int.set_pull(Pull::Up);
        info!("GT911 ready @ 0x{:02x}, product id: {:a}", addr, &id);
        return;
    }

    warn!("GT911 not detected on I2C (tried 0x{:02x} and 0x{:02x})", I2C_ADDR_5D, I2C_ADDR_14);
}

async fn probe_product_id(i2c: &mut BoardI2c, addr: u8) -> Option<(u8, [u8; 4])> {
    let mut id = [0u8; 4];
    for attempt in 0..10 {
        match i2c.blocking_write_read(addr, &reg16_be(REG_PRODUCT_ID), &mut id) {
            Ok(()) if id[0] == b'9' && id[1] == b'1' && id[2] == b'1' => return Some((addr, id)),
            Ok(()) => warn!(
                "GT911 @ 0x{:02x} unexpected id attempt {}: {:02x}{:02x}{:02x}{:02x}",
                addr, attempt, id[0], id[1], id[2], id[3]
            ),
            Err(e) => warn!("GT911 @ 0x{:02x} id read failed attempt {}: {:?}", addr, attempt, e),
        }
        Timer::after(Duration::from_millis(100)).await;
    }
    None
}

pub fn read_touch(i2c: &mut BoardI2c) -> TouchPoint {
    let addr = active_addr();
    let mut status = [0u8; 1];
    if i2c
        .blocking_write_read(addr, &reg16_be(REG_STATUS), &mut status)
        .is_err()
    {
        return TouchPoint {
            i2c_ok: false,
            ..Default::default()
        };
    }

    let _ = i2c.blocking_write(addr, &write_reg16_be(REG_STATUS, 0));

    if status[0] & 0x80 == 0 {
        return TouchPoint {
            i2c_ok: true,
            pressed: false,
            ..Default::default()
        };
    }

    let mut data = [0u8; 8];
    if i2c
        .blocking_write_read(addr, &reg16_be(REG_STATUS + 1), &mut data)
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
