#![no_std]
#![no_main]
use aligned::{A4, Aligned};
use block_device_driver::BlockDevice;
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::msc::subclass::scsi::Scsi;
use embassy_usb::class::msc::subclass::scsi::block_device::BlockDeviceError;
use embassy_usb::class::msc::transport::bulk_only::BulkOnlyTransport;
use panic_probe as _;

// 512 is a standard block size supported by most systems
const BLOCK_SIZE: usize = 512;
const BLOCK_COUNT: usize = 128;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

struct RamBlockDevice {
    data: [u8; BLOCK_SIZE * BLOCK_COUNT],
}

impl BlockDevice<BLOCK_SIZE> for RamBlockDevice {
    type Error = BlockDeviceError;
    type Align = A4;

    async fn size(&mut self) -> Result<u64, BlockDeviceError> {
        Ok((BLOCK_COUNT * BLOCK_SIZE) as u64)
    }

    async fn read(&mut self, lba: u32, blocks: &mut [Aligned<A4, [u8; BLOCK_SIZE]>]) -> Result<(), BlockDeviceError> {
        for (lba, block) in (lba..).zip(blocks) {
            block.copy_from_slice(&self.data[lba as usize * BLOCK_SIZE..(lba as usize + 1) * BLOCK_SIZE]);
        }
        Ok(())
    }

    async fn write(&mut self, lba: u32, blocks: &[Aligned<A4, [u8; BLOCK_SIZE]>]) -> Result<(), BlockDeviceError> {
        for (lba, block) in (lba..).zip(blocks) {
            self.data[lba as usize * BLOCK_SIZE..(lba as usize + 1) * BLOCK_SIZE].copy_from_slice(&block[..]);
        }
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

    // Create SCSI target for our block device
    let mut scsi_buffer = [Aligned::<A4, _>([0u8; BLOCK_SIZE]); 1];
    let scsi = Scsi::new(
        RamBlockDevice {
            data: [0u8; BLOCK_SIZE * BLOCK_COUNT],
        },
        &mut scsi_buffer,
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
