//! Low-level high resolution timer driver.

use embassy_hal_internal::Peri;

use super::*;
use crate::rcc;
use crate::time::Hertz;

impl<'d, T: Instance> Drop for HrTimer<'d, T> {
    fn drop(&mut self) {
        rcc::disable::<T>();
    }
}

/// Low-level HRTIM driver.
pub struct HrTimer<'d, T: Instance> {
    _tim: Peri<'d, T>,
}

impl<'d, T: Instance> HrTimer<'d, T> {
    /// Create a new timer driver.
    pub fn new(tim: Peri<'d, T>) -> Self {

        rcc::enable_and_reset::<T>();

        Self { _tim: tim }
    }

    /// Get access to the timer registers.
    pub fn regs(&self) -> crate::pac::hrtim::Hrtim {
        T::regs()
    }

    /// Calibrate the HRTIM DLL
    pub fn calibrate(&mut self) {
        let regs = self.regs();

        // Enable and and stabilize the DLL
        regs.dllcr().modify(|w| {
            w.set_cal(true);
        });

        trace!("hrtim: wait for dll calibration");
        while !regs.isr().read().dllrdy() {}

        trace!("hrtim: dll calibration complete");

        self.set_periodic_calibration();
    }

    /// Enable and set periodic calibration
    pub fn set_periodic_calibration(&mut self) {
        let regs = self.regs();

        // Cal must be disabled before we can enable it
        regs.dllcr().modify(|w| {
            w.set_cal(false);
        });

        regs.dllcr().modify(|w| {
            w.set_calen(true);
            w.set_calrte(11);
        });
    }

    /// Get the clock frequency of the timer (before prescaler is applied).
    pub fn get_clock_frequency(&self) -> Hertz {
        T::frequency()
    }
}
