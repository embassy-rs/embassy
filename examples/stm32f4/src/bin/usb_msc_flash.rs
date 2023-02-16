#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

use core::cell::RefCell;
use core::ops::Range;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::flash::{self, Flash};
use embassy_stm32::time::mhz;
use embassy_stm32::usb_otg::Driver;
use embassy_stm32::{interrupt, Config};
use embassy_usb::class::msc::subclass::scsi::block_device::{BlockDevice, BlockDeviceError};
use embassy_usb::class::msc::subclass::scsi::Scsi;
use embassy_usb::class::msc::transport::bulk_only::BulkOnlyTransport;
use embassy_usb::Builder;
use embedded_storage::nor_flash::RmwMultiwriteNorFlashStorage;
use embedded_storage::{ReadStorage, Storage};
use futures::future::join;
use {defmt_rtt as _, panic_probe as _};

// Ideally we would use 128K block size, which is the flash sector size of STM32,
// however, most operating systems only support 512 or 4096 byte blocks.
//
// To work around this limitation we must use RmwMultiwriteNorFlashStorage, which performs
// read-modify(-erase)-write operations on flash storage and optimises the number of erase
// operations.
//
// WARNING: this example is way too slow to
const BLOCK_SIZE: usize = 512;

struct FlashBlockDevice<'d> {
    flash: RefCell<RmwMultiwriteNorFlashStorage<'d, Flash<'d>>>,
    range: Range<usize>,
}

impl<'d> BlockDevice for FlashBlockDevice<'d> {
    fn status(&self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn block_size(&self) -> Result<usize, BlockDeviceError> {
        Ok(BLOCK_SIZE)
    }

    fn num_blocks(&self) -> Result<u32, BlockDeviceError> {
        Ok((self.range.len() / BLOCK_SIZE) as u32)
    }

    async fn read_block(&self, lba: u32, block: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.flash
            .borrow_mut()
            .read(self.range.start as u32 + (lba * BLOCK_SIZE as u32), block)
            .map_err(|_| BlockDeviceError::ReadError)?;
        Ok(())
    }

    async fn write_block(&mut self, lba: u32, block: &[u8]) -> Result<(), BlockDeviceError> {
        let mut flash = self.flash.borrow_mut();
        flash
            .write(self.range.start as u32 + (lba * BLOCK_SIZE as u32), block)
            .map_err(|_| BlockDeviceError::WriteError)?;
        Ok(())
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut config = Config::default();
    config.rcc.pll48 = true;
    config.rcc.sys_ck = Some(mhz(48));

    let p = embassy_stm32::init(config);

    // Create the driver, from the HAL.
    let irq = interrupt::take!(OTG_FS);
    let mut ep_out_buffer = [0u8; 256];
    let driver = Driver::new_fs(p.USB_OTG_FS, irq, p.PA12, p.PA11, &mut ep_out_buffer);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("MSC example");
    config.serial_number = Some("12345678");

    // Required for windows compatiblity.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = Default::default();

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
        None,
    );

    let mut flash_buffer = [0u8; flash::ERASE_SIZE];
    let flash = RefCell::new(RmwMultiwriteNorFlashStorage::new(
        Flash::new(p.FLASH),
        &mut flash_buffer,
    ));

    // Use upper 1MB of the 2MB flash
    let range = (1024 * 1024)..(2048 * 1024);

    let mut scsi_buffer = [0u8; BLOCK_SIZE];
    // Create SCSI target for our block device
    let scsi = Scsi::new(FlashBlockDevice { flash, range }, &mut scsi_buffer, "Embassy", "MSC");

    // Use bulk-only transport for our SCSI target
    let mut msc_transport = BulkOnlyTransport::new(&mut builder, &mut state, 64, scsi);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Run mass storage transport
    let msc_fut = msc_transport.run();

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, msc_fut).await;
}
