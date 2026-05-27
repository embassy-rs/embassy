#![no_std]
#![no_main]

use defmt::{assert, assert_eq, info, panic, unwrap};
use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::config::Config;
use hal::flexspi::{ClockConfig as FlexspiClockConfig, Flexspi, NorFlash};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[path = "../flexspi_common.rs"]
mod flexspi_common;

use flexspi_common::{
    FLASH_CONFIG, FLASH_PAGE_SIZE, FLASH_SECTOR_SIZE, READ_LEN_PROBES, SELF_TEST_BYTES, SELF_TEST_PAGES,
    SELF_TEST_SECTORS, check_erased, check_pattern, fill_pattern, pattern_byte,
};

const FLASH_BASE: u32 = (1000 * 0x1000) as u32;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Config::default());

    info!("FlexSPI blocking self-test");

    let flexspi = unwrap!(Flexspi::new_blocking(
        p.FLEXSPI0,
        p.P3_0,
        p.P3_1,
        p.P3_6,
        p.P3_7,
        p.P3_8,
        p.P3_9,
        p.P3_10,
        p.P3_11,
        FlexspiClockConfig::default(),
        FLASH_CONFIG,
    ));

    let mut flash = NorFlash::from_flexspi(flexspi);

    // 1) Vendor ID is idempotent.
    let id_a = unwrap!(flash.blocking_vendor_id());
    let id_b = unwrap!(flash.blocking_vendor_id());
    assert_eq!(id_a, id_b, "vendor id changed between back-to-back reads");
    info!("Vendor ID: 0x{:02x}", id_a);

    // 2) Erase the test span sector by sector.
    info!(
        "Erasing {=u32} sectors at 0x{=u32:08x} ({=u32} bytes)",
        SELF_TEST_SECTORS, FLASH_BASE, SELF_TEST_BYTES
    );
    for s in 0..SELF_TEST_SECTORS {
        let addr = FLASH_BASE + s * FLASH_SECTOR_SIZE as u32;
        unwrap!(flash.blocking_erase_sector(addr));
    }

    // 3) Verify all-0xFF using a sector-sized read, repeated for the whole span.
    let mut sector = [0u8; FLASH_SECTOR_SIZE];
    for s in 0..SELF_TEST_SECTORS {
        let addr = FLASH_BASE + s * FLASH_SECTOR_SIZE as u32;
        sector.fill(0);
        unwrap!(flash.blocking_read(addr, &mut sector));
        if let Some(bad) = check_erased(addr, &sector) {
            panic!("erase verify failed at 0x{:08x}", bad);
        }
    }

    // 3a) Read with a variety of small/odd lengths from the start of the span,
    //     all of which must still see 0xFF.
    let mut probe = [0u8; 1024];
    for &len in READ_LEN_PROBES {
        probe[..len].fill(0);
        unwrap!(flash.blocking_read(FLASH_BASE, &mut probe[..len]));
        if let Some(bad) = check_erased(FLASH_BASE, &probe[..len]) {
            panic!("erased read len {=usize} failed at 0x{=u32:08x}", len, bad);
        }
    }

    // 3b) Read at unaligned offsets within the span.
    for &off in &[1u32, 3, 17, 255, 256, 257, 4095, 4096, 4097] {
        let addr = FLASH_BASE + off;
        probe[..64].fill(0);
        unwrap!(flash.blocking_read(addr, &mut probe[..64]));
        if let Some(bad) = check_erased(addr, &probe[..64]) {
            panic!("unaligned erased read at 0x{:08x} failed at 0x{:08x}", addr, bad);
        }
    }
    info!("Erase verified across {=u32} bytes", SELF_TEST_BYTES);

    // 4) Program every page in the span with a deterministic pattern.
    info!("Programming {=u32} pages", SELF_TEST_PAGES);
    let mut page = [0u8; FLASH_PAGE_SIZE];
    for p_idx in 0..SELF_TEST_PAGES {
        let addr = FLASH_BASE + p_idx * FLASH_PAGE_SIZE as u32;
        fill_pattern(addr, &mut page);
        unwrap!(flash.blocking_page_program(addr, &page));
    }

    // 5) Read back the entire span sector by sector and verify the pattern.
    for s in 0..SELF_TEST_SECTORS {
        let addr = FLASH_BASE + s * FLASH_SECTOR_SIZE as u32;
        sector.fill(0);
        unwrap!(flash.blocking_read(addr, &mut sector));
        if let Some(bad) = check_pattern(addr, &sector) {
            panic!(
                "pattern verify failed at 0x{:08x} (got 0x{:02x}, want 0x{:02x})",
                bad,
                sector[(bad - addr) as usize],
                pattern_byte(bad),
            );
        }
    }

    // 5a) Re-read with the same length probes to make sure the read path
    //     works at non-sector sizes after programming.
    for &len in READ_LEN_PROBES {
        probe[..len].fill(0);
        unwrap!(flash.blocking_read(FLASH_BASE, &mut probe[..len]));
        if let Some(bad) = check_pattern(FLASH_BASE, &probe[..len]) {
            panic!("post-program read len {=usize} mismatch at 0x{=u32:08x}", len, bad);
        }
    }

    // 5b) Read across a sector boundary (last 32B of sector 0 + first 32B of sector 1).
    let cross_addr = FLASH_BASE + FLASH_SECTOR_SIZE as u32 - 32;
    let mut cross = [0u8; 64];
    unwrap!(flash.blocking_read(cross_addr, &mut cross));
    if let Some(bad) = check_pattern(cross_addr, &cross) {
        panic!("cross-sector read mismatch at 0x{:08x}", bad);
    }
    info!("Pattern verified across {=u32} bytes", SELF_TEST_BYTES);

    // 6) Partial-page write: erase last sector, write only 100 bytes of its
    //    first page, verify those bytes match the pattern and the rest of
    //    the page (and rest of the sector) stays at 0xFF.
    let last_sector = FLASH_BASE + (SELF_TEST_SECTORS - 1) * FLASH_SECTOR_SIZE as u32;
    unwrap!(flash.blocking_erase_sector(last_sector));
    let partial_len = 100usize;
    let mut partial = [0u8; FLASH_PAGE_SIZE];
    fill_pattern(last_sector, &mut partial);
    unwrap!(flash.blocking_page_program(last_sector, &partial[..partial_len]));

    sector.fill(0);
    unwrap!(flash.blocking_read(last_sector, &mut sector));
    if let Some(bad) = check_pattern(last_sector, &sector[..partial_len]) {
        panic!("partial-page write mismatch at 0x{:08x}", bad);
    }
    if let Some(bad) = check_erased(last_sector + partial_len as u32, &sector[partial_len..]) {
        panic!("partial-page tail not erased at 0x{:08x}", bad);
    }

    // 7) Erase only sector 1 and confirm sectors 0/2 still hold the pattern.
    let middle = FLASH_BASE + FLASH_SECTOR_SIZE as u32;
    unwrap!(flash.blocking_erase_sector(middle));
    sector.fill(0);
    unwrap!(flash.blocking_read(middle, &mut sector));
    assert!(
        check_erased(middle, &sector).is_none(),
        "middle sector not fully erased"
    );
    sector.fill(0);
    unwrap!(flash.blocking_read(FLASH_BASE, &mut sector));
    assert!(
        check_pattern(FLASH_BASE, &sector).is_none(),
        "sector 0 corrupted after erasing sector 1"
    );
    let third = FLASH_BASE + 2 * FLASH_SECTOR_SIZE as u32;
    sector.fill(0);
    unwrap!(flash.blocking_read(third, &mut sector));
    assert!(
        check_pattern(third, &sector).is_none(),
        "sector 2 corrupted after erasing sector 1"
    );

    info!("FlexSPI blocking self-test PASSED");

    loop {
        Timer::after_secs(1).await;
        info!("FlexSPI demo heartbeat");
    }
}
