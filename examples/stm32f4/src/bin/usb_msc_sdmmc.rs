#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

use core::cell::RefCell;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::{DMA2_CH6, SDIO};
use embassy_stm32::sdmmc::{self, Sdmmc};
use embassy_stm32::time::{mhz, Hertz};
use embassy_stm32::usb_otg::Driver;
use embassy_stm32::{interrupt, Config};
use embassy_usb::class::msc::subclass::scsi::block_device::{BlockDevice, BlockDeviceError};
use embassy_usb::class::msc::subclass::scsi::Scsi;
use embassy_usb::class::msc::transport::bulk_only::BulkOnlyTransport;
use embassy_usb::Builder;
use futures::future::join;
use {defmt_rtt as _, panic_probe as _};

// SDMMC driver only supports 512 byte blocks for now
const BLOCK_SIZE: usize = 512;

type MySdmmc<'d> = Sdmmc<'d, SDIO, DMA2_CH6>;
const SDIO_FREQ: Hertz = Hertz(12_000_000);

struct SdmmcBlockDevice<'d> {
    sdmmc: RefCell<MySdmmc<'d>>,
}

impl<'d> BlockDevice for SdmmcBlockDevice<'d> {
    fn status(&self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn block_size(&self) -> Result<usize, BlockDeviceError> {
        Ok(BLOCK_SIZE)
    }

    fn num_blocks(&self) -> Result<u32, BlockDeviceError> {
        // Ok(128)
        Ok((self.sdmmc.borrow().card().unwrap().csd.card_size() / BLOCK_SIZE as u64) as u32)
    }

    async fn read_block(&self, lba: u32, block: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.sdmmc.borrow_mut().read_block(lba, block).await.map_err(|e| {
            error!("SDMMC read error: {:?}", e);
            BlockDeviceError::ReadError
        })
    }

    async fn write_block(&mut self, lba: u32, block: &[u8]) -> Result<(), BlockDeviceError> {
        self.sdmmc.borrow_mut().write_block(lba, block).await.map_err(|e| {
            error!("SDMMC write error: {:?}", e);
            BlockDeviceError::WriteError
        })
    }
}

#[repr(align(4))]
struct AlignedBuffer([u8; BLOCK_SIZE]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut config = Config::default();
    config.rcc.pll48 = true;
    config.rcc.sys_ck = Some(mhz(48));

    let p = embassy_stm32::init(config);

    let mut config = sdmmc::Config::default();
    // config.data_transfer_timeout = 20_000_000;
    let mut sdmmc = Sdmmc::new_4bit(
        p.SDIO,
        interrupt::take!(SDIO),
        p.DMA2_CH6,
        p.PC12,
        p.PD2,
        p.PC8,
        p.PC9,
        p.PC10,
        p.PC11,
        config,
    );

    sdmmc.init_card(SDIO_FREQ).await.expect("SD card init failed");
    info!("Initialized SD card: {:#?}", Debug2Format(sdmmc.card().unwrap()));

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

    // Create SCSI target for our block device
    let mut scsi_buffer = AlignedBuffer([0u8; BLOCK_SIZE]);
    let scsi = Scsi::new(
        SdmmcBlockDevice {
            sdmmc: RefCell::new(sdmmc),
        },
        &mut scsi_buffer.0,
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
