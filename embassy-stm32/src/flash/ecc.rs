//! Flash reads that report uncorrectable ECC errors as [`Error::Ecc`].
//!
//! The flash corrects single-bit errors transparently. An uncorrectable (double) error, as
//! left behind by a program or erase interrupted by a reset or power loss, is reported in a
//! way that crashes the firmware unless the read is prepared for it:
//!
//! - H7, H7A/B and H7R/S: the read gets a bus error, which escalates to a HardFault. Reads
//!   are done with `FAULTMASK` set and `SCB.CCR.BFHFNMIGN`, which makes the CPU ignore precise
//!   data bus faults while it runs at priority -1, and the flash error flag is checked
//!   afterwards.
//! - All other chips with ECC: the read raises an NMI. embassy-stm32 defines the NMI handler.
//!   During a read through the driver it clears the flags and records the error. Any other NMI
//!   panics.
//!
//! On H5 and C5, the EDATA area and the OTP area have 16-bit words with their own ECC. They
//! can only be read with 16- or 32-bit accesses. A word that was never programmed since it was
//! erased has no valid ECC: reading it gives a double ECC error with all bits set, which these
//! reads return as erased data. For these areas the NMI can be masked (`SBS_ECCNMIR`), so they
//! are read with the NMI masked, checking the flags after each word. The mask doesn't apply to
//! the main flash.

use super::Error;
#[cfg(any(flash_ecc_bus_error, flash_ecc_nmi))]
use super::{cache, family};

/// Reads `bytes.len()` bytes of flash at `address`.
#[cfg(not(any(flash_ecc_bus_error, flash_ecc_nmi)))]
pub(super) fn read(address: u32, bytes: &mut [u8]) -> Result<(), Error> {
    copy(address, bytes);
    Ok(())
}

/// Reads `bytes.len()` bytes of flash at `address`. Returns [`Error::Ecc`] if the flash
/// reports an uncorrectable ECC error.
#[cfg(any(flash_ecc_bus_error, flash_ecc_nmi))]
pub(super) fn read(address: u32, bytes: &mut [u8]) -> Result<(), Error> {
    // The OTP area, and the read-only area after it, have 16-bit words.
    #[cfg(any(flash_v5_h5, flash_v5_h50))]
    if matches!(crate::_generated::FLASH_OTP_BASE, Some(otp) if address >= otp) {
        return read_half_words(address, bytes);
    }

    let mut address = address;
    let mut bytes = bytes;
    while !bytes.is_empty() {
        let n = ((CHUNK - address % CHUNK) as usize).min(bytes.len());
        let (chunk, rest) = bytes.split_at_mut(n);
        if !read_chunk(address, chunk) {
            recover(address);
            return Err(Error::Ecc);
        }
        address += n as u32;
        bytes = rest;
    }
    Ok(())
}

fn copy(address: u32, bytes: &mut [u8]) {
    let flash = unsafe { core::slice::from_raw_parts(address as *const u8, bytes.len()) };
    bytes.copy_from_slice(flash);
}

/// Reads are split in chunks of this size, aligned to it, to keep the time with interrupts
/// masked short. It's the size of the largest flash word.
#[cfg(any(flash_ecc_bus_error, flash_ecc_nmi))]
const CHUNK: u32 = 32;

/// After an error, a new read of the failing flash word may be served from the flash read
/// buffer or a cache without an ECC check, returning bad data without an error (see the ECC
/// section of the reference manuals). Read another flash word of the same bank to replace the
/// read buffer, and flush the caches.
#[cfg(any(flash_ecc_bus_error, flash_ecc_nmi))]
fn recover(address: u32) {
    let word = address & !(CHUNK - 1);
    // Whether this read fails doesn't matter.
    let _ = read_chunk(word ^ (2 * CHUNK), &mut [0; 4]);
    // Atomically, so it doesn't interleave with the cache handling of a program or erase
    // operation running in another task.
    critical_section::with(|_| cache::after_op(&cache::before_op(), (word, CHUNK)));
}

/// Reads a chunk with `FAULTMASK` and `BFHFNMIGN` set. Returns `false` on a double ECC error.
#[cfg(flash_ecc_bus_error)]
fn read_chunk(address: u32, bytes: &mut [u8]) -> bool {
    use core::arch::asm;

    use cortex_m::asm::{dsb, isb};
    use cortex_m::peripheral::SCB;

    /// Ignore precise data bus faults at priority -1 and -2.
    const BFHFNMIGN: u32 = 1 << 8;
    /// CFSR bits of a precise data bus fault (write 1 to clear).
    const BFSR_PRECISERR: u32 = 1 << 9;
    const BFSR_BFARVALID: u32 = 1 << 15;

    let scb = unsafe { &*SCB::PTR };
    // `Faultmask::Inactive` means the FAULTMASK bit is set.
    let faultmask_set = cortex_m::register::faultmask::read().is_inactive();
    unsafe {
        // Raise the execution priority to -1. This also masks all interrupts.
        asm!("cpsid f");
        let ccr = scb.ccr.read();
        scb.ccr.write(ccr | BFHFNMIGN);
        family::ecc_clear();
        dsb();
        isb();

        copy(address, bytes);
        // Wait for the reads to complete before checking the flags.
        dsb();
        let ok = !family::ecc_double_error();
        if !ok {
            // The ignored fault is still recorded in BFSR/BFAR, clear it.
            scb.cfsr.write(BFSR_PRECISERR | BFSR_BFARVALID);
        }

        scb.ccr.write(ccr);
        dsb();
        isb();
        if !faultmask_set {
            asm!("cpsie f");
        }
        ok
    }
}

