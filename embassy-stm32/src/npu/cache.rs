//! Cache maintenance for NPU inference.
//!
//! Two caches sit between the memories and the compute:
//!
//! - **CACHEAXI** ("NPU cache"): a write-back cache on the NPU's AXI masters,
//!   used for weights/activations in external memory. The peripheral base is
//!   modelled in `stm32-metapac` as `pac::CACHEAXI`. Clock and reset go
//!   through `pac::RCC`.
//! - **The Cortex-M55 data cache**: when input/output buffers live in
//!   MCU-cacheable memory, the CPU cache must be cleaned before the NPU reads
//!   and invalidated before the CPU reads back results. The generated network
//!   code calls these operations `LL_ATON_Cache_MCU_*` — the equivalents here
//!   are [`mcu_clean_range`] / [`mcu_invalidate_range`].
//!
//! The functions mirror `npu_cache.c` + `stm32n6xx_hal_cacheaxi.c` from the
//! ST examples.

use crate::pac;

/// Raw `(CR1, SR)` state for one-time bring-up diagnostics.
pub fn npu_cache_debug_state() -> (u32, u32) {
    (pac::CACHEAXI.cr1().read().0, pac::CACHEAXI.sr().read().0)
}

/// Reset and start the CACHEAXI read hit/miss monitors.
pub fn npu_cache_monitor_reset() {
    // ST's HAL resets a monitor by pulsing its enable mask shifted by two.
    let cr1 = pac::CACHEAXI.cr1().read();
    pac::CACHEAXI.cr1().write_value(pac::cacheaxi::regs::Cr1(
        cr1.0 | ((1 << 18) | (1 << 19)), // RHITMRST | RMISSMRST
    ));
    pac::CACHEAXI
        .cr1()
        .write_value(pac::cacheaxi::regs::Cr1(cr1.0 & !((1 << 18) | (1 << 19))));
    pac::CACHEAXI.cr1().modify(|w| {
        w.set_rhitmen(true);
        w.set_rmissmen(true);
    });
}

/// Return cumulative CACHEAXI `(read_hits, read_misses)` monitor values.
pub fn npu_cache_read_counters() -> (u32, u32) {
    (
        pac::CACHEAXI.rhmonr().read().rhitmon(),
        pac::CACHEAXI.rmmonr().read().rmissmon(),
    )
}

/// Enable the NPU cache (CACHEAXI): switches on its RCC clock, pulses its
/// reset and sets the enable bit. The parent NPU clock domain must already
/// be enabled and released from reset (for example by [`super::Npu::new`]),
/// matching ST's `NPU_Config()` ordering. Call once before running inferences
/// that touch cacheable memory pools (external flash / PSRAM).
pub fn npu_cache_enable() {
    // Exact equivalent of ST's `__HAL_RCC_CACHEAXI_CLK_ENABLE()`:
    // atomic set followed by AHB5ENR readback for the RCC startup delay.
    pac::RCC.ahb5ensr().write(|w| w.set_npucacheens(true));
    let _ = pac::RCC.ahb5enr().read();
    pac::RCC.ahb5rstsr().write(|w| w.set_npucachersts(true));
    pac::RCC.ahb5rstcr().write(|w| w.set_npucacherstc(true));

    // Drain the RCC clock/reset writes before accessing CACHEAXI. This does
    // not replace the requirement that the parent NPU clock domain was
    // enabled first.
    cortex_m::asm::dsb();
    cortex_m::asm::isb();

    // The first enable attempt commonly observes BUSYF; wait it out.
    while pac::CACHEAXI.sr().read().busyf() {}
    pac::CACHEAXI.cr1().modify(|w| w.set_en(true));
}

/// Disable the NPU cache and gate its clock.
pub fn npu_cache_disable() {
    pac::CACHEAXI.cr1().modify(|w| w.set_en(false));
    pac::RCC.ahb5encr().write(|w| w.set_npucacheenc(true));
}

/// Invalidate the entire NPU cache. Blocks until done.
pub fn npu_cache_invalidate() {
    while pac::CACHEAXI.sr().read().busyf() || pac::CACHEAXI.sr().read().busycmdf() {}
    pac::CACHEAXI.fcr().write(|w| w.set_cbsyendf(true));
    pac::CACHEAXI.cr1().modify(|w| w.set_cacheinv(true));
    while !pac::CACHEAXI.sr().read().bsyendf() {}
    pac::CACHEAXI.fcr().write(|w| w.set_cbsyendf(true));
}

