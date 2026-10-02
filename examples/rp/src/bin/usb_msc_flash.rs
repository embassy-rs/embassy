//! Expose part of the RP2040's flash as a USB mass-storage device.

#![no_std]
#![no_main]

use core::ops::Range;

use aligned::{A4, Aligned};
use block_device_driver::{BlockDevice, blocks_to_slice, blocks_to_slice_mut};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::flash::{ERASE_SIZE, Flash};
use embassy_rp::mode::Async;
use embassy_rp::peripherals::{DMA_CH0, USB};
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_usb::Builder;
use embassy_usb::class::msc::subclass::scsi::Scsi;
use embassy_usb::class::msc::subclass::scsi::block_device::{BlockDeviceAdapter, BlockDeviceError};
use embassy_usb::class::msc::transport::bulk_only::BulkOnlyTransport;
use embedded_storage::nor_flash::RmwMultiwriteNorFlashStorage;
use embedded_storage::{ReadStorage, Storage};
use panic_probe as _;

const BLOCK_SIZE: usize = 512;
const FLASH_SIZE: usize = 2 * 1024 * 1024;
const STORAGE_RANGE: Range<usize> = (1024 * 1024)..FLASH_SIZE;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>;
});

struct FlashBlockDevice<'d> {
    flash: RmwMultiwriteNorFlashStorage<'d, Flash<'d, Async, FLASH_SIZE>>,
}

impl BlockDevice<BLOCK_SIZE> for FlashBlockDevice<'_> {
    type Error = BlockDeviceError;
    type Align = A4;

    async fn read(&mut self, lba: u32, blocks: &mut [Aligned<A4, [u8; BLOCK_SIZE]>]) -> Result<(), Self::Error> {
        self.flash
            .read(
                STORAGE_RANGE.start as u32 + lba * BLOCK_SIZE as u32,
                blocks_to_slice_mut(blocks),
            )
            .map_err(|_| BlockDeviceError::ReadError)
    }

    async fn write(&mut self, lba: u32, blocks: &[Aligned<A4, [u8; BLOCK_SIZE]>]) -> Result<(), Self::Error> {
        self.flash
            .write(
                STORAGE_RANGE.start as u32 + lba * BLOCK_SIZE as u32,
                blocks_to_slice(blocks),
            )
            .map_err(|_| BlockDeviceError::WriteError)
    }

    async fn size(&mut self) -> Result<u64, Self::Error> {
        Ok(STORAGE_RANGE.len() as u64)
    }
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);
    let flash = Flash::<Async, FLASH_SIZE>::new(p.FLASH, p.DMA_CH0, Irqs);

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("MSC flash example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut state = Default::default();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    let mut flash_buffer = [0; ERASE_SIZE];
    let flash = RmwMultiwriteNorFlashStorage::new(flash, &mut flash_buffer);
    // One erase sector per chunk, so a sector-aligned write costs one erase instead of one per block
    let mut scsi_buffer = [Aligned::<A4, _>([0; BLOCK_SIZE]); ERASE_SIZE / BLOCK_SIZE];
    let scsi = Scsi::new(
        BlockDeviceAdapter::new(FlashBlockDevice { flash }),
        blocks_to_slice_mut(&mut scsi_buffer),
        "Embassy",
        "MSC",
    );
    let mut msc = BulkOnlyTransport::new(&mut builder, &mut state, 64, scsi);
    let mut usb = builder.build();

    info!("USB MSC flash disk running");
    join(usb.run(), msc.run()).await;
}
