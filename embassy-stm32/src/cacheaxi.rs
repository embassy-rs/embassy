//! AXI cache (CACHEAXI)
//!
//! CACHEAXI speeds up AXI memory accesses by caching them in on-chip RAM. It has
//! no independent clock gate on N6, so [`Cacheaxi::new()`] does not call
//! `rcc::enable_and_reset`.

use embassy_hal_internal::Peri;

use crate::pac;
use crate::peripherals::CACHEAXI;

fn regs() -> pac::cacheaxi::Cacheaxi {
    pac::CACHEAXI
}

/// Read-side performance counter selection.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadMonitor {
    Hit,
    Miss,
    Both,
}

impl ReadMonitor {
    fn hit(self) -> bool {
        matches!(self, ReadMonitor::Hit | ReadMonitor::Both)
    }

    fn miss(self) -> bool {
        matches!(self, ReadMonitor::Miss | ReadMonitor::Both)
    }
}

/// Write-side performance counter selection.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WriteMonitor {
    Hit,
    Miss,
    Both,
}

impl WriteMonitor {
    fn hit(self) -> bool {
        matches!(self, WriteMonitor::Hit | WriteMonitor::Both)
    }

    fn miss(self) -> bool {
        matches!(self, WriteMonitor::Miss | WriteMonitor::Both)
    }
}

/// AXI cache driver.
pub struct Cacheaxi<'d> {
    _peri: Peri<'d, CACHEAXI>,
}

impl<'d> Cacheaxi<'d> {
    /// Create a new CACHEAXI driver. The cache is left disabled; call [`Self::enable()`] to turn it on.
    pub fn new(_peri: Peri<'d, CACHEAXI>) -> Self {
        Self { _peri }
    }

    /// Enable the cache.
    pub fn enable(&mut self) {
        regs().cr1().modify(|w| w.set_en(true));
    }

    /// Disable the cache and wait until `EN` reads back as cleared.
    pub fn disable(&mut self) {
        regs().fcr().write(|w| w.set_cbsyendf(true));
        regs().cr1().modify(|w| w.set_en(false));
        while regs().cr1().read().en() {}
    }

    /// Returns whether the cache is currently enabled.
    pub fn is_enabled(&self) -> bool {
        regs().cr1().read().en()
    }

    /// Invalidate the entire cache, blocking until the operation completes.
    pub fn invalidate(&mut self) {
        if !regs().sr().read().busyf() {
            regs().cr1().modify(|w| w.set_cacheinv(true));
        }
        while regs().sr().read().busyf() {}
        regs().fcr().write(|w| w.set_cbsyendf(true));
    }

    /// Start the given read-side performance counter(s).
    pub fn start_read_monitors(&mut self, monitor: ReadMonitor) {
        regs().cr1().modify(|w| {
            if monitor.hit() {
                w.set_rhitmen(true);
            }
            if monitor.miss() {
                w.set_rmissmen(true);
            }
        });
    }

    /// Stop the given read-side performance counter(s).
    pub fn stop_read_monitors(&mut self, monitor: ReadMonitor) {
        regs().cr1().modify(|w| {
            if monitor.hit() {
                w.set_rhitmen(false);
            }
            if monitor.miss() {
                w.set_rmissmen(false);
            }
        });
    }

    /// Reset the given read-side performance counter(s) to zero.
    pub fn reset_read_monitors(&mut self, monitor: ReadMonitor) {
        regs().cr1().modify(|w| {
            if monitor.hit() {
                w.set_rhitmrst(true);
            }
            if monitor.miss() {
                w.set_rmissmrst(true);
            }
        });
        regs().cr1().modify(|w| {
            if monitor.hit() {
                w.set_rhitmrst(false);
            }
            if monitor.miss() {
                w.set_rmissmrst(false);
            }
        });
    }

    /// Start the given write-side performance counter(s).
    pub fn start_write_monitors(&mut self, monitor: WriteMonitor) {
        regs().cr1().modify(|w| {
            if monitor.hit() {
                w.set_whitmen(true);
            }
            if monitor.miss() {
                w.set_wmissmen(true);
            }
        });
    }

    /// Stop the given write-side performance counter(s).
    pub fn stop_write_monitors(&mut self, monitor: WriteMonitor) {
        regs().cr1().modify(|w| {
            if monitor.hit() {
                w.set_whitmen(false);
            }
            if monitor.miss() {
                w.set_wmissmen(false);
            }
        });
    }

    /// Reset the given write-side performance counter(s) to zero.
    pub fn reset_write_monitors(&mut self, monitor: WriteMonitor) {
        regs().cr1().modify(|w| {
            if monitor.hit() {
                w.set_whitmrst(true);
            }
            if monitor.miss() {
                w.set_wmissmrst(true);
            }
        });
        regs().cr1().modify(|w| {
            if monitor.hit() {
                w.set_whitmrst(false);
            }
            if monitor.miss() {
                w.set_wmissmrst(false);
            }
        });
    }

    /// Current read-hit count.
    pub fn read_hit_count(&self) -> u32 {
        regs().rhmonr().read().rhitmon()
    }

    /// Current read-miss count.
    pub fn read_miss_count(&self) -> u32 {
        regs().rmmonr().read().rmissmon()
    }

    /// Current write-hit count.
    pub fn write_hit_count(&self) -> u32 {
        regs().whmonr().read().whitmon()
    }

    /// Current write-miss count.
    pub fn write_miss_count(&self) -> u32 {
        regs().wmmonr().read().wmissmon()
    }

    /// Returns `true` and clears the flag if a cache error has occurred since the last call.
    pub fn take_error(&mut self) -> bool {
        let err = regs().sr().read().errf();
        if err {
            regs().fcr().write(|w| w.set_cerrf(true));
        }
        err
    }
}
