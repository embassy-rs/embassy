//! Expose part of the RP2040's flash as a USB mass-storage device.
//!
//! A `SectorCache` merges 512-byte host blocks into 4 KiB erase sectors.

#![no_std]
#![no_main]

use aligned::{A4, Aligned};
use block_device_driver::{BlockDevice, blocks_to_slice, blocks_to_slice_mut};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::flash::{ERASE_SIZE, Error, Flash};
use embassy_rp::mode::Async;
use embassy_rp::peripherals::{DMA_CH0, USB};
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_usb::Builder;
use embassy_usb::class::msc::{BlockDeviceAdapter, Config as MscConfig, MscClass, State};
use panic_probe as _;

const BLOCK_SIZE: u32 = 512;
const FLASH_SIZE: usize = 2 * 1024 * 1024;
// Upper megabyte of flash.
const STORAGE_OFFSET: u32 = 1024 * 1024;
const STORAGE_SIZE: u32 = 1024 * 1024;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>;
});

struct FlashBlockDevice<'d> {
    flash: Flash<'d, Async, FLASH_SIZE>,
}

impl BlockDevice<ERASE_SIZE> for FlashBlockDevice<'_> {
    type Error = Error;
    type Align = A4;

    async fn read(&mut self, lba: u32, blocks: &mut [Aligned<A4, [u8; ERASE_SIZE]>]) -> Result<(), Error> {
        let offset = STORAGE_OFFSET + lba * ERASE_SIZE as u32;
        self.flash.read(offset, blocks_to_slice_mut(blocks)).await
    }

    async fn write(&mut self, lba: u32, blocks: &[Aligned<A4, [u8; ERASE_SIZE]>]) -> Result<(), Error> {
        let offset = STORAGE_OFFSET + lba * ERASE_SIZE as u32;
        let bytes = blocks_to_slice(blocks);
        self.flash.blocking_erase(offset, offset + bytes.len() as u32)?;
        self.flash.blocking_write(offset, bytes)
    }

    async fn size(&mut self) -> Result<u64, Error> {
        Ok(STORAGE_SIZE as u64)
    }
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("MSC flash example");
    config.serial_number = Some(embassy_rp::uid::uid_hex());
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
        .product_id("Flash Disk")
        .product_revision_level("1.0")
        .serial_number(embassy_rp::uid::uid_hex());
    let mut msc = MscClass::new(&mut builder, &mut state, msc_config);
    let mut usb = builder.build();

    let flash = FlashBlockDevice {
        flash: Flash::new(p.FLASH, p.DMA_CH0, Irqs),
    };
    // Merge host blocks per sector, so each sector is written back at most once per flush.
    let mut cache = Aligned::<A4, _>([0; ERASE_SIZE]);
    let mut disk = unwrap!(BlockDeviceAdapter::new(flash).await).with_cache(&mut cache, BLOCK_SIZE);
    let mut block_buf = [0; BLOCK_SIZE as usize];

    info!("USB MSC flash disk running");
    join(usb.run(), msc.run(&mut disk, &mut block_buf)).await;
}
