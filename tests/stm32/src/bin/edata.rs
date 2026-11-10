#![no_std]
#![no_main]

// required-features: edata

// The EDATA area of both banks: erased locations, programming, error reporting (including
// uncorrectable ECC errors, and with the instruction cache enabled), and the blocking, async and
// NorFlash APIs.
//
// The test enables EDATA in the option bytes, and restores the previous settings at the end.

#[path = "../common.rs"]
mod common;

use common::*;
use defmt::{assert, assert_eq};
use embassy_executor::Spawner;
use embassy_stm32::flash::{self, EDATA_ERASE_SIZE, Edata, Error, Flash};
use embassy_stm32::icache::Icache;
use embassy_stm32::mode::{Async, Blocking};
use embassy_stm32::{bind_interrupts, pac};
use embassy_time::Instant;
use panic_probe as _;

teleprobe_meta::timeout!(120);

bind_interrupts!(struct Irqs {
    FLASH => flash::InterruptHandler;
});

/// EDATA sectors enabled in bank 1 and bank 2.
const SECTORS: [u32; 2] = [2, 8];
const SECTOR: u32 = EDATA_ERASE_SIZE as u32;
/// Sectors per bank that can be EDATA.
const MAX_SECTORS: u32 = 8;

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let mut p = init();

    let saved = edata_option_bytes();
    info!("EDATA option bytes: {:08x} {:08x}", saved[0], saved[1]);
    set_edata_option_bytes([option_byte(SECTORS[0]), option_byte(SECTORS[1])]);

    {
        let layout = Flash::new_blocking(p.FLASH.reborrow(), Default::default()).into_blocking_regions();
        let (mut e1, mut e2) = (layout.edata_bank1, layout.edata_bank2);
        assert_eq!(e1.capacity() as u32, SECTORS[0] * SECTOR);
        assert_eq!(e2.capacity() as u32, SECTORS[1] * SECTOR);

        info!("bank 1, blocking");
        test_blocking(&mut e1, 1);
        info!("bank 2, blocking");
        test_blocking(&mut e2, 2);
        test_errors(&mut e1);
        test_nor_flash(&mut e2);

        let ded = plant_ecc_errors(&mut e2);
        info!("ECC errors");
        check_ecc_errors(&mut e2, ded);

        info!("instruction cache enabled");
        let mut icache = Icache::new(p.ICACHE.reborrow());
        icache.enable();
        check_ecc_errors(&mut e2, ded);
        test_blocking(&mut e1, 1);
        assert!(icache.is_enabled());
        icache.disable();
    }

    {
        let layout = Flash::new(p.FLASH.reborrow(), Irqs, Default::default()).into_regions();
        let (mut e1, mut e2) = (layout.edata_bank1, layout.edata_bank2);
        info!("bank 1, async");
        test_async(&mut e1).await;
        info!("bank 2, async");
        test_async(&mut e2).await;
    }

    info!("bank 1 disabled");
    set_edata_option_bytes([0, option_byte(SECTORS[1])]);
    {
        let mut e1 = Flash::new_blocking(p.FLASH.reborrow(), Default::default())
            .into_blocking_regions()
            .edata_bank1;
        assert_eq!(e1.capacity(), 0);
        assert_eq!(e1.blocking_read(0, &mut [0; 2]), Err(Error::OutOfBounds));
        assert_eq!(e1.blocking_write(0, &[0; 2]), Err(Error::OutOfBounds));
        assert_eq!(e1.blocking_erase(0, SECTOR), Err(Error::OutOfBounds));
        unwrap!(e1.blocking_read(0, &mut []));
    }

    // Leave the user flash sectors used as EDATA erased.
    set_edata_option_bytes(saved);
    let mut f = Flash::new_blocking(p.FLASH.reborrow(), Default::default());
    for bank in [flash::FlashBank::Bank1, flash::FlashBank::Bank2] {
        let region = unwrap!(flash::get_flash_regions().iter().filter(|r| r.bank == bank).last());
        let end = region.end() - flash::FLASH_BASE as u32;
        unwrap!(f.blocking_erase(end - MAX_SECTORS * region.erase_size, end));
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

/// The EDATA option bytes, `EDATA1R` and `EDATA2R`.
fn edata_option_bytes() -> [u32; 2] {
    [pac::FLASH.edata1r_cur().read().0, pac::FLASH.edata2r_cur().read().0]
}

/// The EDATA option byte value that enables the last `sectors` sectors of a bank.
fn option_byte(sectors: u32) -> u32 {
    if sectors == 0 { 0 } else { 1 << 15 | (sectors - 1) }
}

/// Programs the EDATA option bytes. The change takes effect immediately.
fn set_edata_option_bytes(values: [u32; 2]) {
    let f = pac::FLASH;
    if edata_option_bytes() == values {
        return;
    }
    while f.nssr().read().bsy() {}
    if f.optcr().read().optlock() {
        f.optkeyr().write_value(0x0819_2A3B);
        f.optkeyr().write_value(0x4C5D_6E7F);
    }
    // All the option bytes are programmed: the others must keep their value.
    assert_eq!(f.optsr_prg().read().0, f.optsr_cur().read().0);
    assert_eq!(f.optsr2_prg().read().0, f.optsr2_cur().read().0);
    assert_eq!(f.nsbootr_prg().read().0, f.nsbootr_cur().read().0);
    assert_eq!(f.otpblr_prg().read().0, f.otpblr_cur().read().0);
    assert_eq!(f.wrp1r_prg().read().0, f.wrp1r_cur().read().0);
    assert_eq!(f.wrp2r_prg().read().0, f.wrp2r_cur().read().0);
    assert_eq!(f.hdp1r_prg().read().0, f.hdp1r_cur().read().0);
    assert_eq!(f.hdp2r_prg().read().0, f.hdp2r_cur().read().0);
    f.edata1r_prg().write_value(pac::flash::regs::Edata1r(values[0]));
    f.edata2r_prg().write_value(pac::flash::regs::Edata2r(values[1]));
    f.optcr().modify(|w| w.set_optstrt(true));
    while f.optcr().read().optstrt() {}
    let sr = f.nssr().read();
    f.optcr().modify(|w| w.set_optlock(true));
    if sr.optchangeerr() || sr.pgserr() {
        defmt::panic!("option byte change failed: NSSR={:08x}", sr.0);
    }
    assert_eq!(edata_option_bytes(), values);
}

fn pattern(offset: u32, seed: u8) -> u8 {
    let x = offset.wrapping_mul(0x9E37_79B1) >> 13;
    let b = (x as u8) ^ seed;
    if b == 0xFF { 0x5A ^ seed } else { b }
}

fn verify<M: embassy_stm32::mode::Mode>(e: &mut Edata<'_, M>, seed: Option<u8>) {
    let mut buf = [0u8; 256];
    for offset in (0..e.capacity() as u32).step_by(buf.len()) {
        unwrap!(e.blocking_read(offset, &mut buf));
        for (i, &b) in buf.iter().enumerate() {
            let expected = seed.map_or(0xFF, |s| pattern(offset + i as u32, s));
            if b != expected {
                defmt::panic!(
                    "mismatch at {}: read {:02x}, expected {:02x}",
                    offset + i as u32,
                    b,
                    expected
                );
            }
        }
    }
}

fn test_blocking(e: &mut Edata<'_, Blocking>, seed: u8) {
    let capacity = e.capacity() as u32;

    let start = Instant::now();
    unwrap!(e.blocking_erase(0, capacity));
    let erase = start.elapsed();
    let start = Instant::now();
    verify(e, None);
    let read_erased = start.elapsed();

    let mut buf = [0u8; 256];
    let start = Instant::now();
    for offset in (0..capacity).step_by(buf.len()) {
        for (i, b) in buf.iter_mut().enumerate() {
            *b = pattern(offset + i as u32, seed);
        }
        unwrap!(e.blocking_write(offset, &buf));
    }
    let write = start.elapsed();
    let start = Instant::now();
    verify(e, Some(seed));
    let read = start.elapsed();
    info!(
        "{} KB: erase {} ms, read erased {} ms, write {} ms, read {} ms",
        capacity / 1024,
        erase.as_millis(),
        read_erased.as_millis(),
        write.as_millis(),
        read.as_millis()
    );

    // Reads of any alignment and length.
    for (offset, len) in [(1, 1), (1, 2), (3, 5), (capacity - 1, 1), (capacity - 3, 3)] {
        let mut buf = [0u8; 8];
        let buf = &mut buf[..len];
        unwrap!(e.blocking_read(offset, buf));
        for (i, &b) in buf.iter().enumerate() {
            assert_eq!(b, pattern(offset + i as u32, seed));
        }
    }

    unwrap!(e.blocking_erase(0, capacity));
    verify(e, None);
}

fn test_errors(e: &mut Edata<'_, Blocking>) {
    let capacity = e.capacity() as u32;
    unwrap!(e.blocking_erase(0, SECTOR));

    assert_eq!(e.blocking_write(1, &[1, 2]), Err(Error::Unaligned));
    assert_eq!(e.blocking_write(0, &[1, 2, 3]), Err(Error::Unaligned));
    assert_eq!(e.blocking_write(capacity, &[1, 2]), Err(Error::OutOfBounds));
    assert_eq!(e.blocking_write(capacity - 2, &[1, 2, 3, 4]), Err(Error::OutOfBounds));
    assert_eq!(e.blocking_read(capacity - 1, &mut [0; 2]), Err(Error::OutOfBounds));
    assert_eq!(e.blocking_erase(1, SECTOR), Err(Error::Unaligned));
    assert_eq!(e.blocking_erase(0, SECTOR - 2), Err(Error::Unaligned));
    assert_eq!(e.blocking_erase(0, capacity + SECTOR), Err(Error::OutOfBounds));
    assert_eq!(e.blocking_erase(SECTOR, 0), Err(Error::OutOfBounds));

    // A word can't be written twice. A write that includes one doesn't write anything.
    unwrap!(e.blocking_write(2, &[0x12, 0x34]));
    assert_eq!(e.blocking_write(2, &[0x12, 0x34]), Err(Error::NotErased));
    assert_eq!(e.blocking_write(0, &[1, 2, 3, 4]), Err(Error::NotErased));
    let mut buf = [0; 4];
    unwrap!(e.blocking_read(0, &mut buf));
    assert_eq!(buf, [0xFF, 0xFF, 0x12, 0x34]);
    unwrap!(e.blocking_write(0, &[1, 2]));

    // A word written with 0xFFFF reads like an erased one, but is written.
    unwrap!(e.blocking_write(4, &[0xFF, 0xFF]));
    assert_eq!(e.blocking_write(4, &[0, 0]), Err(Error::NotErased));
    unwrap!(e.blocking_read(0, &mut buf));
    assert_eq!(buf, [1, 2, 0x12, 0x34]);

    // Writes of 16- and 32-bit words.
    unwrap!(e.blocking_erase(0, SECTOR));
    unwrap!(e.blocking_write(0, &[1, 2, 3, 4, 5, 6]));
    unwrap!(e.blocking_write(10, &[7, 8, 9, 10, 11, 12, 13, 14, 15, 16]));
    let mut buf = [0; 22];
    unwrap!(e.blocking_read(0, &mut buf));
    assert_eq!(
        buf,
        [
            1, 2, 3, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 0xFF, 0xFF
        ]
    );
    unwrap!(e.blocking_erase(0, SECTOR));
}

fn test_nor_flash(e: &mut Edata<'_, Blocking>) {
    use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};

    assert_eq!(<Edata<'_, Blocking> as NorFlash>::ERASE_SIZE, EDATA_ERASE_SIZE);
    assert_eq!(<Edata<'_, Blocking> as NorFlash>::WRITE_SIZE, 2);
    assert_eq!(ReadNorFlash::capacity(e), e.capacity());
    unwrap!(NorFlash::erase(e, SECTOR, 2 * SECTOR));
    unwrap!(NorFlash::write(e, SECTOR + 2, &[7, 8, 9, 10]));
    let mut buf = [0; 7];
    unwrap!(ReadNorFlash::read(e, SECTOR + 1, &mut buf));
    assert_eq!(buf, [0xFF, 7, 8, 9, 10, 0xFF, 0xFF]);
    unwrap!(NorFlash::erase(e, SECTOR, 2 * SECTOR));
}

async fn test_async(e: &mut Edata<'_, Async>) {
    use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash};

    let capacity = e.capacity() as u32;
    unwrap!(e.erase(0, capacity).await);
    verify(e, None);

    let mut buf = [0u8; 256];
    for offset in (0..capacity).step_by(buf.len()) {
        for (i, b) in buf.iter_mut().enumerate() {
            *b = pattern(offset + i as u32, 0x33);
        }
        unwrap!(e.write(offset, &buf).await);
    }
    verify(e, Some(0x33));
    assert_eq!(e.write(0, &[0, 0]).await, Err(Error::NotErased));
    let mut buf = [0u8; 3];
    unwrap!(e.read(5, &mut buf).await);
    assert_eq!(buf, [pattern(5, 0x33), pattern(6, 0x33), pattern(7, 0x33)]);

    unwrap!(NorFlash::erase(e, 0, SECTOR).await);
    unwrap!(NorFlash::write(e, 0, &[1, 2]).await);
    unwrap!(ReadNorFlash::read(e, 0, &mut buf).await);
    assert_eq!(buf, [1, 2, 0xFF]);

    unwrap!(e.erase(0, capacity).await);
    verify(e, None);
}

/// Address of `offset` in the EDATA area of bank 2.
fn bank2_address(offset: u32) -> u32 {
    let base = if flash::banks_swapped() {
        0x0900_0000
    } else {
        0x0900_C000
    };
    base + (MAX_SECTORS - SECTORS[1]) * SECTOR + offset
}

/// Programs a 16-bit word behind the driver's back: the driver doesn't allow writing a word
/// twice.
fn program(address: u32, value: u16) {
    let f = pac::FLASH;
    while f.nssr().read().bsy() {}
    if f.nscr().read().lock() {
        f.nskeyr().write_value(0x4567_0123);
        f.nskeyr().write_value(0xCDEF_89AB);
    }
    f.nscr().modify(|w| w.set_pg(true));
    unsafe { core::ptr::write_volatile(address as *mut u16, value) };
    cortex_m::asm::dsb();
    while f.nssr().read().bsy() || f.nssr().read().dbne() {}
    f.nscr().modify(|w| w.set_pg(false));
    f.nscr().modify(|w| w.set_lock(true));
}

/// Plants uncorrectable ECC errors in the first sector of `e` (bank 2), by programming words
/// twice so their ECC bits don't match. Returns the offset of a word with a double error.
fn plant_ecc_errors(e: &mut Edata<'_, Blocking>) -> u32 {
    unwrap!(e.blocking_erase(0, SECTOR));

    let (mut ded, mut corrected, mut consistent) = (None, 0, 0);
    let mut offset = 0;
    for i in 0..16 {
        for j in i + 1..16 {
            let (a, b) = (!(1u16 << i), !(1u16 << j));
            program(bank2_address(offset), a);
            program(bank2_address(offset), b);
            let mut buf = [0; 2];
            match e.blocking_read(offset, &mut buf) {
                Err(Error::Ecc) => {
                    ded.get_or_insert(offset);
                }
                Ok(()) if pac::FLASH.ecccorr().read().eccc() => corrected += 1,
                Ok(()) => {
                    assert_eq!(u16::from_le_bytes(buf), a & b);
                    consistent += 1;
                }
                Err(e) => defmt::panic!("read error {}", e),
            }
            offset += 2;
        }
    }
    info!(
        "planted {} words: {} uncorrectable, {} corrected, {} consistent",
        offset / 2,
        offset / 2 - corrected - consistent,
        corrected,
        consistent
    );
    unwrap!(ded)
}

fn check_ecc_errors(e: &mut Edata<'_, Blocking>, ded: u32) {
    let mut buf = [0u8; 2];
    // Every read of the word fails, also partial ones and long ones covering it.
    for _ in 0..3 {
        assert_eq!(e.blocking_read(ded, &mut buf), Err(Error::Ecc));
    }
    assert_eq!(e.blocking_read(ded + 1, &mut buf[..1]), Err(Error::Ecc));
    let mut long = [0u8; 64];
    assert_eq!(e.blocking_read(ded & !31, &mut long), Err(Error::Ecc));
    // It can't be written.
    assert_eq!(e.blocking_write(ded, &[0, 0]), Err(Error::NotErased));

    // The rest of the sector reads fine, and the error doesn't stick.
    let free = 1024;
    unwrap!(e.blocking_read(free, &mut long));
    assert!(long.iter().all(|&b| b == 0xFF));
    unwrap!(e.blocking_write(free, &[0x42, 0x43]));
    unwrap!(e.blocking_read(free, &mut buf));
    assert_eq!(buf, [0x42, 0x43]);

    // Erasing clears the errors.
    unwrap!(e.blocking_erase(0, SECTOR));
    unwrap!(e.blocking_read(ded, &mut buf));
    assert_eq!(buf, [0xFF, 0xFF]);

    // Plant one again for the next round.
    let new = plant_ecc_errors(e);
    assert_eq!(new, ded);
}
