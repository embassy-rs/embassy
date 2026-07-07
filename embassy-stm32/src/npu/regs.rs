//! Minimal register map for the ST Neural-ART accelerator ("ATON") on STM32N6.
//!
//! The NPU is not described in `stm32-metapac` (ST does not ship SVD data for
//! it), so the handful of registers the runtime needs are defined here by
//! hand. Names, offsets and bit positions are taken from the generated
//! `ATON.h` of the STM32N6 (Neural-ART machine description, ATON IP v1.x) as
//! shipped with ST Edge AI / X-CUBE-AI.
//!
//! Only the units the epoch-controller execution model touches are described:
//!
//! - `CLKCTRL`   — ATON-internal clock controller (unit clock gates)
//! - `INTCTRL`   — ATON interrupt controller (4 output lines -> NVIC NPU0..3)
//! - `BUSIF`     — 2 bus interface units (must be enabled for any traffic)
//! - `STRENG`    — 10 streaming engines (only the IRQ register, for acks)
//! - `EPOCHCTRL` — the epoch controller, which executes command blobs

#![allow(dead_code)]

/// ATON register block base address (non-secure alias, matches the address
/// space used by `stm32-metapac` for the N6 family).
/// `NPU_BASE_NS = PERIPH_BASE_NS (0x4000_0000) + 0x0802_0000 + 0x000C_0000`.
pub const ATON_BASE: u32 = 0x480E_0000;

pub const CLKCTRL_BASE: u32 = ATON_BASE; // unit 0 of 1
pub const INTCTRL_BASE: u32 = ATON_BASE + 0x1000; // unit 0 of 1
pub const BUSIF_BASE: u32 = ATON_BASE + 0x2000; // + 0x1000 * n, n < 2
pub const STRENG_BASE: u32 = ATON_BASE + 0x5000; // + 0x1000 * n, n < 10
pub const EPOCHCTRL_BASE: u32 = ATON_BASE + 0x1E000; // unit 0 of 1

/// Number of bus interface units.
pub const BUSIF_NUM: usize = 2;
/// Number of streaming engines.
pub const STRENG_NUM: usize = 10;

#[inline(always)]
pub fn read(addr: u32) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

#[inline(always)]
pub fn write(addr: u32, val: u32) {
    unsafe { core::ptr::write_volatile(addr as *mut u32, val) }
}

#[inline(always)]
pub fn modify(addr: u32, f: impl FnOnce(u32) -> u32) {
    write(addr, f(read(addr)));
}

// ── CLKCTRL ─────────────────────────────────────────────────────────────────

/// Clock controller CTRL register. Bit 0 = EN, bit 1 = CLR.
pub const CLKCTRL_CTRL: u32 = CLKCTRL_BASE + 0x00;
/// Group-A clock gates (always-on infrastructure clocks), bits [31:0].
pub const CLKCTRL_AGATES0: u32 = CLKCTRL_BASE + 0x08;
/// Group-A clock gates, upper word.
pub const CLKCTRL_AGATES1: u32 = CLKCTRL_BASE + 0x0C;
/// Group-B clock gates (one per accelerator unit; 27 used on N6).
pub const CLKCTRL_BGATES: u32 = CLKCTRL_BASE + 0x10;

pub const CLKCTRL_CTRL_EN: u32 = 1 << 0;
pub const CLKCTRL_CTRL_CLR: u32 = 1 << 1;

// ── INTCTRL ─────────────────────────────────────────────────────────────────

/// Interrupt controller CTRL. Bit 0 = EN, bit 1 = CLR, bit 30 = CONFCLR.
pub const INTCTRL_CTRL: u32 = INTCTRL_BASE + 0x00;
/// Latched interrupt status (one bit per ATON interrupt source).
pub const INTCTRL_INTREG: u32 = INTCTRL_BASE + 0x08;
/// Software interrupt set.
pub const INTCTRL_INTSET: u32 = INTCTRL_BASE + 0x0C;
/// Interrupt clear (write 1 to clear latched bits).
pub const INTCTRL_INTCLR: u32 = INTCTRL_BASE + 0x10;

