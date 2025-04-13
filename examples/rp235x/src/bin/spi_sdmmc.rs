//! This example shows how to use `embedded-sdmmc` with the RP235x chip, over SPI.
//!
//! The example will attempt to read a file `MY_FILE.TXT` from the root directory
//! of the SD card and print its contents.

#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::SetConfig;
use embassy_executor::Spawner;
use embassy_rp::spi::Spi;
use embassy_rp::{gpio, spi};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::sdcard::{DummyCsPin, SdCard};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

struct DummyTimesource();

impl embedded_sdmmc::TimeSource for DummyTimesource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // SPI clock needs to be running at <= 400kHz during initialization
    let mut config = spi::Config::default();
    config.frequency = 400_000;
    let spi = Spi::new_blocking(p.SPI1, p.PIN_10, p.PIN_11, p.PIN_12, config);
    // Use a dummy cs pin here, for embedded-hal SpiDevice compatibility reasons
    let spi_dev = ExclusiveDevice::new_no_delay(spi, DummyCsPin);
    // Real cs pin
    let cs = Output::new(p.PIN_16, Level::High);

    let sdcard = SdCard::new(spi_dev, cs, embassy_time::Delay);
    info!("Card size is {} bytes", sdcard.num_bytes().unwrap());

    // Now that the card is initialized, the SPI clock can go faster
    let mut config = spi::Config::default();
    config.frequency = 16_000_000;
    sdcard.spi(|dev| SetConfig::set_config(dev.bus_mut(), &config)).ok();

    // Now let's look for volumes (also known as partitions) on our block device.
    // To do this we need a Volume Manager. It will take ownership of the block device.
    let mut volume_mgr = embedded_sdmmc::VolumeManager::new(sdcard, DummyTimesource());

    // Try and access Volume 0 (i.e. the first partition).
    // The volume object holds information about the filesystem on that volume.
    let mut volume0 = volume_mgr.open_volume(embedded_sdmmc::VolumeIdx(0)).unwrap();
    info!("Volume 0: {:?}", defmt::Debug2Format(&volume0));

    // Open the root directory (mutably borrows from the volume).
    let mut root_dir = volume0.open_root_dir().unwrap();

    // Open a file called "MY_FILE.TXT" in the root directory
    // This mutably borrows the directory.
    let mut my_file = root_dir
        .open_file_in_dir("MY_FILE.TXT", embedded_sdmmc::Mode::ReadOnly)
        .unwrap();

    // Print the contents of the file
    while !my_file.is_eof() {
        let mut buf = [0u8; 32];
        if let Ok(n) = my_file.read(&mut buf) {
            info!("{:a}", buf[..n]);
        }
    }

    loop {}
}
