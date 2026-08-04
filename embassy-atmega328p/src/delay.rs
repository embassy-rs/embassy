//! Cycle-counted blocking delays.

use embedded_hal::delay::DelayNs;

/// Blocking delay provider using the selected CPU clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct Delay;

impl Delay {
    /// Creates a delay provider.
    pub const fn new() -> Self {
        Self
    }

    /// Blocks for at least `microseconds` microseconds, excluding interrupts.
    pub fn delay_us(&mut self, microseconds: u32) {
        let cycles_per_us = crate::CPU_HZ / 1_000_000;
        delay_cycles_saturating(microseconds.saturating_mul(cycles_per_us));
    }

    /// Blocks for at least `milliseconds` milliseconds, excluding interrupts.
    pub fn delay_ms(&mut self, milliseconds: u32) {
        for _ in 0..milliseconds {
            self.delay_us(1_000);
        }
    }
}

impl DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        if ns == 0 {
            return;
        }
        let cycles = (u64::from(ns) * u64::from(crate::CPU_HZ)).div_ceil(1_000_000_000);
        delay_cycles_saturating(cycles.min(u64::from(u32::MAX)) as u32);
    }
}

fn delay_cycles_saturating(cycles: u32) {
    if cycles != 0 {
        avr_device::asm::delay_cycles(cycles);
    }
}
