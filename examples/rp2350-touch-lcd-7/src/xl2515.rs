//! XL2515 SPI CAN controller (MCP2515-compatible register map).
//!
//! Ported from Waveshare `bsp_xl2515.c`.

use defmt::info;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::SPI0;
use embassy_rp::spi::{Blocking, Config as SpiConfig, Spi};
use embassy_time::{Duration, Timer};

const CAN_WRITE: u8 = 0x02;
const CAN_READ: u8 = 0x03;
const CAN_RESET: u8 = 0xC0;

const CANCTRL: u8 = 0x0F;
const CANSTAT: u8 = 0x0E;
const CANINTE: u8 = 0x2B;
const CANINTF: u8 = 0x2C;
const CNF1: u8 = 0x2A;
const CNF2: u8 = 0x29;
const CNF3: u8 = 0x28;
const TXB0CTRL: u8 = 0x30;
const TXB0SIDH: u8 = 0x31;
const TXB0SIDL: u8 = 0x32;
const TXB0EID8: u8 = 0x33;
const TXB0EID0: u8 = 0x34;
const TXB0DLC: u8 = 0x35;
const TXB0D0: u8 = 0x36;
const RXB0SIDH: u8 = 0x61;
const RXB0SIDL: u8 = 0x62;
const RXB0CTRL: u8 = 0x60;
const RXB0DLC: u8 = 0x65;
const RXB0D0: u8 = 0x66;
const RXF0SIDH: u8 = 0x00;
const RXF0SIDL: u8 = 0x01;
const RXM0SIDH: u8 = 0x20;
const RXM0SIDL: u8 = 0x21;

const REQOP_NORMAL: u8 = 0x00;
const CLKOUT_ENABLED: u8 = 0x04;
const OPMODE_NORMAL: u8 = 0x00;
const DLC_8: u8 = 0x08;

/// Bitrate timing triples from Waveshare demo (`KBPS500` = index 7).
const RATE_TABLE: [[u8; 3]; 10] = [
    [0xBF, 0xFF, 0x87],
    [0x5F, 0xFF, 0x87],
    [0x18, 0xA4, 0x04],
    [0x09, 0xA4, 0x04],
    [0x04, 0x9E, 0x03],
    [0x03, 0x9E, 0x03],
    [0x01, 0x1E, 0x03],
    [0x00, 0x9E, 0x03],
    [0x00, 0x92, 0x02],
    [0x00, 0x82, 0x02],
];

pub struct CanSpi {
    spi: Spi<'static, SPI0, Blocking>,
    cs: Output<'static>,
}

impl CanSpi {
    pub fn new(
        spi0: embassy_rp::Peri<'static, SPI0>,
        sck: embassy_rp::Peri<'static, impl embassy_rp::spi::ClkPin<SPI0>>,
        mosi: embassy_rp::Peri<'static, impl embassy_rp::spi::MosiPin<SPI0>>,
        miso: embassy_rp::Peri<'static, impl embassy_rp::spi::MisoPin<SPI0>>,
        cs_pin: embassy_rp::Peri<'static, impl embassy_rp::gpio::Pin>,
    ) -> Self {
        let mut cfg = SpiConfig::default();
        cfg.frequency = 10_000_000;
        let spi = Spi::new_blocking(spi0, sck, mosi, miso, cfg);
        let cs = Output::new(cs_pin, Level::High);
        Self { spi, cs }
    }

