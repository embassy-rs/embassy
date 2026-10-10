//! This example shows how to use USB Mass Storage Class (MSC) with the RP2040 chip.
//!
//! This exposes a read-only virtual 1 GiB FAT16 disk, created on the fly in firmware,
//! containing a single README.TXT file.

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

const BLOCK_SIZE: usize = 512;
const BLOCK_COUNT: u32 = (1 * 1024 * 1024 * 1024) / BLOCK_SIZE as u32;
const README: &[u8] = b"Hello from Embassy USB Mass Storage Class!\r\n";

struct VirtualFatDisk;

impl BlockDevice for VirtualFatDisk {
    type Error = ();

    fn block_size(&self) -> u32 {
        BLOCK_SIZE as u32
    }

    fn block_count(&self) -> u32 {
        BLOCK_COUNT
    }

    fn read_blocks(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        for (i, block) in buf.chunks_exact_mut(BLOCK_SIZE).enumerate() {
            read_block(lba + i as u32, block);
        }
        Ok(())
    }

    fn write_blocks(&mut self, _lba: u32, _data: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn read_block(lba: u32, buf: &mut [u8]) {
    buf.fill(0);
    match lba {
        0 => {
            // FAT16 Boot Sector (BIOS Parameter Block)
            buf[0..3].copy_from_slice(&[0xEB, 0x3C, 0x90]);
            buf[3..11].copy_from_slice(b"MSDOS5.0");
            buf[11..13].copy_from_slice(&(BLOCK_SIZE as u16).to_le_bytes());
            buf[13] = 32; // 32 sectors per cluster (16 KiB clusters)
            buf[14..16].copy_from_slice(&1u16.to_le_bytes()); // 1 reserved sector
            buf[16] = 2; // 2 FAT tables
            buf[17..19].copy_from_slice(&512u16.to_le_bytes()); // 512 root entries (32 sectors)
            buf[21] = 0xF8; // media descriptor (fixed disk)
            buf[22..24].copy_from_slice(&256u16.to_le_bytes()); // 256 sectors per FAT
            buf[24..26].copy_from_slice(&63u16.to_le_bytes()); // sectors per track
            buf[26..28].copy_from_slice(&255u16.to_le_bytes()); // heads
            buf[32..36].copy_from_slice(&BLOCK_COUNT.to_le_bytes());
            buf[36] = 0x80; // drive number
            buf[38] = 0x29; // boot signature
            buf[39..43].copy_from_slice(b"\x12\x34\x56\x78"); // volume ID
            buf[43..54].copy_from_slice(b"EMBASSY USB");
            buf[54..62].copy_from_slice(b"FAT16   ");
            buf[510..512].copy_from_slice(&[0x55, 0xAA]);
        }
        1 | 257 => {
            // FAT1 (LBA 1) and FAT2 (LBA 257)
            // Cluster 0 (media descriptor) = 0xFFF8, Cluster 1 = 0xFFFF, Cluster 2 (README) = EOF (0xFFFF)
            buf[0..2].copy_from_slice(&0xFFF8u16.to_le_bytes());
            buf[2..4].copy_from_slice(&0xFFFFu16.to_le_bytes());
            buf[4..6].copy_from_slice(&0xFFFFu16.to_le_bytes());
        }
        513 => {
            // Root directory sector 0 (1 reserved sector + 2 * 256 FAT sectors = LBA 513)
            // Entry 0: 8.3 filename "README.TXT" pointing to cluster 2
            buf[0..11].copy_from_slice(b"README  TXT");
            buf[11] = 0x20; // archive attribute
            buf[26..28].copy_from_slice(&2u16.to_le_bytes()); // starting cluster = 2
            buf[28..32].copy_from_slice(&(README.len() as u32).to_le_bytes());
        }
        545 => {
            // Cluster 2 data start = LBA 513 + 32 (root dir sectors) = LBA 545
            buf[..README.len()].copy_from_slice(README);
        }
        _ => {}
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
        .product_id("Virtual Disk")
        .product_revision_level("1.0")
        .serial_number("12345678");

    let mut msc = MscClass::new(&mut builder, &mut state, msc_config);
    let mut usb = builder.build();

    let mut disk = VirtualFatDisk;
    let mut block_buf = [0u8; BLOCK_SIZE];

    info!("USB MSC example started (virtual 1GiB FAT16 disk).");

    join(usb.run(), msc.run(&mut disk, &mut block_buf)).await;
}
