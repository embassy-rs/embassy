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

    let buf = b"123456789";

    let mut crc = Crc::new_ccitt_false(p.CRC0.reborrow());
    let sum = crc.feed_bytes(buf);
    assert_eq!(sum, 0x29b1);

    let mut crc = Crc::new_maxim(p.CRC0.reborrow());
    let sum = crc.feed_bytes(buf);
    assert_eq!(sum, 0x44c2);

    let mut crc = Crc::new_kermit(p.CRC0.reborrow());
    let sum = crc.feed_bytes(buf);
    assert_eq!(sum, 0x2189);

    let mut crc = Crc::new_iso_hdlc(p.CRC0.reborrow());
    let sum = crc.feed_bytes(buf);
    assert_eq!(sum, 0xcbf4_3926);

    let mut crc = Crc::new_posix(p.CRC0.reborrow());
    let sum = crc.feed_bytes(buf);
    assert_eq!(sum, 0x765e_7680);

    defmt::info!("CRC successful");
}
