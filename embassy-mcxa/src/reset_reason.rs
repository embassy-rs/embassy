//! Reset reason
//!
//! MCXA families keep the most recent reset reason in the SRS
//! register of the CMC block. This lets users understand why the MCU
//! has reset and take appropriate corrective actions if required.
//!
//! The reset reason bits are cached for the during of this boot,
//! allowing the user to query the reset reason as many times as
//! necessary.

use core::sync::atomic::{AtomicU32, Ordering};

static RESET_REASON: AtomicU32 = AtomicU32::new(0);

/// Reads the most recent reset reason from the Core Mode Controller
/// (CMC).
pub fn reset_reason() -> ResetReasonRaw {
    let regs = crate::pac::CMC;

    let reason = critical_section::with(|_| {
        let mut r = RESET_REASON.load(Ordering::Relaxed);

        if r == 0 {
            // Read status
            r = regs.srs().read().0;

            // Clear status
            regs.ssrs().modify(|w| w.0 = r);

            RESET_REASON.store(r, Ordering::Relaxed);
        }

        r
    });

    ResetReasonRaw(reason)
}

/// Raw reset reason bits. Can be queried or all reasons can be iterated over
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug)]
pub struct ResetReasonRaw(u32);

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug)]
pub struct ResetReasonRawIter(u32);

impl ResetReasonRaw {
    const MAP: &[(u32, ResetReason)] = &[
        (1 << 0, ResetReason::WakeUp),
        (1 << 1, ResetReason::Por),
        (1 << 2, ResetReason::VoltageDetect),
        (1 << 4, ResetReason::Warm),
        (1 << 5, ResetReason::Fatal),
        (1 << 8, ResetReason::Pin),
        (1 << 9, ResetReason::Dap),
        (1 << 10, ResetReason::ResetAckTimeout),
        (1 << 11, ResetReason::LowPowerAckTimeout),
        (1 << 12, ResetReason::SystemClockGeneration),
        (1 << 13, ResetReason::Wwdt0),
        (1 << 14, ResetReason::Software),
        (1 << 15, ResetReason::Lockup),
        (1 << 26, ResetReason::Cdog0),
        (1 << 27, ResetReason::Cdog1),
        (1 << 28, ResetReason::Jtag),
    ];

    /// Convert to an iterator of contained reset reasons
    pub fn into_iter(self) -> ResetReasonRawIter {
        ResetReasonRawIter(self.0)
    }

    /// Wake up
    #[inline]
    pub fn is_wakeup(&self) -> bool {
        (self.0 & (1 << 0)) != 0
    }

    /// Power-on Reset
    #[inline]
    pub fn is_por(&self) -> bool {
        (self.0 & (1 << 1)) != 0
    }

    /// Voltage detect
    #[inline]
    pub fn is_voltage_detect(&self) -> bool {
        (self.0 & (1 << 2)) != 0
    }

    /// Warm
    #[inline]
    pub fn is_warm(&self) -> bool {
        (self.0 & (1 << 4)) != 0
    }

    /// Fatal
    #[inline]
    pub fn is_fatal(&self) -> bool {
        (self.0 & (1 << 5)) != 0
    }

    /// Pin
    #[inline]
    pub fn is_pin(&self) -> bool {
        (self.0 & (1 << 8)) != 0
    }

    /// DAP
    #[inline]
    pub fn is_dap(&self) -> bool {
        (self.0 & (1 << 9)) != 0
    }

    /// Reset ack timeout
    #[inline]
    pub fn is_reset_ack_timeout(&self) -> bool {
        (self.0 & (1 << 10)) != 0
    }

    /// Low power ack timeout
    #[inline]
    pub fn is_low_power_ack_timeout(&self) -> bool {
        (self.0 & (1 << 11)) != 0
    }

    /// System clock generation
    #[inline]
    pub fn is_system_clock_generation(&self) -> bool {
        (self.0 & (1 << 12)) != 0
    }

    /// Watchdog 0
    #[inline]
    pub fn is_watchdog0(&self) -> bool {
        (self.0 & (1 << 13)) != 0
    }

    /// Software
    pub fn is_software(&self) -> bool {
        (self.0 & (1 << 14)) != 0
    }

    /// Lockup
    pub fn is_lockup(&self) -> bool {
        (self.0 & (1 << 15)) != 0
    }

    /// Code watchdog 0
    pub fn is_code_watchdog0(&self) -> bool {
        (self.0 & (1 << 26)) != 0
    }

    /// Code watchdog 1
    pub fn is_code_watchdog1(&self) -> bool {
        (self.0 & (1 << 27)) != 0
    }

    /// JTAG
    pub fn is_jtag(&self) -> bool {
        (self.0 & (1 << 28)) != 0
    }
}

impl Iterator for ResetReasonRawIter {
    type Item = ResetReason;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        for (mask, var) in ResetReasonRaw::MAP {
            // If the bit is set...
            if self.0 & mask != 0 {
                // clear the bit
                self.0 &= !mask;
                // and return the answer
                return Some(*var);
            }
        }

        // Shouldn't happen, but oh well.
        None
    }
}

/// Indicates the type and source of the most recent reset.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ResetReason {
    /// Tamper reset.
    Tamper,

    /// JTAG System Reset request.
    Jtag,

    /// Code Watchdog 0 reset.
    Cdog0,

    /// Code Watchdog 1 reset.
    Cdog1,

    /// Lockup reset.
    Lockup,

    /// Software reset.
    Software,

    /// Windowed Watchdog 0 reset.
    Wwdt0,

    /// System clock generation reset.
    SystemClockGeneration,

    /// Low Power Acknowledge Timeout reset.
    LowPowerAckTimeout,

    /// Reset Timeout.
    ResetAckTimeout,

    /// Debug Access Port reset.
    Dap,

    /// External assertion of RESET_b pin.
    Pin,

    /// Fatal reset.
    Fatal,

    /// Warm reset.
    Warm,

    /// Voltage detect reset.
    VoltageDetect,

    /// Power-on reset.
    Por,

    /// Wake-up reset.
    WakeUp,
}
