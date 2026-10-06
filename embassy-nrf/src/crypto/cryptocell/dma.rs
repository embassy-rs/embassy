//! DIN/DOUT DMA path shared by the CryptoCell symmetric engines (AES, HASH, CHACHA).
//!
//! There is one DIN and one DOUT DMA, so one operation runs at a time. The owner of the
//! `CRYPTO_SYMMETRIC` peripheral, or the lock of the `embassy-crypto` drivers, guarantees
//! that.

use core::sync::atomic::{Ordering, compiler_fence};

use crate::pac;
use crate::pac::cc_ctl::vals::{CryptoBusyStatus, HashBusyStatus};

/// Cryptographic data flow through the engines.
pub(crate) type Flow = pac::cc_ctl::vals::Mode;

/// Waits until all engines are idle.
pub(crate) fn wait_idle() {
    while pac::CC_CTL.crypto_busy().read().status() == CryptoBusyStatus::Busy {}
    while pac::CC_CTL.hash_busy().read().status() == HashBusyStatus::Busy {}
}

/// Clears all pending CryptoCell interrupt flags.
pub(crate) fn clear_irq() {
    pac::CC_HOST_RGF.icr().write(|w| w.0 = 0xFFFF_FFFF);
}

/// Masks the DMA interrupt sources, so polling them here never raises the CRYPTOCELL IRQ.
fn mask_dma_irqs() {
    use pac::cc_host_rgf::vals::{DoutToMemMask, DoutToSramMask, MemToDinMask, SramToDinMask};
    pac::CC_HOST_RGF.imr().modify(|w| {
        w.set_sram_to_din_mask(SramToDinMask::IrqDisable);
        w.set_dout_to_sram_mask(DoutToSramMask::IrqDisable);
        w.set_mem_to_din_mask(MemToDinMask::IrqDisable);
        w.set_dout_to_mem_mask(DoutToMemMask::IrqDisable);
    });
}

/// Prepares the DMA path for a transfer through the engines selected by `flow`.
///
/// Call before configuring the engine registers, then start the transfer with [`transfer`].
pub(crate) fn prepare(flow: Flow) {
    wait_idle();
    mask_dma_irqs();
    clear_irq();
    pac::CC_CTL.crypto_ctl().write(|w| w.set_mode(flow));
}

/// Runs one transfer of `len` bytes from `input` through the engines and waits for it.
///
/// The engine output goes to `output`, unless it is null.
///
/// # Safety
///
/// `input` must point to `len` readable bytes in RAM. `output` must be null or point to
/// `len` writable bytes in RAM.
pub(crate) unsafe fn transfer(input: *const u8, output: *mut u8, len: usize) {
    debug_assert!(crate::util::slice_in_ram(core::ptr::slice_from_raw_parts(input, len)));
    if len == 0 {
        return;
    }

    compiler_fence(Ordering::SeqCst);

    if !output.is_null() {
        pac::CC_DOUT.dst_mem_addr().write_value(output as u32);
        pac::CC_DOUT.dst_mem_size().write(|w| w.set_size(len as u32));
    }
    // Writing the source size starts the transfer.
    pac::CC_DIN.src_mem_addr().write_value(input as u32);
    pac::CC_DIN.src_mem_size().write(|w| w.set_size(len as u32));

    if output.is_null() {
        while !pac::CC_HOST_RGF.irr().read().mem_to_din_int() {}
    } else {
        while !pac::CC_HOST_RGF.irr().read().dout_to_mem_int() {}
    }
    wait_idle();

    compiler_fence(Ordering::SeqCst);
    clear_irq();
}
