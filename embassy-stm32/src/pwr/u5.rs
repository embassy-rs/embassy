use crate::peripherals;

/// Voltage Scale
///
/// Represents the voltage range feeding the CPU core. The maximum core
/// clock frequency depends on this value.
#[derive(Copy, Clone, PartialEq)]
pub enum VoltageScale {
    // Highest frequency
    Range1,
    Range2,
    Range3,
    // Lowest power
    Range4,
}

/// Power Configuration
///
/// Generated when the PWR peripheral is frozen. The existence of this
/// value indicates that the voltage scaling configuration can no
/// longer be changed.
pub struct Power {
    pub(crate) vos: VoltageScale,
}

impl Power {
    pub fn new(_peri: peripherals::PWR) -> Self {
        Self {
            vos: VoltageScale::Range4,
        }
    }
}
