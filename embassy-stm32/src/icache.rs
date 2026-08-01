//! Instruction cache (ICACHE)
//!
//! ICACHE speeds up instruction fetches from external memories (e.g. OCTOSPI/XSPI-mapped
//! flash) by caching them in a small internal RAM. It has no independent clock gate — it's
//! always clocked whenever the CPU is — so unlike most other peripherals, [`Icache::new()`]
//! does not call `rcc::enable_and_reset`.
//!
//! Supported on STM32U5, U3, WBA, H5, L5 and N6. On N6 the cache doesn't support memory
//! remapping (no `CRR` registers), so [`Icache::enable_remap_region()`] and
//! [`Icache::disable_remap_region()`] aren't available there — those methods only exist on
//! chips whose ICACHE block has region-remap support.

use embassy_hal_internal::Peri;

use crate::pac;
use crate::pac::icache::vals::Waysel;
use crate::peripherals::ICACHE;

fn regs() -> pac::icache::Icache {
    pac::ICACHE
}

/// Cache set-associativity.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Associativity {
    /// Direct-mapped (1-way) cache.
    DirectMapped,
    /// N-way set-associative cache (hardware default).
    SetAssociative,
}

impl From<Associativity> for Waysel {
    fn from(a: Associativity) -> Self {
        match a {
            Associativity::DirectMapped => Waysel::DirectMapped,
            Associativity::SetAssociative => Waysel::NWaySetAssociative,
        }
    }
}

/// Which of the two independent hit/miss performance counters to operate on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Monitor {
    /// Hit counter only.
    Hit,
    /// Miss counter only.
    Miss,
    /// Both counters.
    Both,
}

impl Monitor {
    fn hit(self) -> bool {
        matches!(self, Monitor::Hit | Monitor::Both)
    }

    fn miss(self) -> bool {
        matches!(self, Monitor::Miss | Monitor::Both)
    }
}

/// Instruction cache driver.
pub struct Icache<'d> {
    _peri: Peri<'d, ICACHE>,
}

impl<'d> Icache<'d> {
    /// Create a new instruction cache driver. The cache is left disabled; call [`Self::enable()`]
    /// to turn it on.
    pub fn new(_peri: Peri<'d, ICACHE>) -> Self {
        Self { _peri }
    }

    /// Enable the cache.
    ///
    /// This always succeeds even if a cache maintenance operation is ongoing: the cache is
    /// bypassed until it completes.
    pub fn enable(&mut self) {
        regs().cr().modify(|w| w.set_en(true));
    }

    /// Disable the cache.
    ///
    /// Disabling automatically triggers a full cache invalidation; this waits for `EN` to read
    /// back as cleared, but not for that invalidation to finish (call [`Self::invalidate()`]
    /// afterwards if you need that guarantee).
    pub fn disable(&mut self) {
        // Clear any stale BSYENDF left over from a previous operation before disabling, since
        // disabling starts an automatic invalidation of its own.
        regs().fcr().write(|w| w.set_cbsyendf(true));
        regs().cr().modify(|w| w.set_en(false));
        while regs().cr().read().en() {}
    }

    /// Returns whether the cache is currently enabled.
    pub fn is_enabled(&self) -> bool {
        regs().cr().read().en()
    }

    /// Set the cache set-associativity.
    ///
    /// Only possible while the cache is disabled. Returns `false` (and does nothing) if the
    /// cache is currently enabled.
    pub fn set_associativity(&mut self, assoc: Associativity) -> bool {
        if regs().cr().read().en() {
            return false;
        }
        regs().cr().modify(|w| w.set_waysel(assoc.into()));
        true
    }

    /// Invalidate the entire cache content, blocking until the operation completes.
    ///
    /// Can be called whether the cache is enabled or disabled.
    pub fn invalidate(&mut self) {
        if !regs().sr().read().busyf() {
            regs().cr().modify(|w| w.set_cacheinv(true));
        }
        while regs().sr().read().busyf() {}
        regs().fcr().write(|w| w.set_cbsyendf(true));
    }

    /// Start the given performance counter(s). Use [`Self::reset_monitors()`] first if you want
    /// them to start counting from zero.
    pub fn start_monitors(&mut self, monitor: Monitor) {
        regs().cr().modify(|w| {
            if monitor.hit() {
                w.set_hitmen(true);
            }
            if monitor.miss() {
                w.set_missmen(true);
            }
        });
    }

    /// Stop the given performance counter(s). Their counts are retained.
    pub fn stop_monitors(&mut self, monitor: Monitor) {
        regs().cr().modify(|w| {
            if monitor.hit() {
                w.set_hitmen(false);
            }
            if monitor.miss() {
                w.set_missmen(false);
            }
        });
    }

    /// Reset the given performance counter(s) to zero.
    pub fn reset_monitors(&mut self, monitor: Monitor) {
        regs().cr().modify(|w| {
            if monitor.hit() {
                w.set_hitmrst(true);
            }
            if monitor.miss() {
                w.set_missmrst(true);
            }
        });
        regs().cr().modify(|w| {
            if monitor.hit() {
                w.set_hitmrst(false);
            }
            if monitor.miss() {
                w.set_missmrst(false);
            }
        });
    }

    /// Current cache hit count. Saturates (does not wrap) at `0xFFFF_FFFF`.
    pub fn hit_count(&self) -> u32 {
        regs().hmonr().read()
    }

    /// Current cache miss count. Saturates (does not wrap) at `0xFFFF`.
    pub fn miss_count(&self) -> u16 {
        regs().mmonr().read().missmon()
    }

    /// Returns `true` and clears the flag if a cache error (invalid maintenance operation) has
    /// occurred since the last call.
    pub fn take_error(&mut self) -> bool {
        let err = regs().sr().read().errf();
        if err {
            regs().fcr().write(|w| w.set_cerrf(true));
        }
        err
    }
}

#[cfg(any(icache_v1_3crr, icache_v1_4crr))]
mod region;
#[cfg(any(icache_v1_3crr, icache_v1_4crr))]
pub use region::*;