pub const INTCTRL_CTRL_EN: u32 = 1 << 0;
pub const INTCTRL_CTRL_CLR: u32 = 1 << 1;
pub const INTCTRL_CTRL_CONFCLR: u32 = 1 << 30;

/// OR-mask for interrupt line `line` (0..=3). A `1` bit *disables* the
/// corresponding source on that line; sources left at `0` raise the line
/// whenever pending ("OR" semantics).
#[inline(always)]
pub const fn intctrl_intormsk(line: usize) -> u32 {
    INTCTRL_BASE + 0x14 + 4 * line as u32
}

/// AND-mask for interrupt line `line` (0..=3). The line fires when *all*
/// sources left enabled (bit = `0`) are pending simultaneously; the reset
/// value of all-ones disables the AND group entirely.
#[inline(always)]
pub const fn intctrl_intandmsk(line: usize) -> u32 {
    INTCTRL_BASE + 0x24 + 4 * line as u32
}

// ── Interrupt source bit assignment on the (single) INTCTRL, 32 sources ─────

/// Streaming engine completion events, bits 0..=9.
pub const INT_STRENG_EVT_MASK: u32 = 0x0000_03FF;
/// Streaming engine error events, bits 10..=19.
pub const INT_STRENG_ERR_MASK: u32 = 0x000F_FC00;
pub const INT_STRENG_ERR_SHIFT: u32 = 10;
/// Bus interface error interrupts (BUSIF0 = bit 25, BUSIF1 = bit 26).
pub const INT_BUSIF_ERR_MASK: u32 = 0x0600_0000;
pub const INT_BUSIF_ERR_SHIFT: u32 = 25;
/// Epoch controller "epoch interrupt" (EPOCH_INT instruction / end of blob).
pub const INT_ECTRL_EVT: u32 = 1 << 28;
/// Epoch controller missing-acknowledge error.
pub const INT_ECTRL_NOACK: u32 = 1 << 29;
/// Epoch controller error (illegal opcode, bus error, ...).
pub const INT_ECTRL_ERR: u32 = 1 << 30;

// ── BUSIF ───────────────────────────────────────────────────────────────────

#[inline(always)]
pub const fn busif_ctrl(n: usize) -> u32 {
    BUSIF_BASE + 0x1000 * n as u32
}

#[inline(always)]
pub const fn busif_err(n: usize) -> u32 {
    BUSIF_BASE + 0x1000 * n as u32 + 0x10
}

pub const BUSIF_CTRL_EN: u32 = 1 << 0;

// ── STRENG ──────────────────────────────────────────────────────────────────

/// Streaming engine IRQ status register (write back the read value to ack).
#[inline(always)]
pub const fn streng_irq(n: usize) -> u32 {
    STRENG_BASE + 0x1000 * n as u32 + 0x3C
}

// ── EPOCHCTRL ───────────────────────────────────────────────────────────────

/// Epoch controller CTRL. Bit 0 = EN, bit 1 = CLR, bit 3 = SM (step mode),
/// bit 30 = CONFCLR, bit 31 = RUNNING (read-only).
pub const EPOCHCTRL_CTRL: u32 = EPOCHCTRL_BASE + 0x00;
/// Blob start address (must be 8-byte aligned, points at the blob magic).
pub const EPOCHCTRL_ADDR: u32 = EPOCHCTRL_BASE + 0x08;
/// Epoch controller IRQ status (write back the read value to ack).
pub const EPOCHCTRL_IRQ: u32 = EPOCHCTRL_BASE + 0x0C;
/// Label register (debug: label of the current/last executed blob section).
pub const EPOCHCTRL_LABEL: u32 = EPOCHCTRL_BASE + 0x1C;
/// Opcode counter (debug).
pub const EPOCHCTRL_BC: u32 = EPOCHCTRL_BASE + 0x20;

pub const EPOCHCTRL_CTRL_EN: u32 = 1 << 0;
pub const EPOCHCTRL_CTRL_CLR: u32 = 1 << 1;
pub const EPOCHCTRL_CTRL_SM: u32 = 1 << 3;
pub const EPOCHCTRL_CTRL_CONFCLR: u32 = 1 << 30;
pub const EPOCHCTRL_CTRL_RUNNING: u32 = 1 << 31;
