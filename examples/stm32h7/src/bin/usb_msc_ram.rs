#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::msc::subclass::scsi::Scsi;
use embassy_usb::class::msc::subclass::scsi::block_device::{BlockDevice, BlockDeviceError};
use embassy_usb::class::msc::transport::bulk_only::BulkOnlyTransport;
use panic_probe as _;

const BLOCK_SIZE: usize = 512;
const BLOCK_COUNT: usize = 128;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

struct RamBlockDevice {
    data: [u8; BLOCK_SIZE * BLOCK_COUNT],
    multiblock_lba: Option<u32>,
}

impl RamBlockDevice {
    fn block_range(&self, lba: u32) -> Result<core::ops::Range<usize>, BlockDeviceError> {
        let start = (lba as usize)
            .checked_mul(BLOCK_SIZE)
            .ok_or(BlockDeviceError::LbaOutOfRange)?;
        let end = start
            .checked_add(BLOCK_SIZE)
            .filter(|&end| end <= self.data.len())
            .ok_or(BlockDeviceError::LbaOutOfRange)?;
        Ok(start..end)
    }
}

impl BlockDevice for RamBlockDevice {
    fn status(&self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn block_size(&self) -> Result<usize, BlockDeviceError> {
        Ok(BLOCK_SIZE)
    }

    async fn num_blocks(&self) -> Result<u32, BlockDeviceError> {
        Ok(BLOCK_COUNT as u32)
    }

    async fn read_block(&self, lba: u32, block: &mut [u8]) -> Result<(), BlockDeviceError> {
        block.copy_from_slice(&self.data[self.block_range(lba)?]);
        Ok(())
    }

    async fn write_block(&mut self, lba: u32, block: &[u8]) -> Result<(), BlockDeviceError> {
        let range = self.block_range(lba)?;
        self.data[range].copy_from_slice(block);
        Ok(())
    }

    async fn prepare_multiblock_write(&mut self, lba: u32, blocks_count: u32) -> Result<(), BlockDeviceError> {
        if blocks_count != 0 {
            self.block_range(lba)?;
            self.block_range(
                lba.checked_add(blocks_count - 1)
                    .ok_or(BlockDeviceError::LbaOutOfRange)?,
            )?;
        }
        self.multiblock_lba = Some(lba);
        Ok(())
    }

    async fn write_multiblock_block(&mut self, block: &[u8]) -> Result<(), BlockDeviceError> {
        let lba = self.multiblock_lba.ok_or(BlockDeviceError::Unknown)?;
        self.write_block(lba, block).await?;
        self.multiblock_lba = Some(lba.checked_add(1).ok_or(BlockDeviceError::LbaOutOfRange)?);
        Ok(())
    }

    async fn stop_multiblock_write(&mut self) -> Result<(), BlockDeviceError> {
        self.multiblock_lba = None;
        Ok(())
    }

    async fn prepare_multiblock_read(&mut self, lba: u32) -> Result<(), BlockDeviceError> {
        self.block_range(lba)?;
        self.multiblock_lba = Some(lba);
        Ok(())
    }

    async fn read_multiblock_block(&mut self, block: &mut [u8]) -> Result<(), BlockDeviceError> {
        let lba = self.multiblock_lba.ok_or(BlockDeviceError::Unknown)?;
        self.read_block(lba, block).await?;
        self.multiblock_lba = Some(lba.checked_add(1).ok_or(BlockDeviceError::LbaOutOfRange)?);
        Ok(())
    }

    async fn stop_multiblock_read(&mut self) -> Result<(), BlockDeviceError> {
        self.multiblock_lba = None;
        Ok(())
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul50,
            fracn: None,
            divp: Some(PllDiv::Div2),
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div2;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.apb3_pre = APBPrescaler::Div2;
        config.rcc.apb4_pre = APBPrescaler::Div2;
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.mux.usbsel = mux::Usbsel::Hsi48;
    }
    let p = embassy_stm32::init(config);

    let mut ep_out_buffer = [0u8; 256];
    let mut config = embassy_stm32::usb::Config::default();
    config.vbus_detection = false;
    let driver = Driver::new_fs(p.USB_OTG_FS, p.PA12, p.PA11, Irqs, &mut ep_out_buffer, config);

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("MSC example");
    config.serial_number = Some("12345678");

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

    let mut scsi_buffer = [0u8; BLOCK_SIZE];
    let scsi = Scsi::new(
        RamBlockDevice {
            data: [0; BLOCK_SIZE * BLOCK_COUNT],
            multiblock_lba: None,
        },
        &mut scsi_buffer,
        "Embassy",
        "MSC",
    );
    let mut msc_transport = BulkOnlyTransport::new(&mut builder, &mut state, 64, scsi);
    let mut usb = builder.build();

    info!("USB MSC RAM disk running");
    join(usb.run(), msc_transport.run()).await;
}
