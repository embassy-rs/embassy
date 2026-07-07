//! Cache maintenance for NPU inference.
//!
//! Two caches sit between the memories and the compute:
//!
//! - **CACHEAXI** ("NPU cache"): a write-back cache on the NPU's AXI masters,
//!   used for weights/activations in external memory. Not described in
//!   `stm32-metapac`, so it is driven here through a minimal hand-written
//!   register map (offsets from RM0486 / the STM32N6 CMSIS device header).
//! - **The Cortex-M55 data cache**: when input/output buffers live in
//!   MCU-cacheable memory, the CPU cache must be cleaned before the NPU reads
//!   and invalidated before the CPU reads back results. The generated network
//!   code calls these operations `LL_ATON_Cache_MCU_*` — the equivalents here
//!   are [`mcu_clean_range`] / [`mcu_invalidate_range`].
//!
//! The functions mirror `npu_cache.c` + `stm32n6xx_hal_cacheaxi.c` from the
//! ST examples.

use crate::pac;

/// CACHEAXI register base (non-secure alias):
/// `AHB5PERIPH_BASE_NS (0x4802_0000) + 0x000B_FC00`.
const CACHEAXI_BASE: u32 = 0x480D_FC00;

const CR1: u32 = CACHEAXI_BASE + 0x000;
const SR: u32 = CACHEAXI_BASE + 0x004;
const FCR: u32 = CACHEAXI_BASE + 0x00C;
const CR2: u32 = CACHEAXI_BASE + 0x100;
const CMDRSADDRR: u32 = CACHEAXI_BASE + 0x104;
const CMDREADDRR: u32 = CACHEAXI_BASE + 0x108;

const CR1_EN: u32 = 1 << 0;
const CR1_CACHEINV: u32 = 1 << 1;
const SR_BUSYF: u32 = 1 << 0;
const SR_BSYENDF: u32 = 1 << 1;
const SR_BUSYCMDF: u32 = 1 << 3;
const SR_CMDENDF: u32 = 1 << 4;
const FCR_CBSYENDF: u32 = 1 << 1;
const FCR_CERRF: u32 = 1 << 2;
const FCR_CCMDENDF: u32 = 1 << 4;
const CR2_STARTCMD: u32 = 1 << 0;
const CMD_CLEAN: u32 = 0b01 << 1;
const CMD_CLEAN_INVALIDATE: u32 = 0b11 << 1;

#[inline(always)]
fn read(addr: u32) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

#[inline(always)]
fn write(addr: u32, val: u32) {
    unsafe { core::ptr::write_volatile(addr as *mut u32, val) }
}

/// Enable the NPU cache (CACHEAXI): switches on its RCC clock, pulses its
/// reset and sets the enable bit. Call once before running inferences that
/// touch cacheable memory pools (external flash / PSRAM).
pub fn npu_cache_enable() {
    pac::RCC.ahb5enr().modify(|w| w.set_npucacheen(true));
    pac::RCC.ahb5rstr().modify(|w| w.set_npucacherst(true));
    pac::RCC.ahb5rstr().modify(|w| w.set_npucacherst(false));

    // The first enable attempt commonly observes BUSYF; wait it out.
    while read(SR) & SR_BUSYF != 0 {}
    write(CR1, read(CR1) | CR1_EN);
}

/// Disable the NPU cache and gate its clock.
pub fn npu_cache_disable() {
    write(CR1, read(CR1) & !CR1_EN);
    pac::RCC.ahb5enr().modify(|w| w.set_npucacheen(false));
}

/// Invalidate the entire NPU cache. Blocks until done.
pub fn npu_cache_invalidate() {
    while read(SR) & (SR_BUSYF | SR_BUSYCMDF) != 0 {}
    write(FCR, FCR_CBSYENDF);
    write(CR1, read(CR1) | CR1_CACHEINV);
    while read(SR) & SR_BSYENDF == 0 {}
    write(FCR, FCR_CBSYENDF);
}

fn npu_cache_command(cmd: u32, start: u32, len: u32) {
    if len == 0 {
        return;
    }
    while read(SR) & (SR_BUSYF | SR_BUSYCMDF) != 0 {}
    write(FCR, FCR_CBSYENDF | FCR_CCMDENDF | FCR_CERRF);
    write(CMDRSADDRR, start);
    write(CMDREADDRR, start + len - 1);
    write(CR2, cmd);
    write(CR2, cmd | CR2_STARTCMD);
    while read(SR) & SR_CMDENDF == 0 {}
    write(FCR, FCR_CCMDENDF);
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
        write(op_reg, addr);
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