/// Set while the driver reads flash.
#[cfg(flash_ecc_nmi)]
static READING: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
/// Set by the NMI handler when it handled a double ECC error during a read.
#[cfg(flash_ecc_nmi)]
static ECC_ERROR: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// Reads a chunk, with the NMI handler armed. Returns `false` on a double ECC error.
#[cfg(flash_ecc_nmi)]
fn read_chunk(address: u32, bytes: &mut [u8]) -> bool {
    use core::sync::atomic::Ordering::Relaxed;
    use core::sync::atomic::{Ordering, compiler_fence};

    use cortex_m::asm::{dsb, isb};

    critical_section::with(|_| {
        // The hardware doesn't flag a double error while a previous error is still flagged.
        family::ecc_clear();
        ECC_ERROR.store(false, Relaxed);
        READING.store(true, Relaxed);
        compiler_fence(Ordering::SeqCst);
        dsb();

        copy(address, bytes);
        // Reading a flash register waits for the reads to complete. If one of them raised the
        // NMI, wait for the handler to run.
        dsb();
        let flagged = family::ecc_double_error();
        isb();
        let mut error = ECC_ERROR.load(Relaxed);
        if flagged && !error {
            for _ in 0..1000 {
                compiler_fence(Ordering::SeqCst);
                error = ECC_ERROR.load(Relaxed);
                if error {
                    break;
                }
            }
            if !error {
                // The NMI didn't come: clear the flags here.
                family::ecc_clear();
                error = true;
            }
        }

        compiler_fence(Ordering::SeqCst);
        READING.store(false, Relaxed);
        !error
    })
}

/// Reads `bytes.len()` bytes at `address` from an area with 16-bit words (EDATA, OTP), with
/// 16-bit accesses. Words that were not programmed since they were erased read as `0xFFFF`.
#[cfg(any(flash_v5_h5, flash_v5_h50, flash_v5_c5))]
pub(super) fn read_half_words(address: u32, bytes: &mut [u8]) -> Result<(), Error> {
    if bytes.is_empty() {
        return Ok(());
    }
    let end = address + bytes.len() as u32;
    half_word_reads(|| {
        for word in (address & !1..end).step_by(2) {
            let data = read_half_word(word)?.unwrap_or(0xFFFF).to_le_bytes();
            for (i, &b) in data.iter().enumerate() {
                let a = word + i as u32;
                if (address..end).contains(&a) {
                    bytes[(a - address) as usize] = b;
                }
            }
        }
        Ok(())
    })
}

/// Reads the 16-bit word at `address` of an area with 16-bit words. Returns `None` if the word
/// was not programmed since it was erased.
///
/// Must be called inside [`half_word_reads`].
#[cfg(any(flash_v5_h5, flash_v5_h50, flash_v5_c5))]
pub(super) fn read_half_word(address: u32) -> Result<Option<u16>, Error> {
    // In a critical section: while the flag is set, the hardware doesn't flag (or raise the NMI
    // for) other double errors, and another read through the driver would clear it.
    critical_section::with(|_| {
        // The hardware doesn't flag a double error while a previous error is still flagged.
        family::ecc_clear();
        let data = unsafe { core::ptr::read_volatile(address as *const u16) };
        cortex_m::asm::dsb();
        if !family::ecc_double_error() {
            return Ok(Some(data));
        }
        let data = family::ecc_data();
        family::ecc_clear();
        // An erased word has all bits set, including the ECC bits.
        if data == 0xFFFF { Ok(None) } else { Err(Error::Ecc) }
    })
}

/// Runs `f` with the double ECC error NMI of the areas with 16-bit words masked, and the
/// instruction cache disabled. These areas must not be cached: a cache line fill would read
/// words that the driver doesn't check, and on C5 it's a HardFault.
#[cfg(any(flash_v5_h5, flash_v5_h50, flash_v5_c5))]
pub(super) fn half_word_reads<R>(f: impl FnOnce() -> R) -> R {
    let eccnmir = crate::pac::SYSCFG.eccnmir();
    let masked = eccnmir.read().eccnmi_mask_en();
    eccnmir.write(|w| w.set_eccnmi_mask_en(true));
    let state = cache::before_op();
    let res = f();
    cache::after_op(&state, (0, 0));
    eccnmir.write(|w| w.set_eccnmi_mask_en(masked));
    res
}

/// NMI handler. A double ECC error raised by a flash read of the driver makes the read return
/// [`Error::Ecc`]. Any other NMI panics.
#[cfg(all(feature = "rt", flash_ecc_nmi))]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
unsafe extern "C" fn NonMaskableInt() {
    use core::sync::atomic::Ordering::Relaxed;

    if READING.load(Relaxed) && family::ecc_double_error() {
        family::ecc_clear();
        ECC_ERROR.store(true, Relaxed);
    } else {
        panic!("NMI");
    }
}
