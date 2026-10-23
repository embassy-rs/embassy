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

/// Set while the driver reads a chunk of flash.
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
                // The NMI didn't come (it can be masked on some chips): clear the flags here.
                family::ecc_clear();
                error = true;
            }
        }

        compiler_fence(Ordering::SeqCst);
        READING.store(false, Relaxed);
        !error
    })
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
