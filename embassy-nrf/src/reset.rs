//! Reset

#![macro_use]

use bitflags::bitflags;
use nrf_pac::reset::regs::Resetreas;
#[cfg(feature = "_nrf5340-app")]
use nrf_pac::reset::vals::Forceoff;

use crate::pac::RESET;

#[cfg(feature = "_nrf5340")]
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

#[cfg(feature = "_nrf54l")]
bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    /// Bitflag representation of the `RESETREAS` register
    pub struct ResetReason: u32 {
        /// Reset Pin (CTRL-AP generating a pin reset has its own bit)
        const RESETPIN = 1;
        /// Application watchdog timer 0
        const DOG0 = 1 << 1;
        /// Application watchdog timer 1
        const DOG1 = 1 << 2;
        /// CTRL-AP soft reset
        const CTRLAPSOFT = 1 << 3;
        /// CTRL-AP hard reset
        const CTRLAPHARD = 1 << 4;
        /// CTRL-AP pin reset
        const CTRLAPPIN = 1 << 5;
        /// Application soft reset
        const SREQ = 1 << 6;
        /// Application CPU lockup
        const LOCKUP = 1 << 7;
        /// Wakeup from System OFF when wakeup is triggered by DETECT signal from GPIO
        const OFF = 1 << 8;
        /// Wakeup from System OFF when wakeup is triggered by ANADETECT signal from LPCOMP
        const LPCOMP = 1 << 9;
        /// Wakeup from System OFF when wakeup is triggered by entering the Debug Interface mode
        const DIF = 1 << 10;
        /// Wakeup from GRTC
        const GRTC = 1 << 11;
        /// Wakeup from System OFF mode due to NFC field being detected
        const NFC = 1 << 12;
        /// Illegal tampering of the device
        const SECTAMPER = 1 << 13;
        /// Wakeup from System OFF mode due to VBUS rising into valid range
        #[cfg(feature = "_nrf54lm20")]
        const VBUS = 1 << 14;
    }
}

/// Reads the reset reason bitflags.
///
/// On the nRF54L, an empty value means a power-on, brownout or glitch detector reset.
/// On the nRF54LM20, when [`ResetReason::RESETPIN`] is set, the other reasons are to
/// be ignored.
pub fn read_reasons() -> ResetReason {
    ResetReason::from_bits_retain(RESET.resetreas().read().0)
}

/// Resets the reset reasons
pub fn clear_reasons() {
    RESET.resetreas().write(|w| *w = Resetreas(ResetReason::all().bits()));
}

/// Returns if the network core is held in reset
#[cfg(feature = "_nrf5340-app")]
pub fn network_core_held() -> bool {
    RESET.network().forceoff().read().forceoff() == Forceoff::Hold
}

/// Releases the network core from the FORCEOFF state
#[cfg(feature = "_nrf5340-app")]
pub fn release_network_core() {
    RESET.network().forceoff().write(|w| w.set_forceoff(Forceoff::Release));
}

/// Holds the network core in the FORCEOFF state
#[cfg(feature = "_nrf5340-app")]
pub fn hold_network_core() {
    RESET.network().forceoff().write(|w| w.set_forceoff(Forceoff::Hold));
}
