//! Reset reason
//!
//! MCXA families keep the most recent reset reason in the SRS
//! register of the CMC block. This lets users understand why the MCU
//! has reset and take appropriate corrective actions if required.

/// Reads the most recent reset reason from the Core Mode Controller
/// (CMC).
pub fn reset_reason() -> ResetReason {
    let regs = unsafe { &*crate::pac::Cmc::steal() };

    let srs = regs.srs().read();

    if srs.wakeup().is_enabled() {
        ResetReason::WakeUp
    } else if srs.por().bit_is_set() {
        ResetReason::Por
    } else if srs.vd().bit_is_set() {
        ResetReason::VoltageDetect
    } else if srs.warm().bit_is_set() {
        ResetReason::Warm
    } else if srs.fatal().bit_is_set() {
        ResetReason::Fatal
    } else if srs.pin().bit_is_set() {
        ResetReason::Pin
    } else if srs.dap().bit_is_set() {
        ResetReason::Dap
    } else if srs.rstack().bit_is_set() {
        ResetReason::ResetAckTimeout
    } else if srs.lpack().bit_is_set() {
        ResetReason::LowPowerAckTimeout
    } else if srs.scg().bit_is_set() {
        ResetReason::SystemClockGeneration
    } else if srs.wwdt0().bit_is_set() {
        ResetReason::Wwdt0
    } else if srs.sw().bit_is_set() {
        ResetReason::Software
    } else if srs.lockup().bit_is_set() {
        ResetReason::Lockup
    } else if srs.cdog0().bit_is_set() {
        ResetReason::Cdog0
    } else if srs.cdog1().bit_is_set() {
        ResetReason::Cdog1
    } else if srs.jtag().bit_is_set() {
        ResetReason::Jtag
    } else {
        ResetReason::Tamper
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
