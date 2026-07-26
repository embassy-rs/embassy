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
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
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
    let mut flash = defmt::unwrap!(Flash::new());
    defmt::info!("\n Flash init successfull!\n");

    defmt::info!("\n Config flash memory access time.\n");

    let pflash_block_base = defmt::unwrap!(flash.get_property(FlashProperty::PflashBlockBaseAddr));
    let pflash_sector_size = defmt::unwrap!(flash.get_property(FlashProperty::PflashSectorSize));
    let pflash_total_size = defmt::unwrap!(flash.get_property(FlashProperty::PflashTotalSize));
    let pflash_page_size = defmt::unwrap!(flash.get_property(FlashProperty::PflashPageSize));

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
    defmt::unwrap!(flash.blocking_erase(dest_addr, pflash_sector_size));

    defmt::info!("\n Calling flash_verify_erase_sector() API.");
    defmt::unwrap!(flash.verify_erase_sector(dest_addr, pflash_sector_size));
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
    defmt::unwrap!(flash.blocking_program(dest_addr, &write_buf));

    defmt::info!("\n Calling FLASH_VerifyProgram() API.");
    defmt::unwrap!(flash.verify_program(dest_addr, &write_buf));

    let mut read_buf = [0u8; 512];
    let read_offset = dest_addr - pflash_block_base;
    defmt::unwrap!(flash.blocking_read(read_offset, &mut read_buf));
    defmt::assert_eq!(read_buf, write_buf);

    defmt::info!(
        "\n Successfully programmed and verified location: 0x{:x} -> 0x{:x} \n",
        dest_addr,
        dest_addr + (write_buf.len() as u32)
    );

    defmt::unwrap!(flash.blocking_erase(dest_addr, pflash_sector_size));

    // -----------------------------------------------------------------------
    // 5. NorFlash trait
    // -----------------------------------------------------------------------

    defmt::info!("\n Pflash using NorFlash trait");
    let dest_offset: u32 = pflash_total_size - ((SECTOR_INDEX_FROM_END + 1) * pflash_sector_size);
    let dest_addr: u32 = pflash_block_base + dest_offset;

    defmt::info!("\n Erase another sector of flash using NorFlash trait",);
    defmt::unwrap!(flash.erase(dest_offset, dest_offset + pflash_sector_size));

    defmt::info!("\n Calling flash_verify_erase_sector() API for the second sector.");
    defmt::unwrap!(flash.verify_erase_sector(dest_addr, pflash_sector_size));
    defmt::info!(
        "\n Successfully erased sector: 0x{:x} -> 0x{:x}\n",
        dest_addr,
        dest_addr + pflash_sector_size
    );

    defmt::info!("\n Calling NorFlash::write().");
    defmt::unwrap!(flash.write(dest_offset, &write_buf));

    defmt::info!("\n Calling FLASH_VerifyProgram() API.");
    defmt::unwrap!(flash.verify_program(dest_addr, &write_buf));

    let mut read_buf = [0u8; 512];
    defmt::unwrap!(flash.read(dest_offset, &mut read_buf));
    defmt::assert_eq!(read_buf, write_buf);

    defmt::info!(
        "\n Successfully programmed and verified location: 0x{:x} -> 0x{:x} using NorFlash trait ",
        dest_addr,
        dest_addr + (write_buf.len() as u32)
    );

    // -----------------------------------------------------------------------
    // 5. 16 byte write
    // -----------------------------------------------------------------------

    let dest_offset: u32 = pflash_total_size - ((SECTOR_INDEX_FROM_END + 1) * pflash_sector_size) + 512;
    defmt::info!("\n Single 16 byte write using NorFlash trait: {:x}", dest_offset);
    let dest_addr: u32 = pflash_block_base + dest_offset;

    let mut write_buf = [0xFFu8; 32];
    for i in 0..16 {
        write_buf[i] = i as u8;
    }

    defmt::unwrap!(flash.write(dest_offset, &write_buf[..16]));

    let mut read_buf = [0u8; 32];
    defmt::unwrap!(flash.read(dest_offset, &mut read_buf));
    defmt::assert_eq!(read_buf, write_buf);

    defmt::info!(
        "\n Successfully programmed and verified location: 0x{:x} -> 0x{:x} using NorFlash trait ",
        dest_addr,
        dest_addr + (write_buf.len() as u32)
    );

    defmt::unwrap!(flash.blocking_erase(
        pflash_total_size - ((SECTOR_INDEX_FROM_END + 1) * pflash_sector_size),
        pflash_sector_size
    ));

    defmt::info!("\n End of PFlash Example! \n");

    loop {
        cortex_m::asm::wfi();
    }
}
