#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::*;
use embassy_executor::Spawner;
use embassy_imxrt::crc::{Config, Crc, Polynomial};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_imxrt::init(Default::default());
    let data = b"123456789";

    info!("Initializing CRC");

    // CRC-CCITT
    let mut crc = Crc::new(p.CRC.reborrow(), Default::default());
    let output = crc.feed_bytes(data);
    defmt::assert_eq!(output, 0x29b1);

    // CRC16-ARC
    let mut crc = Crc::new(
        p.CRC.reborrow(),
        Config {
            polynomial: Polynomial::Crc16,
            reverse_in: true,
            reverse_out: true,
            complement_out: false,
            seed: 0,
            ..Default::default()
        },
    );
    let output = crc.feed_bytes(data);
    defmt::assert_eq!(output, 0xbb3d);

    // CRC16-CMS
    let mut crc = Crc::new(
        p.CRC.reborrow(),
        Config {
            polynomial: Polynomial::Crc16,
            reverse_in: false,
            reverse_out: false,
            complement_out: false,
            seed: 0xffff,
            ..Default::default()
        },
    );
    let output = crc.feed_bytes(data);
    defmt::assert_eq!(output, 0xaee7);

    // CRC16-DDS-110
    let mut crc = Crc::new(
        p.CRC.reborrow(),
        Config {
            polynomial: Polynomial::Crc16,
            reverse_in: false,
            reverse_out: false,
            complement_out: false,
            seed: 0x800d,
            ..Default::default()
        },
    );
    let output = crc.feed_bytes(data);
    defmt::assert_eq!(output, 0x9ecf);

    // CRC16-MAXIM-DOW
    let mut crc = Crc::new(
        p.CRC.reborrow(),
        Config {
            polynomial: Polynomial::Crc16,
            reverse_in: true,
            reverse_out: true,
            complement_out: true,
            seed: 0,
            ..Default::default()
        },
    );
    let output = crc.feed_bytes(data);
    defmt::assert_eq!(output, 0x44c2);

    // CRC16-MODBUS
    let mut crc = Crc::new(
        p.CRC.reborrow(),
        Config {
            polynomial: Polynomial::Crc16,
            reverse_in: true,
            reverse_out: true,
            complement_out: false,
            seed: 0xffff,
            ..Default::default()
        },
    );
    let output = crc.feed_bytes(data);
    defmt::assert_eq!(output, 0x4b37);

    // CRC32-BZIP2
    let mut crc = Crc::new(
        p.CRC.reborrow(),
        Config {
            polynomial: Polynomial::Crc32,
            reverse_in: false,
            reverse_out: false,
            complement_out: true,
            seed: 0xffff_ffff,
            ..Default::default()
        },
    );
    let output = crc.feed_bytes(data);
    defmt::assert_eq!(output, 0xfc89_1918);

    // CRC32-CKSUM
    let mut crc = Crc::new(
        p.CRC.reborrow(),
        Config {
            polynomial: Polynomial::Crc32,
            reverse_in: false,
            reverse_out: false,
            complement_out: true,
            seed: 0,
            ..Default::default()
        },
    );
    let output = crc.feed_bytes(data);
    defmt::assert_eq!(output, 0x765e_7680);

    // CRC32-ISO-HDLC
    let mut crc = Crc::new(
        p.CRC.reborrow(),
        Config {
            polynomial: Polynomial::Crc32,
            reverse_in: true,
            reverse_out: true,
            complement_out: true,
            seed: 0xffff_ffff,
            ..Default::default()
        },
    );
    let output = crc.feed_bytes(data);
    defmt::assert_eq!(output, 0xcbf4_3926);

    // CRC32-JAMCRC
    let mut crc = Crc::new(
        p.CRC.reborrow(),
        Config {
            polynomial: Polynomial::Crc32,
            reverse_in: true,
            reverse_out: true,
            complement_out: false,
            seed: 0xffff_ffff,
            ..Default::default()
        },
    );
    let output = crc.feed_bytes(data);
    defmt::assert_eq!(output, 0x340b_c6d9);

    // CRC32-MPEG-2
    let mut crc = Crc::new(
        p.CRC.reborrow(),
        Config {
            polynomial: Polynomial::Crc32,
            reverse_in: false,
            reverse_out: false,
            complement_out: false,
            seed: 0xffff_ffff,
            ..Default::default()
        },
    );
    let output = crc.feed_bytes(data);
    defmt::assert_eq!(output, 0x0376_e6e7);

    info!("end program");
    cortex_m::asm::bkpt();
}
