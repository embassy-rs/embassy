#![no_std]
#![no_main]

use core::cmp::min;

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::{self, Driver};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_usb::Builder;
use embassy_usb::class::msc::{BlockDevice, MscClass, State};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});

const BLOCK_SIZE: usize = 512;
const DISK_BYTES: u64 = 1024 * 1024 * 1024;
const BLOCK_COUNT: u32 = (DISK_BYTES / BLOCK_SIZE as u64) as u32;

const SECTORS_PER_CLUSTER: u32 = 32;
const RESERVED_SECTORS: u32 = 1;
const FAT_COUNT: u32 = 2;
const FAT_SECTORS: u32 = 256;
const ROOT_DIR_ENTRIES: u16 = 512;
const ROOT_DIR_SECTORS: u32 = (ROOT_DIR_ENTRIES as u32 * 32) / BLOCK_SIZE as u32;

const FAT1_START: u32 = RESERVED_SECTORS;
const FAT2_START: u32 = FAT1_START + FAT_SECTORS;
const ROOT_DIR_START: u32 = FAT2_START + FAT_SECTORS;
const DATA_START: u32 = ROOT_DIR_START + ROOT_DIR_SECTORS;

const INDEX_CLUSTER: u16 = 2;
const README_CLUSTER: u16 = 3;

const INDEX_HTML: &[u8] = br#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Embassy USB MSC</title>
</head>
<body>
  <h1>Embassy USB MSC virtual disk</h1>
  <p>This is a virtual 1GiB FAT16 disk.</p>
  <p>Files are generated on-the-fly by firmware.</p>
</body>
</html>
"#;

const README_MD: &[u8] = br#"# Embassy USB MSC

This disk is virtual and reports a capacity of 1 GiB.

- file: index.html
- file: readme.md

The content is generated in firmware without allocating a 1 GiB RAM buffer.
"#;

struct VirtualFatDisk;

impl VirtualFatDisk {
    fn new() -> Self {
        Self
    }

    fn write_u16_le(buf: &mut [u8], offset: usize, value: u16) {
        let bytes = value.to_le_bytes();
        buf[offset..offset + 2].copy_from_slice(&bytes);
    }

    fn write_u32_le(buf: &mut [u8], offset: usize, value: u32) {
        let bytes = value.to_le_bytes();
        buf[offset..offset + 4].copy_from_slice(&bytes);
    }

    fn short_name_checksum(short_name: &[u8; 11]) -> u8 {
        let mut sum = 0u8;
        for &c in short_name {
            sum = ((sum & 1) << 7).wrapping_add(sum >> 1).wrapping_add(c);
        }
        sum
    }

    fn write_lfn_entry_single(buf: &mut [u8], offset: usize, name: &str, checksum: u8) {
        let mut chars = [0xFFFFu16; 13];
        let mut len = 0usize;
        for b in name.bytes() {
            if len == chars.len() {
                break;
            }
            chars[len] = b as u16;
            len += 1;
        }
        if len < chars.len() {
            chars[len] = 0;
        }

        buf[offset] = 0x41; // single + last LFN entry
        buf[offset + 11] = 0x0F;
        buf[offset + 12] = 0;
        buf[offset + 13] = checksum;
        Self::write_u16_le(buf, offset + 26, 0);

        for (slot, idx) in [1usize, 3, 5, 7, 9].iter().zip(0..5) {
            Self::write_u16_le(buf, offset + *slot, chars[idx]);
        }
        for (slot, idx) in [14usize, 16, 18, 20, 22, 24].iter().zip(5..11) {
            Self::write_u16_le(buf, offset + *slot, chars[idx]);
        }
        for (slot, idx) in [28usize, 30].iter().zip(11..13) {
            Self::write_u16_le(buf, offset + *slot, chars[idx]);
        }
    }

    fn write_short_entry(
        buf: &mut [u8],
        offset: usize,
        short_name: &[u8; 11],
        attr: u8,
        first_cluster: u16,
        size: u32,
    ) {
        buf[offset..offset + 11].copy_from_slice(short_name);
        buf[offset + 11] = attr;
        buf[offset + 12] = 0;
        buf[offset + 13] = 0;
        Self::write_u16_le(buf, offset + 14, 0);
        Self::write_u16_le(buf, offset + 16, 0);
        Self::write_u16_le(buf, offset + 18, 0);
        Self::write_u16_le(buf, offset + 20, 0);
        Self::write_u16_le(buf, offset + 22, 0);
        Self::write_u16_le(buf, offset + 24, 0);
        Self::write_u16_le(buf, offset + 26, first_cluster);
        Self::write_u32_le(buf, offset + 28, size);
    }

