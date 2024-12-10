//! Reset

#![macro_use]

use bitflags::bitflags;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use nrf_pac::reset::regs::Resetreas;
#[cfg(not(feature = "_nrf5340-net"))]
use nrf_pac::reset::vals::Forceoff;

use crate::pac;

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

/// An instance of the RESET.
pub struct Reset<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Reset<'d, T> {
    /// Create a new RESET instance
    pub fn new(_p: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(_p);
        Self { _p }
    }

    /// Reads the bitflag of the reset reasons
    pub fn read_reasons(&self) -> ResetReason {
        let regs = T::regs();

        ResetReason::from_bits_retain(regs.resetreas().read().0)
    }

    /// Resets the reset reasons
    pub fn clear_reasons(&self) {
        let regs = T::regs();

        regs.resetreas().write(|w| *w = Resetreas(ResetReason::all().bits()));
    }

    /// Returns if the network core is held in reset
    #[cfg(not(feature = "_nrf5340-net"))]
    pub fn network_core_held(&self) -> bool {
        let regs = T::regs();

        regs.network().forceoff().read().forceoff() == Forceoff::HOLD
    }

    /// Releases the network core from the FORCEOFF state
    #[cfg(not(feature = "_nrf5340-net"))]
    pub fn release_network_core(&self) {
        let regs = T::regs();

        regs.network().forceoff().write(|w| w.set_forceoff(Forceoff::RELEASE));
    }

    /// Holds the network core in the FORCEOFF state
    #[cfg(not(feature = "_nrf5340-net"))]
    pub fn hold_network_core(&self) {
        let regs = T::regs();

        regs.network().forceoff().write(|w| w.set_forceoff(Forceoff::HOLD));
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> pac::reset::Reset;
}

/// Basic Reset Instance
#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + SealedInstance + 'static + Send {}

macro_rules! impl_reset {
    ($type:ident, $pac_type:ident) => {
        impl crate::reset::SealedInstance for peripherals::$type {
            fn regs() -> pac::reset::Reset {
                pac::$pac_type
            }
        }
        impl crate::reset::Instance for peripherals::$type {}
    };
}
