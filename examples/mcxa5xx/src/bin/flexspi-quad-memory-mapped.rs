#![no_std]
#![no_main]

//! FlexSPI memory-mapped (XIP-style) read example for MCXA577.
//!
//! Once [`Flexspi::new_blocking`] has configured `FLEXSPI0`, the external NOR
//! flash is directly readable through the FlexSPI AHB memory-mapped window —
//! no IP commands required. This example programs a sector with IP commands
//! (the writable path the driver exposes) and then reads it back purely
//! through the memory-mapped window, verifying the two agree.
//!
//! Notes for MCXA577:
//! * The core boots in the secure state, so the secure window alias
//!   `0x9000_0000` is used. The non-secure alias `0x8000_0000` would bus-fault
//!   when accessed from secure code.
//! * The shared `FLASH_CONFIG` read sequence uses a 3-byte (24-bit) address,
//!   so only the low 16 MiB is reachable; the test offset stays under that.

use defmt::{info, panic, unwrap};
use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::config::Config;
use hal::flexspi::{ClockConfig as FlexspiClockConfig, Flexspi, NorFlash};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[path = "../flexspi_common.rs"]
mod flexspi_common;

use flexspi_common::{FLASH_CONFIG, FLASH_PAGE_SIZE, FLASH_SECTOR_SIZE, check_pattern, fill_pattern};

/// FlexSPI AHB memory-mapped window base (secure alias; the core boots secure).
const FLEXSPI_AMBA_BASE: u32 = 0x9000_0000;
/// Flash offset to exercise (sector aligned, under the 16 MiB 3-byte reach).
const FLASH_OFFSET: u32 = 0x0040_0000;

/// Read `buf.len()` bytes from the memory-mapped flash window at `offset`.
///
/// Each access lands in the FlexSPI AHB window, where the controller services
/// it by running the AHB read LUT sequence against the NOR flash — there is no
/// driver call involved, just a load.
fn memory_mapped_read(offset: u32, buf: &mut [u8]) {
    for (i, b) in buf.iter_mut().enumerate() {
        let addr = (FLEXSPI_AMBA_BASE + offset + i as u32) as *const u8;
        // SAFETY: `addr` is inside the FlexSPI memory-mapped window configured
        // by the driver; the region is valid for reads while the driver lives.
        *b = unsafe { core::ptr::read_volatile(addr) };
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Config::default());

    info!("FlexSPI memory-mapped read example");

    let flexspi = unwrap!(Flexspi::new_blocking(
        p.FLEXSPI0,
        p.P3_0,
        p.P3_7,
        p.P3_6,
        p.P3_8,
        p.P3_9,
        p.P3_10,
        p.P3_11,
        FlexspiClockConfig::default(),
        FLASH_CONFIG,
    ));
    let mut flash = NorFlash::new(flexspi);

    // Program a known pattern into one sector using IP commands.
    info!(
        "Programming sector at flash offset 0x{=u32:08x} via IP commands",
        FLASH_OFFSET
    );
    unwrap!(flash.blocking_erase_sector(FLASH_OFFSET));
    let mut page = [0u8; FLASH_PAGE_SIZE];
    for pg in 0..(FLASH_SECTOR_SIZE / FLASH_PAGE_SIZE) as u32 {
        let addr = FLASH_OFFSET + pg * FLASH_PAGE_SIZE as u32;
        fill_pattern(addr, &mut page);
        unwrap!(flash.blocking_page_program(addr, &page));
    }

    // Read the whole sector back through the memory-mapped window only.
    let window_addr = FLEXSPI_AMBA_BASE + FLASH_OFFSET;
    info!(
        "Reading it back through the memory-mapped window at 0x{=u32:08x}",
        window_addr
    );
    let mut readback = [0u8; FLASH_SECTOR_SIZE];
    memory_mapped_read(FLASH_OFFSET, &mut readback);

    match check_pattern(FLASH_OFFSET, &readback) {
        None => info!(
            "Memory-mapped read PASSED: {=usize} bytes match the IP-programmed pattern",
            readback.len()
        ),
        Some(bad) => panic!("memory-mapped read mismatch at flash offset 0x{=u32:08x}", bad),
    }

    loop {
        Timer::after_secs(1).await;
        info!("FlexSPI memory-mapped demo heartbeat");
    }
}