    pub async fn init(&mut self, bitrate: u32) {
        self.reset();
        Timer::after(Duration::from_millis(100)).await;

        let idx = match bitrate {
            1_000_000 => 9,
            500_000 => 7,
            250_000 => 6,
            125_000 => 5,
            100_000 => 4,
            50_000 => 3,
            20_000 => 2,
            10_000 => 1,
            5_000 => 0,
            _ => 7, // 500 kbit/s if unknown
        };
        let timing = RATE_TABLE[idx];
        self.write_reg_byte(CNF1, timing[0]);
        self.write_reg_byte(CNF2, timing[1]);
        self.write_reg_byte(CNF3, timing[2]);

        self.write_reg_byte(TXB0SIDH, 0xFF);
        self.write_reg_byte(TXB0SIDL, 0xE0);
        self.write_reg_byte(TXB0DLC, 0x40 | DLC_8);

        self.write_reg_byte(RXB0SIDH, 0x00);
        self.write_reg_byte(RXB0SIDL, 0x60);
        self.write_reg_byte(RXB0CTRL, 0x00);
        self.write_reg_byte(RXB0DLC, DLC_8);

        self.write_reg_byte(RXF0SIDH, 0);
        self.write_reg_byte(RXF0SIDL, 0);
        self.write_reg_byte(RXM0SIDH, 0xFF);
        self.write_reg_byte(RXM0SIDL, 0xE0);

        self.write_reg_byte(CANINTF, 0x00);
        self.write_reg_byte(CANINTE, 0x01);
        self.write_reg_byte(CANCTRL, REQOP_NORMAL | CLKOUT_ENABLED);
        let stat = self.read_reg_byte(CANSTAT);
        if stat & 0xE0 != OPMODE_NORMAL {
            self.write_reg_byte(CANCTRL, REQOP_NORMAL | CLKOUT_ENABLED);
        }
        info!("XL2515 CAN ready @ {} bit/s", bitrate);
    }

    pub fn send_standard(&mut self, id: u16, data: &[u8]) {
        let mut delay = 0;
        while self.read_reg_byte(TXB0CTRL) & 0x08 != 0 && delay < 50 {
            delay += 1;
        }

        self.write_reg_byte(TXB0SIDH, (id >> 3) as u8);
        self.write_reg_byte(TXB0SIDL, ((id & 0x07) << 5) as u8);
        self.write_reg_byte(TXB0EID8, 0);
        self.write_reg_byte(TXB0EID0, 0);
        self.write_reg_byte(TXB0DLC, data.len().min(8) as u8);
        self.write_reg(TXB0D0, &data[..data.len().min(8)]);
        self.write_reg_byte(TXB0CTRL, 0x08);
    }

    pub fn try_receive(&mut self) -> Option<(u16, u8, [u8; 8])> {
        if self.read_reg_byte(CANINTF) & 0x01 == 0 {
            return None;
        }

        let sid_h = self.read_reg_byte(RXB0SIDH);
        let sid_l = self.read_reg_byte(RXB0SIDL);
        let id = u16::from(sid_h) << 3 | u16::from(sid_l >> 5);
        let len = self.read_reg_byte(RXB0DLC).min(8);
        let mut data = [0u8; 8];
        for (i, byte) in data.iter_mut().enumerate().take(len as usize) {
            *byte = self.read_reg_byte(RXB0D0 + i as u8);
        }

        self.write_reg_byte(CANINTF, 0);
        self.write_reg_byte(CANINTE, 0x01);
        self.write_reg_byte(RXB0SIDH, 0x00);
        self.write_reg_byte(RXB0SIDL, 0x60);
        Some((id, len, data))
    }

    fn reset(&mut self) {
        self.cs.set_low();
        let mut buf = [CAN_RESET];
        let _ = self.spi.blocking_transfer_in_place(&mut buf);
        self.cs.set_high();
    }

    fn write_reg_byte(&mut self, reg: u8, byte: u8) {
        self.cs.set_low();
        let tx = [CAN_WRITE, reg, byte];
        let mut rx = [0u8; 3];
        let _ = self.spi.blocking_transfer(&mut rx, &tx);
        self.cs.set_high();
    }

    fn write_reg(&mut self, reg: u8, data: &[u8]) {
        self.cs.set_low();
        let mut tx = [0u8; 10];
        tx[0] = CAN_WRITE;
        tx[1] = reg;
        tx[2..2 + data.len()].copy_from_slice(data);
        let mut rx = [0u8; 10];
        let _ = self.spi.blocking_transfer(&mut rx[..2 + data.len()], &tx[..2 + data.len()]);
        self.cs.set_high();
    }

    fn read_reg_byte(&mut self, reg: u8) -> u8 {
        self.cs.set_low();
        let tx = [CAN_READ, reg, 0];
        let mut rx = [0u8; 3];
        let _ = self.spi.blocking_transfer(&mut rx, &tx);
        self.cs.set_high();
        rx[2]
    }
}
