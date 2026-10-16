//! USB MSC device backed by the microSD socket on the Waveshare RP2350-PiZero.
//!
//! Insert a card before starting the example. The host sees the entire card,
//! including its partition table, and may read and write it. Do not access the
//! card from the firmware at the same time as the USB host.
//!
//! Build with `cargo build --release --bin usb_msc_sdcard --no-default-features --features rp235xb`.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_embedded_hal::SetConfig;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::USB;
use embassy_rp::spi::{self, Spi};
use embassy_rp::time::Hertz;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_usb::Builder;
use embassy_usb::class::msc::device::{BlockDevice as MscBlockDevice, Config as MscConfig, MscClass, State};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::sdcard::{DummyCsPin, SdCard};
use embedded_sdmmc::{Block, BlockDevice as SdBlockDevice, BlockIdx};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

struct SdDisk<C> {
    card: C,
    block: Block,
    block_count: u32,
}

impl<C: SdBlockDevice> MscBlockDevice for SdDisk<C> {
    type Error = C::Error;

    fn block_size(&self) -> u32 {
        Block::LEN_U32
    }

    fn block_count(&self) -> u32 {
        self.block_count
    }

    fn read_blocks(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        for (i, chunk) in buf.chunks_exact_mut(Block::LEN).enumerate() {
            let idx = BlockIdx(lba + i as u32);
            self.card.read(core::slice::from_mut(&mut self.block), idx, "USB MSC")?;
            chunk.copy_from_slice(&self.block.contents);
        }
        Ok(())
    }

    fn write_blocks(&mut self, lba: u32, data: &[u8]) -> Result<(), Self::Error> {
        for (i, chunk) in data.chunks_exact(Block::LEN).enumerate() {
            self.block.contents.copy_from_slice(chunk);
            self.card
                .write(core::slice::from_ref(&self.block), BlockIdx(lba + i as u32))?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // embedded-sdmmc completes each write before returning.
        Ok(())
    }
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Waveshare RP2350-PiZero: SCK=GPIO30, MOSI=GPIO31, MISO=GPIO40, CS=GPIO43.
    let mut spi_config = spi::Config::default();
    spi_config.frequency = Hertz(400_000);
    let spi = Spi::new_blocking(p.SPI1, p.PIN_30, p.PIN_31, p.PIN_40, spi_config).unwrap();
    let spi_dev = ExclusiveDevice::new_no_delay(spi, DummyCsPin);
    let cs = Output::new(p.PIN_43, Level::High);
    let card = SdCard::new(spi_dev, cs, embassy_time::Delay);
    let block_count = card.num_blocks().unwrap().0;
    info!("SD card: {} blocks", block_count);

    spi_config.frequency = Hertz(16_000_000);
    card.spi(|dev| SetConfig::set_config(dev.bus_mut(), &spi_config))
        .unwrap();

    let mut disk = SdDisk {
        card,
        block: Block::new(),
        block_count,
    };

    // Use the RP2350's native USB connector, not the PIO-USB connector.
    let driver = Driver::new(p.USB, Irqs);
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("RP2350-PiZero SD card");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut state = State::new();
    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    let msc_config = MscConfig::new(64)
        .vendor_id("Embassy")
        .product_id("SD Card")
        .product_revision_level("1.0")
        .serial_number("12345678");
    let mut msc = MscClass::new(&mut builder, &mut state, msc_config);
    let mut usb = builder.build();
    let mut block_buf = [0u8; Block::LEN];

    info!("USB MSC SD card started");
    join(usb.run(), msc.run(&mut disk, &mut block_buf)).await;
}
