//! This example shows how to use USB Mass Storage Class (MSC) with the RP2040 chip.
//!
//! This exposes a read-write 64 KiB block device, backed by RAM. Note that modern
//! operating systems tend to have limited support for block devices this small.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_usb::Builder;
use embassy_usb::class::msc::device::{BlockDevice, Config as MscConfig, MscClass, State};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

const BLOCK_SIZE: u32 = 512;
const BLOCK_COUNT: u32 = (64 * 1024) / BLOCK_SIZE;

struct RamBlockDevice {
    data: [u8; (BLOCK_SIZE * BLOCK_COUNT) as usize],
}

impl BlockDevice for RamBlockDevice {
    type Error = ();

    fn block_size(&self) -> u32 {
        BLOCK_SIZE
    }

    fn block_count(&self) -> u32 {
        BLOCK_COUNT
    }

    fn read_block(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        let start = (lba * BLOCK_SIZE) as usize;
        buf.copy_from_slice(&self.data[start..start + buf.len()]);
        Ok(())
    }

    fn write_block(&mut self, lba: u32, buf: &[u8]) -> Result<(), Self::Error> {
        let start = (lba * BLOCK_SIZE) as usize;
        self.data[start..start + buf.len()].copy_from_slice(buf);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-MSC example");
    config.serial_number = Some("12345678");
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
        .product_id("RAM Disk")
        .product_revision_level("1.0")
        .serial_number("12345678");

    let mut msc = MscClass::new(&mut builder, &mut state, msc_config);
    let mut usb = builder.build();

    let mut disk = RamBlockDevice {
        data: [0u8; (BLOCK_SIZE * BLOCK_COUNT) as usize],
    };
    let mut block_buf = [0u8; BLOCK_SIZE as usize];

    info!("USB MSC example started (ram disk)");

    join(usb.run(), msc.run(&mut disk, &mut block_buf)).await;
}