    fn fill_boot_sector(&self, buf: &mut [u8]) {
        buf.fill(0);
        buf[0] = 0xEB;
        buf[1] = 0x3C;
        buf[2] = 0x90;
        buf[3..11].copy_from_slice(b"MSDOS5.0");

        Self::write_u16_le(buf, 11, BLOCK_SIZE as u16);
        buf[13] = SECTORS_PER_CLUSTER as u8;
        Self::write_u16_le(buf, 14, RESERVED_SECTORS as u16);
        buf[16] = FAT_COUNT as u8;
        Self::write_u16_le(buf, 17, ROOT_DIR_ENTRIES);
        Self::write_u16_le(buf, 19, 0);
        buf[21] = 0xF8;
        Self::write_u16_le(buf, 22, FAT_SECTORS as u16);
        Self::write_u16_le(buf, 24, 63);
        Self::write_u16_le(buf, 26, 255);
        Self::write_u32_le(buf, 28, 0);
        Self::write_u32_le(buf, 32, BLOCK_COUNT);

        buf[36] = 0x80;
        buf[37] = 0;
        buf[38] = 0x29;
        Self::write_u32_le(buf, 39, 0x20260325);
        buf[43..54].copy_from_slice(b"EMBASSY USB");
        buf[54..62].copy_from_slice(b"FAT16   ");

        buf[510] = 0x55;
        buf[511] = 0xAA;
    }

    fn fill_fat_sector(&self, fat_sector: u32, buf: &mut [u8]) {
        buf.fill(0);
        if fat_sector != 0 {
            return;
        }

        // Cluster 0/1 reserved, cluster 2/3 are our files and marked EOC.
        Self::write_u16_le(buf, 0, 0xFFF8);
        Self::write_u16_le(buf, 2, 0xFFFF);
        Self::write_u16_le(buf, 4, 0xFFFF);
        Self::write_u16_le(buf, 6, 0xFFFF);
    }

    fn fill_root_dir_sector(&self, sector: u32, buf: &mut [u8]) {
        buf.fill(0);
        if sector != 0 {
            return;
        }

        let index_short: [u8; 11] = *b"INDEX~1 HTM";
        let readme_short: [u8; 11] = *b"README  MD ";

        let index_sum = Self::short_name_checksum(&index_short);
        let readme_sum = Self::short_name_checksum(&readme_short);

        Self::write_lfn_entry_single(buf, 0, "index.html", index_sum);
        Self::write_short_entry(buf, 32, &index_short, 0x20, INDEX_CLUSTER, INDEX_HTML.len() as u32);

        Self::write_lfn_entry_single(buf, 64, "readme.md", readme_sum);
        Self::write_short_entry(buf, 96, &readme_short, 0x20, README_CLUSTER, README_MD.len() as u32);
    }

    fn fill_file_sector(&self, file: &[u8], sector_in_cluster: u32, buf: &mut [u8]) {
        buf.fill(0);
        let start = sector_in_cluster as usize * BLOCK_SIZE;
        if start >= file.len() {
            return;
        }
        let end = min(start + BLOCK_SIZE, file.len());
        buf[..(end - start)].copy_from_slice(&file[start..end]);
    }
}

impl BlockDevice for VirtualFatDisk {
    type Error = ();

    fn block_size(&self) -> u32 {
        BLOCK_SIZE as u32
    }

    fn block_count(&self) -> u32 {
        BLOCK_COUNT
    }

    fn read_block(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        if buf.len() != BLOCK_SIZE {
            return Err(());
        }
        if lba >= BLOCK_COUNT {
            return Err(());
        }

        if lba == 0 {
            self.fill_boot_sector(buf);
            return Ok(());
        }

        if (FAT1_START..FAT1_START + FAT_SECTORS).contains(&lba) {
            self.fill_fat_sector(lba - FAT1_START, buf);
            return Ok(());
        }
        if (FAT2_START..FAT2_START + FAT_SECTORS).contains(&lba) {
            self.fill_fat_sector(lba - FAT2_START, buf);
            return Ok(());
        }

        if (ROOT_DIR_START..ROOT_DIR_START + ROOT_DIR_SECTORS).contains(&lba) {
            self.fill_root_dir_sector(lba - ROOT_DIR_START, buf);
            return Ok(());
        }

        if lba >= DATA_START {
            let rel = lba - DATA_START;
            let cluster = rel / SECTORS_PER_CLUSTER + 2;
            let sector_in_cluster = rel % SECTORS_PER_CLUSTER;

            if cluster == INDEX_CLUSTER as u32 {
                self.fill_file_sector(INDEX_HTML, sector_in_cluster, buf);
                return Ok(());
            }
            if cluster == README_CLUSTER as u32 {
                self.fill_file_sector(README_MD, sector_in_cluster, buf);
                return Ok(());
            }
        }

        buf.fill(0);
        Ok(())
    }

    fn write_block(&mut self, _lba: u32, data: &[u8]) -> Result<(), Self::Error> {
        if data.len() != BLOCK_SIZE {
            return Err(());
        }
        // Keep the virtual image stateless while still accepting host write traffic.
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;

        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV2,
            mul: PllMul::MUL72,
            divp: None,
            divq: Some(PllQDiv::DIV6),
            divr: Some(PllRDiv::DIV2),
        });
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.boost = true;
        config.rcc.mux.clk48sel = mux::Clk48sel::HSI48;
    }

    let p = embassy_stm32::init(config);

    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-MSC example");
    config.serial_number = Some("12345678");
    config.composite_with_iads = false;
    config.device_class = 0;
    config.device_sub_class = 0;
    config.device_protocol = 0;

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

    let mut msc = MscClass::new(&mut builder, &mut state, 64);
    let mut usb = builder.build();

    let mut disk = VirtualFatDisk::new();
    let mut block_buf = [0u8; BLOCK_SIZE];

    info!("USB MSC example started (virtual 1GiB, index.html, readme.md).");

    join(usb.run(), msc.run(&mut disk, &mut block_buf)).await;
}
