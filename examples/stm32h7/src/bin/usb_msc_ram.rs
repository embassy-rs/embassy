#![no_std]
#![no_main]

use aligned::{A4, Aligned};
use block_device_driver::BlockDevice;
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::msc::subclass::scsi::Scsi;
use embassy_usb::class::msc::subclass::scsi::block_device::BlockDeviceError;
use embassy_usb::class::msc::transport::bulk_only::BulkOnlyTransport;
use panic_probe as _;

const BLOCK_SIZE: usize = 512;
const BLOCK_COUNT: usize = 128;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

struct RamBlockDevice {
    data: [u8; BLOCK_SIZE * BLOCK_COUNT],
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

impl BlockDevice<BLOCK_SIZE> for RamBlockDevice {
    type Error = BlockDeviceError;
    type Align = A4;

    async fn size(&mut self) -> Result<u64, BlockDeviceError> {
        Ok((BLOCK_COUNT * BLOCK_SIZE) as u64)
    }

    async fn read(&mut self, lba: u32, blocks: &mut [Aligned<A4, [u8; BLOCK_SIZE]>]) -> Result<(), BlockDeviceError> {
        for (lba, block) in (lba..).zip(blocks) {
            block.copy_from_slice(&self.data[self.block_range(lba)?]);
        }
        Ok(())
    }

    async fn write(&mut self, lba: u32, blocks: &[Aligned<A4, [u8; BLOCK_SIZE]>]) -> Result<(), BlockDeviceError> {
        for (lba, block) in (lba..).zip(blocks) {
            let range = self.block_range(lba)?;
            self.data[range].copy_from_slice(&block[..]);
        }
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

    let mut scsi_buffer = [Aligned::<A4, _>([0u8; BLOCK_SIZE]); 1];
    let scsi = Scsi::new(
        RamBlockDevice {
            data: [0; BLOCK_SIZE * BLOCK_COUNT],
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
