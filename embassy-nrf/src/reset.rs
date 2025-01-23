//! Reset

#![macro_use]

use bitflags::bitflags;
use nrf_pac::reset::regs::Resetreas;
#[cfg(not(feature = "_nrf5340-net"))]
use nrf_pac::reset::vals::Forceoff;

use crate::chip::pac::RESET;

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    /// Bitflag representation of the `RESETREAS` register
    pub struct ResetReason: u32 {
        /// Reset Pin
        const RESETPIN = 1;
        /// Application watchdog timer 0
        const DOG0 = 1 << 1;
        /// Application CTRL-AP
        const CTRLAP = 1 << 2;
        /// Application soft reset
        const SREQ = 1 << 3;
        /// Application CPU lockup
        const LOCKUP = 1 << 4;
        /// Wakeup from System OFF when wakeup is triggered by DETECT signal from GPIO
        const OFF = 1 << 5;
        /// Wakeup from System OFF when wakeup is triggered by ANADETECT signal from LPCOMP
        const LPCOMP = 1 << 6;
        /// Wakeup from System OFF when wakeup is triggered by entering the Debug Interface mode
        const DIF = 1 << 7;
        /// Network soft reset
        #[cfg(feature = "_nrf5340-net")]
        const LSREQ = 1 << 16;
        /// Network CPU lockup
        #[cfg(feature = "_nrf5340-net")]
        const LLOCKUP = 1 << 17;
        /// Network watchdog timer
        #[cfg(feature = "_nrf5340-net")]
        const LDOG = 1 << 18;
        /// Force-OFF reset from application core
        #[cfg(feature = "_nrf5340-net")]
        const MFORCEOFF = 1 << 23;
        /// Wakeup from System OFF mode due to NFC field being detected
        const NFC = 1 << 24;
        /// Application watchdog timer 1
        const DOG1 = 1 << 25;
        /// Wakeup from System OFF mode due to VBUS rising into valid range
        const VBUS = 1 << 26;
        /// Network CTRL-AP
        #[cfg(feature = "_nrf5340-net")]
        const LCTRLAP = 1 << 27;
    }
}

/// Reads the bitflag of the reset reasons
pub fn read_reasons() -> ResetReason {
    ResetReason::from_bits_retain(RESET.resetreas().read().0)
}

/// Resets the reset reasons
pub fn clear_reasons() {
    RESET.resetreas().write(|w| *w = Resetreas(ResetReason::all().bits()));
}

/// Returns if the network core is held in reset
#[cfg(not(feature = "_nrf5340-net"))]
pub fn network_core_held() -> bool {
    RESET.network().forceoff().read().forceoff() == Forceoff::HOLD
}

/// Releases the network core from the FORCEOFF state
#[cfg(not(feature = "_nrf5340-net"))]
pub fn release_network_core() {
    RESET.network().forceoff().write(|w| w.set_forceoff(Forceoff::RELEASE));
}

/// Holds the network core in the FORCEOFF state
#[cfg(not(feature = "_nrf5340-net"))]
pub fn hold_network_core() {
    RESET.network().forceoff().write(|w| w.set_forceoff(Forceoff::HOLD));
}
