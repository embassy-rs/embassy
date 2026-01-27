#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use embassy_executor::Spawner;
use hal::config::Config;
use hal::crc::Crc;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const CCITT_FALSE: crc::Algorithm<u16> = crc::Algorithm {
    width: 16,
    poly: 0x1021,
    init: 0xffff,
    refin: false,
    refout: false,
    xorout: 0,
    check: 0x29b1,
    residue: 0x0000,
};

const POSIX: crc::Algorithm<u32> = crc::Algorithm {
    width: 32,
    poly: 0x04c1_1db7,
    init: 0,
    refin: false,
    refout: false,
    xorout: 0xffff_ffff,
    check: 0x765e_7680,
    residue: 0x0000,
};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let mut p = hal::init(config);

    defmt::info!("CRC example");

    let buf_u8 = [0x00u8, 0x11, 0x22, 0x33];
    let buf_u16 = [0x0000u16, 0x1111, 0x2222, 0x3333];
    let buf_u32 = [0x0000_0000u32, 0x1111_1111, 0x2222_2222, 0x3333_3333];

    // CCITT False

    let sw_crc = crc::Crc::<u16>::new(&CCITT_FALSE);
    let mut digest = sw_crc.digest();
    digest.update(&buf_u8);
    let sw_sum = digest.finalize();

    let mut crc = Crc::new_ccitt_false(p.CRC0.reborrow());
    crc.feed(&buf_u8);
    let sum = crc.finalize();
    assert_eq!(sum, sw_sum);

    let mut crc = Crc::new_ccitt_false(p.CRC0.reborrow());
    crc.feed(&buf_u16);
    let sum = crc.finalize();
    assert_eq!(sum, 0xa467);

    let mut crc = Crc::new_ccitt_false(p.CRC0.reborrow());
    crc.feed(&buf_u32);
    let sum = crc.finalize();
    assert_eq!(sum, 0xe5c7);

    // Maxim

    let sw_crc = crc::Crc::<u16>::new(&crc::CRC_16_MAXIM_DOW);
    let mut digest = sw_crc.digest();
    digest.update(&buf_u8);
    let sw_sum = digest.finalize();

    let mut crc = Crc::new_maxim(p.CRC0.reborrow());
    crc.feed(&buf_u8);
    let sum = crc.finalize();
    assert_eq!(sum, sw_sum);

    let mut crc = Crc::new_maxim(p.CRC0.reborrow());
    crc.feed(&buf_u16);
    let sum = crc.finalize();
    assert_eq!(sum, 0x2afe);

    let mut crc = Crc::new_maxim(p.CRC0.reborrow());
    crc.feed(&buf_u32);
    let sum = crc.finalize();
    assert_eq!(sum, 0x17d7);

    // Kermit

    let sw_crc = crc::Crc::<u16>::new(&crc::CRC_16_KERMIT);
    let mut digest = sw_crc.digest();
    digest.update(&buf_u8);
    let sw_sum = digest.finalize();

    let mut crc = Crc::new_kermit(p.CRC0.reborrow());
    crc.feed(&buf_u8);
    let sum = crc.finalize();
    assert_eq!(sum, sw_sum);

    let mut crc = Crc::new_kermit(p.CRC0.reborrow());
    crc.feed(&buf_u16);
    let sum = crc.finalize();
    assert_eq!(sum, 0x66eb);

    let mut crc = Crc::new_kermit(p.CRC0.reborrow());
    crc.feed(&buf_u32);
    let sum = crc.finalize();
    assert_eq!(sum, 0x75ea);

    // ISO HDLC

    let sw_crc = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
    let mut digest = sw_crc.digest();
    digest.update(&buf_u8);
    let sw_sum = digest.finalize();

    let mut crc = Crc::new_iso_hdlc(p.CRC0.reborrow());
    crc.feed(&buf_u8);
    let sum = crc.finalize();
    assert_eq!(sum, sw_sum);

    let mut crc = Crc::new_iso_hdlc(p.CRC0.reborrow());
    crc.feed(&buf_u16);
    let sum = crc.finalize();
    assert_eq!(sum, 0x8a61_4178);

    let mut crc = Crc::new_iso_hdlc(p.CRC0.reborrow());
    crc.feed(&buf_u32);
    let sum = crc.finalize();
    assert_eq!(sum, 0xfab5_d04e);

    // POSIX

    let sw_crc = crc::Crc::<u32>::new(&POSIX);
    let mut digest = sw_crc.digest();
    digest.update(&buf_u8);
    let sw_sum = digest.finalize();

    let mut crc = Crc::new_posix(p.CRC0.reborrow());
    crc.feed(&buf_u8);
    let sum = crc.finalize();

    assert_eq!(sum, sw_sum);

    let mut crc = Crc::new_posix(p.CRC0.reborrow());
    crc.feed(&buf_u16);
    let sum = crc.finalize();
    assert_eq!(sum, 0x6d76_4f58);

    let mut crc = Crc::new_posix(p.CRC0.reborrow());
    crc.feed(&buf_u32);
    let sum = crc.finalize();
    assert_eq!(sum, 0x2a5b_cb90);

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
