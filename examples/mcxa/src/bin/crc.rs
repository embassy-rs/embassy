#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hal::config::Config;
use hal::crc::Crc;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let mut p = hal::init(config);

    defmt::info!("CRC example");

    let buf_u8 = [0x00u8, 0x11, 0x22, 0x33];
    let buf_u16 = [0x0000u16, 0x1111, 0x2222, 0x3333];
    let buf_u32 = [0x0000_0000u32, 0x1111_1111, 0x2222_2222, 0x3333_3333];

    // CCITT False

    let mut crc = Crc::new_ccitt_false(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u8);
    assert_eq!(sum, 0x9627);

    let mut crc = Crc::new_ccitt_false(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u16);
    assert_eq!(sum, 0xa467);

    let mut crc = Crc::new_ccitt_false(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u32);
    assert_eq!(sum, 0xe5c7);

    // Maxim

    let mut crc = Crc::new_maxim(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u8);
    assert_eq!(sum, 0x4ff7);

    let mut crc = Crc::new_maxim(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u16);
    assert_eq!(sum, 0x2afe);

    let mut crc = Crc::new_maxim(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u32);
    assert_eq!(sum, 0x17d7);

    // Kermit

    let mut crc = Crc::new_kermit(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u8);
    assert_eq!(sum, 0xccd2);

    let mut crc = Crc::new_kermit(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u16);
    assert_eq!(sum, 0x66eb);

    let mut crc = Crc::new_kermit(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u32);
    assert_eq!(sum, 0x75ea);

    // ISO HDLC

    let mut crc = Crc::new_iso_hdlc(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u8);
    assert_eq!(sum, 0x24c2_316d);

    let mut crc = Crc::new_iso_hdlc(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u16);
    assert_eq!(sum, 0x8a61_4178);

    let mut crc = Crc::new_iso_hdlc(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u32);
    assert_eq!(sum, 0xfab5_d04e);

    // POSIX

    let mut crc = Crc::new_posix(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u8);
    assert_eq!(sum, 0xba8d_7868);

    let mut crc = Crc::new_posix(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u16);
    assert_eq!(sum, 0x6d76_4f58);

    let mut crc = Crc::new_posix(p.CRC0.reborrow());
    let sum = crc.feed(&buf_u32);
    assert_eq!(sum, 0x2a5b_cb90);

    defmt::info!("CRC successful");
}
