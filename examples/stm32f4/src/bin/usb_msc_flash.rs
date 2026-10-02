#![no_std]
#![no_main]
use core::cell::RefCell;
use core::ops::Range;

use aligned::{A4, Aligned};
use block_device_driver::{BlockDevice, blocks_to_slice, blocks_to_slice_mut};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::flash::{Flash, MAX_ERASE_SIZE};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::msc::subclass::scsi::Scsi;
use embassy_usb::class::msc::subclass::scsi::block_device::{BlockDeviceAdapter, BlockDeviceError};
use embassy_usb::class::msc::transport::bulk_only::BulkOnlyTransport;
use embedded_storage::nor_flash::RmwMultiwriteNorFlashStorage;
use embedded_storage::{ReadStorage, Storage};
use panic_probe as _;

// Ideally we would use 128K block size, which is the flash sector size of STM32,
// however, most operating systems only support 512 or 4096 byte blocks.
//
// To work around this limitation we must use RmwMultiwriteNorFlashStorage, which performs
// read-modify(-erase)-write operations on flash storage and optimises the number of erase
// operations.
//
// WARNING: this example is way too slow to
const BLOCK_SIZE: usize = 512;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
    FLASH => embassy_stm32::flash::InterruptHandler;
});

struct FlashBlockDevice<'d> {
    flash: RefCell<RmwMultiwriteNorFlashStorage<'d, Flash<'d>>>,
    range: Range<usize>,
}

impl<'d> BlockDevice<BLOCK_SIZE> for FlashBlockDevice<'d> {
    type Error = BlockDeviceError;
    type Align = A4;

    async fn size(&mut self) -> Result<u64, BlockDeviceError> {
        Ok(self.range.len() as u64)
    }

    async fn read(&mut self, lba: u32, blocks: &mut [Aligned<A4, [u8; BLOCK_SIZE]>]) -> Result<(), BlockDeviceError> {
        self.flash
            .borrow_mut()
            .read(
                self.range.start as u32 + (lba * BLOCK_SIZE as u32),
                blocks_to_slice_mut(blocks),
            )
            .map_err(|_| BlockDeviceError::ReadError)?;
        Ok(())
    }

    async fn write(&mut self, lba: u32, blocks: &[Aligned<A4, [u8; BLOCK_SIZE]>]) -> Result<(), BlockDeviceError> {
        let mut flash = self.flash.borrow_mut();
        flash
            .write(
                self.range.start as u32 + (lba * BLOCK_SIZE as u32),
                blocks_to_slice(blocks),
            )
            .map_err(|_| BlockDeviceError::WriteError)?;
        Ok(())
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::Hse;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul168,
            divp: Some(PllPDiv::Div2),
            divq: Some(PllQDiv::Div7),
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::Div1;
        config.rcc.apb1_pre = APBPrescaler::Div4;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.mux.clk48sel = mux::Clk48sel::Pll1Q;
    }

    let p = embassy_stm32::init(config);

    // Create the driver, from the HAL.
    let mut ep_out_buffer = [0u8; 256];
    let mut config = embassy_stm32::usb::Config::default();
    config.vbus_detection = false;
    let driver = Driver::new_fs(p.USB_OTG_FS, p.PA12, p.PA11, Irqs, &mut ep_out_buffer, config);

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
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = Default::default();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    let mut flash_buffer = [0u8; MAX_ERASE_SIZE];
    let flash = RefCell::new(RmwMultiwriteNorFlashStorage::new(
        Flash::new(p.FLASH, Irqs),
        &mut flash_buffer,
    ));

    // Use upper 1MB of the 2MB flash
    let range = (1024 * 1024)..(2048 * 1024);

    let mut scsi_buffer = [Aligned::<A4, _>([0u8; BLOCK_SIZE]); 1];
    // Create SCSI target for our block device
    let scsi = Scsi::new(
        BlockDeviceAdapter::new(FlashBlockDevice { flash, range }),
        blocks_to_slice_mut(&mut scsi_buffer),
        "Embassy",
        "MSC",
    );

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
