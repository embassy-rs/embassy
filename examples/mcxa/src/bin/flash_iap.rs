//! Flash IAP (In-Application Programming) example for MCXA276.
//!
//! Demonstrates the ROM API flash driver: init, erase, program, verify, and read.
//!
//! This example operates on sector `0x0C0000` (`SECTOR_INDEX_FROM_END = 32`),
//! matching the C SDK `frdmmcxa276_romapi_flashiap` demo. The top-of-flash
//! sectors (0x0FC000+) are avoided because they overlap the protected
//! FFR / CFPA / CFPB configuration region on MCXA276.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hal::config::Config;
use hal::flash::{Flash, FlashProperty};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

/// Number of sectors from the end of flash to use as the test target.
///
/// The C SDK header defaults to 2, but on MCXA276 the top-of-flash region
/// (0x0FC000+) is the protected FFR / CFPA / CFPB configuration area.
/// Accessing it causes a fault. The C SDK demo for MCXA276 therefore uses
/// 32, which targets sector 0x0C0000 — well below the protected zone.
const SECTOR_INDEX_FROM_END: u32 = 32;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = hal::init(Config::default());

    defmt::info!("Flash driver API tree demo application.");

    // -----------------------------------------------------------------------
    // 1. Initialise the flash driver via ROM API
    // -----------------------------------------------------------------------
    defmt::info!("\n Initializing flash driver.");
    let mut flash = match Flash::new() {
        Ok(f) => f,
        Err(e) => {
            defmt::error!("Flash init failed: {}", e);
            halt();
        }
    };
    defmt::info!("\n Flash init successfull!\n");

    defmt::info!("\n Config flash memory access time.\n");

    let pflash_block_base = match flash.get_property(FlashProperty::PflashBlockBaseAddr) {
        Ok(v) => v,
        Err(e) => {
            defmt::error!("Flash get_property(base) failed: {}", e);
            halt();
        }
    };
    let pflash_sector_size = match flash.get_property(FlashProperty::PflashSectorSize) {
        Ok(v) => v,
        Err(e) => {
            defmt::error!("Flash get_property(sector) failed: {}", e);
            halt();
        }
    };
    let pflash_total_size = match flash.get_property(FlashProperty::PflashTotalSize) {
        Ok(v) => v,
        Err(e) => {
            defmt::error!("Flash get_property(total) failed: {}", e);
            halt();
        }
    };
    let pflash_page_size = match flash.get_property(FlashProperty::PflashPageSize) {
        Ok(v) => v,
        Err(e) => {
            defmt::error!("Flash get_property(page) failed: {}", e);
            halt();
        }
    };

    // -----------------------------------------------------------------------
    // 2. Compute the test address (second-to-last sector)
    // -----------------------------------------------------------------------
    defmt::info!("\n PFlash Information:");
    defmt::info!("\n kFLASH_PropertyPflashBlockBaseAddr = 0x{:X}", pflash_block_base);
    defmt::info!("\n kFLASH_PropertyPflashSectorSize = {}", pflash_sector_size);
    defmt::info!("\n kFLASH_PropertyPflashTotalSize = {}", pflash_total_size);
    defmt::info!("\n kFLASH_PropertyPflashPageSize = 0x{:X}", pflash_page_size);

    let dest_addr: u32 = pflash_block_base + (pflash_total_size - (SECTOR_INDEX_FROM_END * pflash_sector_size));

    // -----------------------------------------------------------------------
    // 4. Erase the sector
    // -----------------------------------------------------------------------
    defmt::info!("\n Erase a sector of flash");
    if let Err(e) = flash.blocking_erase(dest_addr, pflash_sector_size) {
        defmt::error!("Erase failed: {}", e);
        halt();
    }

    defmt::info!("\n Calling flash_verify_erase_sector() API.");
    if let Err(e) = flash.verify_erase_sector(dest_addr, pflash_sector_size) {
        defmt::error!("Erase verification failed: {}", e);
        halt();
    }
    defmt::info!(
        "\n Successfully erased sector: 0x{:x} -> 0x{:x}\n",
        dest_addr,
        dest_addr + pflash_sector_size
    );

    // Prepare user buffer (512 bytes = 128 u32s), matching the C example.
    let mut write_buf = [0u8; 512];
    for (i, chunk) in write_buf.chunks_exact_mut(4).enumerate() {
        let val = (i as u32).to_le_bytes();
        chunk.copy_from_slice(&val);
    }

    defmt::info!("\n Calling FLASH_Program() API.");
    if let Err(e) = flash.blocking_program(dest_addr, &write_buf) {
        defmt::error!("Program failed: {}", e);
        halt();
    }

    defmt::info!("\n Calling FLASH_VerifyProgram() API.");
    if let Err(e) = flash.verify_program(dest_addr, &write_buf) {
        defmt::error!("Program verification failed: {}", e);
        halt();
    }

    let mut read_buf = [0u8; 512];
    let read_offset = dest_addr - pflash_block_base;
    if let Err(e) = flash.blocking_read(read_offset, &mut read_buf) {
        defmt::error!("Readback failed: {}", e);
        halt();
    }
    if read_buf != write_buf {
        defmt::error!("Readback mismatch detected.");
        halt();
    }

    defmt::info!(
        "\n Successfully programmed and verified location: 0x{:x} -> 0x{:x} \n",
        dest_addr,
        dest_addr + (write_buf.len() as u32)
    );

    if let Err(e) = flash.blocking_erase(dest_addr, pflash_sector_size) {
        defmt::error!("Cleanup erase failed: {}", e);
        halt();
    }
    defmt::info!("\n End of PFlash Example! \n");

    loop {
        cortex_m::asm::wfi();
    }
}

/// Halt the CPU in an infinite loop (used on unrecoverable errors).
fn halt() -> ! {
    defmt::error!("HALTED DUE TO FLASH ERROR");
    loop {
        cortex_m::asm::wfi();
    }
}
