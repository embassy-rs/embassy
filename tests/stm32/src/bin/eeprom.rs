#![no_std]
#![no_main]

// required-features: eeprom

// The data EEPROM: writes of any alignment and length over existing data, writes that don't
// change anything, bounds checks, and the blocking, async and embedded-storage APIs.

#[path = "../common.rs"]
mod common;

use common::*;
use defmt::{assert, assert_eq};
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::flash::{self, EEPROM_SIZE, Eeprom, Error, Flash};
use embassy_stm32::mode::{Async, Blocking, Mode};
use embassy_time::{Duration, Instant};
use panic_probe as _;

teleprobe_meta::timeout!(120);

bind_interrupts!(struct Irqs {
    FLASH => flash::InterruptHandler;
});

/// The part of the EEPROM used by most of the test: from the start of the second EEPROM bank of the
/// chips with two. The CPU, which runs from the first flash bank, doesn't stall while it's written.
/// Long stalls make the debugger give up (on the L073, it shares the bus with the CPU), also when
/// it attaches to the board while it runs this test after a power cycle.
const AREA: usize = EEPROM_SIZE / 2;
const LEN: usize = 128;

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let mut p = init();
    info!("EEPROM_SIZE = {}", EEPROM_SIZE);

    {
        let mut e = Flash::new_blocking(p.FLASH.reborrow(), Default::default())
            .into_blocking_regions()
            .eeprom;
        assert_eq!(e.capacity(), EEPROM_SIZE);
        test_blocking(&mut e);
        test_errors(&mut e);
        test_storage(&mut e);
    }
    {
        let mut e = Flash::new(p.FLASH.reborrow(), Irqs, Default::default())
            .into_regions()
            .eeprom;
        test_async(&mut e).await;
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

fn pattern(offset: usize, seed: u8) -> u8 {
    let x = (offset as u32).wrapping_mul(0x9E37_79B1) >> 13;
    let b = (x as u8) ^ seed;
    if b == 0 { 0x5A ^ seed } else { b }
}

fn fill(buf: &mut [u8], offset: usize, seed: u8) {
    for (i, b) in buf.iter_mut().enumerate() {
        *b = pattern(offset + i, seed);
    }
}

/// Checks that the `LEN` bytes of the EEPROM at `AREA` are `expected`.
fn verify<M: Mode>(e: &mut Eeprom<'_, M>, expected: &[u8; LEN]) {
    let mut buf = [0u8; LEN];
    unwrap!(e.blocking_read(AREA as u32, &mut buf));
    if &buf != expected {
        let i = unwrap!(buf.iter().zip(expected).position(|(a, b)| a != b));
        defmt::panic!("mismatch at {}: read {:02x}, expected {:02x}", i, buf[i], expected[i]);
    }
}

/// Writes of any alignment and length, at the given offsets and lengths.
const WRITES: [(usize, usize); 8] = [(1, 1), (2, 1), (3, 1), (5, 2), (6, 3), (9, 7), (17, 30), (63, 2)];

fn test_blocking(e: &mut Eeprom<'_, Blocking>) {
    let mut shadow = [0u8; LEN];

    fill(&mut shadow, 0, 0x11);
    unwrap!(e.blocking_write(AREA as u32, &shadow));
    verify(e, &shadow);

    // Writing data the EEPROM already holds doesn't program anything, so it's much faster than
    // programming the words.
    fill(&mut shadow, 0, 0x22);
    let start = Instant::now();
    unwrap!(e.blocking_write(AREA as u32, &shadow));
    let program = start.elapsed();
    verify(e, &shadow);
    let start = Instant::now();
    unwrap!(e.blocking_write(AREA as u32, &shadow));
    let unchanged = start.elapsed();
    info!(
        "write {} bytes: {} us, again: {} us",
        LEN,
        program.as_micros(),
        unchanged.as_micros()
    );
    assert!(unchanged < program / 50);

    for (i, &(offset, len)) in WRITES.iter().enumerate() {
        let data = &mut shadow[offset..][..len];
        fill(data, offset, 0x33 + i as u8);
        unwrap!(e.blocking_write((AREA + offset) as u32, data));
        verify(e, &shadow);
    }

    // The erased value.
    shadow[..16].fill(0);
    unwrap!(e.blocking_write(AREA as u32, &shadow[..16]));
    verify(e, &shadow);

    // Across the two EEPROM banks of the chips that have two, and at the end.
    for offset in [AREA - 3, EEPROM_SIZE - 5] {
        let mut data = [0u8; 5];
        fill(&mut data, offset, 0x44);
        unwrap!(e.blocking_write(offset as u32, &data));
        let mut read = [0u8; 5];
        unwrap!(e.blocking_read(offset as u32, &mut read));
        assert_eq!(read, data);
    }
}

fn test_errors(e: &mut Eeprom<'_, Blocking>) {
    let end = EEPROM_SIZE as u32;
    assert_eq!(e.blocking_write(end, &[1]), Err(Error::OutOfBounds));
    assert_eq!(e.blocking_write(end - 1, &[1, 2]), Err(Error::OutOfBounds));
    assert_eq!(e.blocking_write(u32::MAX, &[1]), Err(Error::OutOfBounds));
    assert_eq!(e.blocking_read(end - 1, &mut [0; 2]), Err(Error::OutOfBounds));
    unwrap!(e.blocking_write(end, &[]));
    unwrap!(e.blocking_read(end, &mut []));
}

fn test_storage(e: &mut Eeprom<'_, Blocking>) {
    use embedded_storage::{ReadStorage, Storage};

    assert_eq!(ReadStorage::capacity(e), EEPROM_SIZE);
    let mut data = [0u8; 10];
    fill(&mut data, 7, 0x55);
    unwrap!(Storage::write(e, (AREA + 7) as u32, &data));
    let mut read = [0u8; 10];
    unwrap!(ReadStorage::read(e, (AREA + 7) as u32, &mut read));
    assert_eq!(read, data);
}

async fn test_async(e: &mut Eeprom<'_, Async>) {
    let mut shadow = [0u8; LEN];
    unwrap!(e.read(AREA as u32, &mut shadow).await);

    for (i, &(offset, len)) in WRITES.iter().enumerate() {
        let data = &mut shadow[offset..][..len];
        fill(data, offset, 0x66 + i as u8);
        unwrap!(e.write((AREA + offset) as u32, data).await);
        verify(e, &shadow);
    }

    fill(&mut shadow, 0, 0x77);
    let start = Instant::now();
    unwrap!(e.write(AREA as u32, &shadow).await);
    let program = start.elapsed();
    verify(e, &shadow);
    let start = Instant::now();
    unwrap!(e.write(AREA as u32, &shadow).await);
    let unchanged = start.elapsed();
    info!(
        "async write {} bytes: {} us, again: {} us",
        LEN,
        program.as_micros(),
        unchanged.as_micros()
    );
    assert!(unchanged < program / 50);
    assert!(program > Duration::from_millis(LEN as u64 / 4));

    assert_eq!(e.write(EEPROM_SIZE as u32, &[1]).await, Err(Error::OutOfBounds));
}