const CMD_CLEAN: u8 = 0b01;
const CMD_CLEAN_INVALIDATE: u8 = 0b11;

fn npu_cache_command(cmd: u8, start: u32, len: u32) {
    if len == 0 {
        return;
    }
    while pac::CACHEAXI.sr().read().busyf() || pac::CACHEAXI.sr().read().busycmdf() {}
    pac::CACHEAXI.fcr().write(|w| {
        w.set_cbsyendf(true);
        w.set_ccmdendf(true);
        w.set_cerrf(true);
    });
    pac::CACHEAXI.cmdrsaddrr().write(|w| w.set_cmdstartaddr(start >> 6));
    pac::CACHEAXI
        .cmdreaddrr()
        .write(|w| w.set_cmdendaddr((start + len - 1) >> 6));
    pac::CACHEAXI.cr2().write(|w| w.set_cachecmd(cmd));
    pac::CACHEAXI.cr2().write(|w| {
        w.set_cachecmd(cmd);
        w.set_startcmd(true);
    });
    while !pac::CACHEAXI.sr().read().cmdendf() {}
    pac::CACHEAXI.fcr().write(|w| w.set_ccmdendf(true));
}

/// Write back (clean) an address range from the NPU cache to memory.
/// Equivalent of `LL_ATON_Cache_NPU_Clean_Range`.
pub fn npu_cache_clean_range(start: u32, len: u32) {
    npu_cache_command(CMD_CLEAN, start, len);
}

/// Write back and invalidate an address range in the NPU cache.
/// Equivalent of `LL_ATON_Cache_NPU_Clean_Invalidate_Range`.
pub fn npu_cache_clean_invalidate_range(start: u32, len: u32) {
    npu_cache_command(CMD_CLEAN_INVALIDATE, start, len);
}

// ── Cortex-M55 data-cache maintenance by address ────────────────────────────
//
// The cortex-m crate only exposes cache maintenance for ARMv7-M, so the
// (architecturally identical) ARMv8-M cache-maintenance registers are written
// directly here. All operate on 32-byte cache lines.

const DCACHE_LINE: u32 = 32;
/// Data cache invalidate by MVA to PoC.
const SCB_DCIMVAC: u32 = 0xE000_EF5C;
/// Data cache clean by MVA to PoC.
const SCB_DCCMVAC: u32 = 0xE000_EF68;
/// Data cache clean and invalidate by MVA to PoC.
const SCB_DCCIMVAC: u32 = 0xE000_EF70;

fn mcu_cache_op(op_reg: u32, start: u32, len: u32) {
    if len == 0 {
        return;
    }
    cortex_m::asm::dsb();
    let mut addr = start & !(DCACHE_LINE - 1);
    let end = start.wrapping_add(len);
    while addr < end {
        unsafe { core::ptr::write_volatile(op_reg as *mut u32, addr) };
        addr += DCACHE_LINE;
    }
    cortex_m::asm::dsb();
    cortex_m::asm::isb();
}

/// Clean (write back) the CPU data cache for `[start, start + len)`.
/// Equivalent of `LL_ATON_Cache_MCU_Clean_Range`. Call after the CPU writes
/// an input buffer and before the NPU reads it.
pub fn mcu_clean_range(start: u32, len: u32) {
    mcu_cache_op(SCB_DCCMVAC, start, len);
}

/// Invalidate the CPU data cache for `[start, start + len)`. Equivalent of
/// `LL_ATON_Cache_MCU_Invalidate_Range`. Call after the NPU writes an output
/// buffer and before the CPU reads it.
///
/// `start`/`len` should be 32-byte aligned; unaligned edges are cleaned and
/// invalidated to avoid corrupting neighbouring data.
pub fn mcu_invalidate_range(start: u32, len: u32) {
    if len == 0 {
        return;
    }
    // Protect partially covered lines at the edges.
    if start % DCACHE_LINE != 0 {
        mcu_cache_op(SCB_DCCIMVAC, start & !(DCACHE_LINE - 1), 1);
    }
    let end = start + len;
    if end % DCACHE_LINE != 0 {
        mcu_cache_op(SCB_DCCIMVAC, end & !(DCACHE_LINE - 1), 1);
    }
    mcu_cache_op(SCB_DCIMVAC, start, len);
}

/// Clean and invalidate the CPU data cache for `[start, start + len)`.
/// Equivalent of `LL_ATON_Cache_MCU_Clean_Invalidate_Range`.
pub fn mcu_clean_invalidate_range(start: u32, len: u32) {
    mcu_cache_op(SCB_DCCIMVAC, start, len);
}
