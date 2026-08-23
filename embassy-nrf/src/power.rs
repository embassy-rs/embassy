//! Power

#[cfg(feature = "nrf52840")]
use crate::chip::pac::POWER;
#[cfg(any(feature = "nrf9160-s", feature = "nrf9160-ns", feature = "_nrf54l"))]
use crate::chip::pac::REGULATORS;

/// Puts the MCU into "System Off" mode with minimal power usage
pub fn set_system_off() {
    #[cfg(feature = "nrf52840")]
    POWER.systemoff().write(|w| w.set_systemoff(true));
    #[cfg(any(feature = "nrf9160-s", feature = "nrf9160-ns", feature = "_nrf54l"))]
    REGULATORS.systemoff().write(|w| w.set_systemoff(true));
}
