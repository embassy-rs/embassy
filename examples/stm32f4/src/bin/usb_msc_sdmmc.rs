#![no_std]
#![no_main]
use core::cell::RefCell;

use block_device_driver::{BlockDevice as StorageBlockDevice, slice_to_blocks, slice_to_blocks_mut};
use defmt::{panic, *};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::sdmmc::Sdmmc;
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, dma, peripherals, sdmmc, usb};
use embassy_time::Delay;
use embassy_usb::Builder;
use embassy_usb::class::msc::subclass::scsi::Scsi;
use embassy_usb::class::msc::subclass::scsi::block_device::{BlockDevice, BlockDeviceError};
use embassy_usb::class::msc::transport::bulk_only::BulkOnlyTransport;
use panic_probe as _;
use sdio::sd::Card;

// SDMMC driver only supports 512 byte blocks for now
const BLOCK_SIZE: usize = 512;

type MySdmmc<'d> = sdio::BlockDevice<Card, Sdmmc<'d>, Delay, BLOCK_SIZE>;
const SDIO_FREQ: u32 = 12_000_000;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
    SDIO => sdmmc::InterruptHandler<peripherals::SDIO>;
    DMA2_STREAM6 => dma::InterruptHandler<peripherals::DMA2_CH6>;
});

struct SdmmcBlockDevice<'d> {
    sdmmc: RefCell<MySdmmc<'d>>,
    multiblock_lba: Option<u32>,
}

impl<'d> BlockDevice for SdmmcBlockDevice<'d> {
    fn status(&self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn block_size(&self) -> Result<usize, BlockDeviceError> {
        Ok(BLOCK_SIZE)
    }

    async fn num_blocks(&self) -> Result<u32, BlockDeviceError> {
        self.sdmmc
            .borrow_mut()
            .size()
            .await
            .map(|size| (size / BLOCK_SIZE as u64) as u32)
            .map_err(|_| BlockDeviceError::ReadError)
    }

    async fn read_block(&self, lba: u32, block: &mut [u8]) -> Result<(), BlockDeviceError> {
        if block.as_ptr() as usize % 4 != 0 {
            panic!("Invalid block alignment for SDMMC");
        }

        self.sdmmc
            .borrow_mut()
            .read(lba, slice_to_blocks_mut(block))
            .await
            .map_err(|e| {
                error!("SDMMC read error: {:?}", e);
                BlockDeviceError::ReadError
            })
    }

    async fn write_block(&mut self, lba: u32, block: &[u8]) -> Result<(), BlockDeviceError> {
        if block.as_ptr() as usize % 4 != 0 {
            panic!("Invalid block alignment for SDMMC");
        }

        self.sdmmc
            .borrow_mut()
            .write(lba, slice_to_blocks(block))
            .await
            .map_err(|e| {
                error!("SDMMC write error: {:?}", e);
                BlockDeviceError::WriteError
            })
    }

    async fn prepare_multiblock_write(&mut self, lba: u32, _blocks_count: u32) -> Result<(), BlockDeviceError> {
        self.multiblock_lba = Some(lba);
        Ok(())
    }

    async fn write_multiblock_block(&mut self, block: &[u8]) -> Result<(), BlockDeviceError> {
        let lba = self.multiblock_lba.ok_or(BlockDeviceError::Unknown)?;
        self.write_block(lba, block).await?;
        self.multiblock_lba = Some(lba + 1);
        Ok(())
    }

    async fn stop_multiblock_write(&mut self) -> Result<(), BlockDeviceError> {
        self.multiblock_lba = None;
        Ok(())
    }

    async fn prepare_multiblock_read(&mut self, lba: u32) -> Result<(), BlockDeviceError> {
        self.multiblock_lba = Some(lba);
        Ok(())
    }

    async fn read_multiblock_block(&mut self, block: &mut [u8]) -> Result<(), BlockDeviceError> {
        let lba = self.multiblock_lba.ok_or(BlockDeviceError::Unknown)?;
        self.read_block(lba, block).await?;
        self.multiblock_lba = Some(lba + 1);
        Ok(())
    }

    async fn stop_multiblock_read(&mut self) -> Result<(), BlockDeviceError> {
        self.multiblock_lba = None;
        Ok(())
    }
}

#[repr(align(4))]
struct AlignedBuffer([u8; BLOCK_SIZE]);

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

    let sdmmc = Sdmmc::new_4bit(
        p.SDIO,
        p.DMA2_CH6,
        p.PC12,
        p.PD2,
        p.PC8,
        p.PC9,
        p.PC10,
        p.PC11,
        Irqs,
        Default::default(),
    );

    let sdmmc = sdio::BlockDevice::new_sd_card(sdmmc, SDIO_FREQ, Delay)
        .await
        .expect("SD card init failed");
    info!("Initialized SD card: {:#?}", Debug2Format(sdmmc.card()));

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

    // Create SCSI target for our block device
    let mut scsi_buffer = AlignedBuffer([0u8; BLOCK_SIZE]);
    let scsi = Scsi::new(
        SdmmcBlockDevice {
            sdmmc: RefCell::new(sdmmc),
            multiblock_lba: None,
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
