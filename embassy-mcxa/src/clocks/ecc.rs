//! RAM A clock prerequisite for ECC diagnostics. Live SRAM is never reset or gated off.

use super::{CLOCKS, ClockError, PoweredClock, WakeGuard};
use crate::pac;

/// Enables the RAM A clock without disturbing live SRAM and retains a wake vote.
pub(crate) fn enable_ecc_ram_a_clock() -> Result<WakeGuard, ClockError> {
    critical_section::with(|cs| {
        let clocks = CLOCKS.borrow_ref(cs);
        let clocks = clocks.as_ref().ok_or(ClockError::NeverInitialized)?;
        clocks.ensure_cpu_system_clk_active(&PoweredClock::NormalEnabledDeepSleepDisabled)?;
        let guard = WakeGuard::new();
        #[cfg(feature = "_mcxa2xx")]
        let cc = pac::MRCC0.mrcc_glb_cc2();
        #[cfg(feature = "_mcxa5xx")]
        let cc = pac::MRCC0.mrcc_glb_cc3();
        cc.modify(|w| w.set_rama(true));
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
        if !cc.read().rama() {
            // Do not undo the enable: this clock also supplies live memory.
            return Err(ClockError::BadConfig {
                clock: "RAMA",
                reason: "clock enable readback failed",
            });
        }
        Ok(guard)
    })
}
