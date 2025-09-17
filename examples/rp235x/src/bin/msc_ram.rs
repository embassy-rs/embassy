//! Example: RP235x firmware using embassy-rp + MSC submodule with RAM disk
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::peripherals::USB;
use embassy_rp::{bind_interrupts, usb};
use embassy_time::Delay;
use embassy_usb::class::msc::{BlockDevice, MassStorage, BLOCK_SIZE};
use embassy_usb::{Builder, Config};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

struct RamDisk {
    storage: &'static mut [u8],
    num_blocks: u32,
}

impl RamDisk {
    fn new(storage: &'static mut [u8]) -> Self {
        assert_eq!(storage.len() % BLOCK_SIZE, 0, "Buffer size should be multiple of 512");
        let num_blocks = (storage.len() / BLOCK_SIZE) as u32;
        Self { storage, num_blocks }
    }
}

#[allow(async_fn_in_trait)]
impl BlockDevice for RamDisk {
    fn num_blocks(&self) -> u32 {
        self.num_blocks
    }

    async fn read_block(&mut self, block_idx: u32, buf: &mut [u8; 512]) {
        let start = (block_idx as usize) * BLOCK_SIZE;
        let end = start + BLOCK_SIZE;
        buf.copy_from_slice(&self.storage[start..end]);
    }

    async fn write_block(&mut self, block_idx: u32, buf: &[u8; 512]) -> Result<(), ()> {
        let start = (block_idx as usize) * BLOCK_SIZE;
        let end = start + BLOCK_SIZE;
        self.storage[start..end].copy_from_slice(buf);
        Ok(())
    }
}

// =====================================================================================
// USB driver setup
// =====================================================================================

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Create RAM disk backing storage.
    static DISK: StaticCell<[u8; 64 * 1024]> = StaticCell::new();
    let disk = DISK.init([0; 64 * 1024]); // 64KB RAM disk

    // Preload with a simple text file at block 0.
    let content = b"Hello from Embassy-RP Mass Storage!\r\n";
    disk[..content.len()].copy_from_slice(content);

    let p = embassy_rp::init(Default::default());

    // USB driver
    let driver = usb::Driver::new(p.USB, Irqs);

    // USB device config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("RP235x MSC");
    config.serial_number = Some("TEST1234");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut msos_descriptor = [0; 256];

    // Create builder
    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    // Create MSC class with RamDisk
    let bd = RamDisk::new(disk);
    let mut msc = MassStorage::new(&mut builder, bd, b"VENDOR  ", b"RP235x RAMDISK  ", b"0001");

    let mut usb = builder.build();
    let mut delay = Delay;

    // Run USB + MSC concurrently
    embassy_futures::join::join(usb.run(), msc.run(&mut delay)).await;
}
