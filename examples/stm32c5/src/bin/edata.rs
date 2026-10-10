#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::flash::{Blocking, EDataBank, Flash};
use panic_probe as _;

/// The first half-word in the EDATA page used by this example.
const TEST_OFFSET: u32 = 0;
/// EDATA page erased before every test run.
const TEST_PAGE: u8 = 0;

/// The NMI handler only treats an ECCD as expected while this probe is active.
const EDATA_MODE_IDLE: u8 = 0;
const EDATA_MODE_READING: u8 = 1;

/// Coordination between `try_read_first_word` and the NMI handler.
///
/// A virgin (erased and never programmed) EDATA half-word has no ECC bits.
/// Reading it raises a FLASH double-ECC error (ECCD), which is delivered as an
/// NMI. The handler records that expected exception here so the main code can
/// identify the word as virgin after the exception returns.
static EDATA_ECCD_MODE: AtomicU8 = AtomicU8::new(EDATA_MODE_IDLE);
static EDATA_ECCD_SEEN: AtomicBool = AtomicBool::new(false);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut f = Flash::new_blocking(p.FLASH);

    info!("STM32C5 EDATA example");

    if !f.edata_is_enabled() {
        panic!("EDATA is disabled; enable EDATA_EN in the option bytes");
    }

    info!("Erasing EDATA bank 1, page 0...");
    unwrap!(f.edata_erase_page(embassy_stm32::flash::EDataBank::Bank1, TEST_PAGE));

    let expected: [u16; 4] = [0x1234, 0x5678, 0xabcd, 0xef01];

    // A virgin EDATA half-word must be probed through the NMI path. See
    // `try_read_first_word` for why an ordinary read is not sufficient.
    info!("Probing the erased EDATA word...");
    match try_read_first_word(&mut f) {
        Ok(ProbeWord::Virgin) => {
            info!("EDATA word is virgin; programming test data...");
            unwrap!(f.edata_write_u16_slice(embassy_stm32::flash::EDataBank::Bank1, TEST_OFFSET, &expected));
        },
        // The page was just erased. A value here means erase or ECC handling
        // did not behave as expected, so do not continue with the test.
        Ok(ProbeWord::Value(v)) => panic!("erased EDATA word unexpectedly contains {v:#06x}"),
        Err(_) => panic!("failed to probe erased EDATA word"),
    }

    info!("Reading EDATA...");
    let mut actual = [0u16; 4];
    unwrap!(f.edata_read_u16_slice(embassy_stm32::flash::EDataBank::Bank1, TEST_OFFSET, &mut actual));

    info!("Read values: {:?}", actual);
    assert_eq!(actual, expected);

    info!("EDATA test succeeded!");

    loop {
        cortex_m::asm::wfi();
    }
}

enum ProbeWord {
    /// The NMI handler observed the expected ECCD from a virgin word.
    Virgin,
    /// A programmed word can be read normally without an ECCD.
    Value(u16),
}

/// Cortex-M NMI vector for FLASH double-ECC errors.
///
/// Reading a virgin EDATA word intentionally arrives here. Any other NMI is
/// unexpected and leaves the CPU halted in the debugger rather than silently
/// continuing after a possible data-integrity failure.
#[unsafe(no_mangle)]
pub extern "C" fn NonMaskableInt() {
    if handle_expected_edata_eccd() {
        defmt::warn!("expected EDATA ECCD");
    } else {
        defmt::error!("unexpected FLASH/EDATA ECC double error");
        loop {
            cortex_m::asm::bkpt();
        }
    }
}

/// Reads the first EDATA word while recognizing the expected ECCD of a virgin
/// word. The NMI handler sets `EDATA_ECCD_SEEN` before returning to this code.
fn try_read_first_word(flash: &mut Flash<'_, Blocking>) -> Result<ProbeWord, ()> {
    // Clear a result from an earlier probe before NMI can classify a new ECCD.
    EDATA_ECCD_SEEN.store(false, Ordering::Release);
    // Publish that an ECCD from the following read is intentional.
    EDATA_ECCD_MODE.store(EDATA_MODE_READING, Ordering::Release);
    let result = flash.edata_read_u16(EDataBank::Bank1, 0);
    // NMI exception return precedes this instruction; subsequent ECCDs are
    // therefore not part of this probe.
    EDATA_ECCD_MODE.store(EDATA_MODE_IDLE, Ordering::Relaxed);
    if EDATA_ECCD_SEEN.swap(false, Ordering::AcqRel) {
        return Ok(ProbeWord::Virgin);
    }
    Ok(ProbeWord::Value(result.map_err(|_| ())?))
}

/// Returns `true` only for the ECCD deliberately caused by the active probe.
///
/// `Acquire` pairs with the probe's `Release` store of `READING`; the `Release`
/// store of `SEEN` below is observed by the probe's `AcqRel` swap.
fn handle_expected_edata_eccd() -> bool {
    if EDATA_ECCD_MODE.load(Ordering::Acquire) == EDATA_MODE_IDLE {
        return false;
    }

    let status = embassy_stm32::pac::FLASH.eccdetr().read();

    // The NMI may also be caused by a main-FLASH ECCD. Only accept the exact
    // double-ECC condition for EDATA while a probe is active.
    if !status.eccd() || !status.edata_ecc() {
        return false;
    }

    // Publish the outcome before clearing the peripheral's write-one-to-clear
    // status bit and returning from the NMI.
    EDATA_ECCD_SEEN.store(true, Ordering::Release);
    // ECCD is write-one-to-clear in the FLASH ECC detection register.
    embassy_stm32::pac::FLASH.eccdetr().write(|w| w.set_eccd(true));

    true
}
