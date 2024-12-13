//! Power

use crate::chip::pac::{NFCT, POWER};

/// Puts the MCU into "System Off" mode with a power usage 0f 0.4 uA
pub fn set_system_off() {
    POWER.systemoff().write(|w| w.set_systemoff(true));
}

/// Wake the system if there if an NFC field close to the nrf52840's antenna
pub fn wake_on_nfc_sense() {
    NFCT.tasks_sense().write_value(0x01);
}
