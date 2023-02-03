#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

use defmt::{panic, todo, *};
use embassy_executor::Spawner;
use embassy_stm32::time::mhz;
use embassy_stm32::usb_otg::{Driver, Instance};
use embassy_stm32::{interrupt, Config};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::class::msc::subclass::scsi::block_device::BlockDevice;
use embassy_usb::class::msc::subclass::scsi::Scsi;
use embassy_usb::class::msc::transport::bulk_only::BulkOnlyTransport;
use embassy_usb::class::msc::transport::CommandSetHandler;
use embassy_usb::class::msc::MscSubclass;
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use futures::future::join;
use {defmt_rtt as _, panic_probe as _};

// struct CommandSet {}

// impl CommandSetHandler for CommandSet {
//     async fn command_out(
//         &mut self,
//         lun: u8,
//         cmd: &[u8],
//         pipe: &mut impl embassy_usb::class::msc::transport::DataPipeOut,
//     ) -> Result<(), embassy_usb::class::msc::transport::CommandError> {
//         info!("CMD_OUT: {:?}", cmd);
//         Ok(())
//     }

//     async fn command_in(
//         &mut self,
//         lun: u8,
//         cmd: &[u8],
//         pipe: &mut impl embassy_usb::class::msc::transport::DataPipeIn,
//     ) -> Result<(), embassy_usb::class::msc::transport::CommandError> {
//         info!("CMD_IN: {:?}", cmd);
//         Ok(())
//     }
// }

struct Device {
    data: [u8; 512 * 128],
}

impl BlockDevice for Device {
    fn block_size(&self) -> usize {
        512
    }

    fn num_blocks(&self) -> u32 {
        (self.data.len() / self.block_size()) as u32 - 1
    }

    fn read_block(
        &self,
        lba: u32,
        block: &mut [u8],
    ) -> Result<(), embassy_usb::class::msc::subclass::scsi::block_device::BlockDeviceError> {
        block.copy_from_slice(&self.data[lba as usize * 512..(lba as usize + 1) * 512]);
        Ok(())
    }

    fn write_block(
        &mut self,
        lba: u32,
        block: &[u8],
    ) -> Result<(), embassy_usb::class::msc::subclass::scsi::block_device::BlockDeviceError> {
        self.data[lba as usize * 512..(lba as usize + 1) * 512].copy_from_slice(block);
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
    config.product = Some("USB-serial example");
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

    let scsi = Scsi::new(Device { data: [0u8; 512 * 128] });

    let mut msc = BulkOnlyTransport::new(
        &mut builder,
        &mut state,
        MscSubclass::ScsiTransparentCommandSet,
        64,
        0,
        scsi,
    );

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let echo_fut = async {
        loop {
            msc.run().await;
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, echo_fut).await;
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: Instance + 'd>(class: &mut CdcAcmClass<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}
