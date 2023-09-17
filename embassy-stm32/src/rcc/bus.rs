use core::ops::Div;

#[allow(unused_imports)]
use crate::pac::rcc;
pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Ppre as APBPrescaler};
use crate::time::Hertz;

/// Voltage Scale
///
/// Represents the voltage range feeding the CPU core. The maximum core
/// clock frequency depends on this value.
///
/// Scale0 represents the highest voltage range
#[derive(Copy, Clone, PartialEq)]
pub enum VoltageScale {
    Scale0,
    Scale1,
    #[cfg(not(any(rcc_wl5, rcc_wle)))]
    Scale2,
    #[cfg(not(any(rcc_wl5, rcc_wle)))]
    Scale3,
}

impl Div<AHBPrescaler> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: AHBPrescaler) -> Self::Output {
        let divisor = match rhs {
            AHBPrescaler::DIV1 => 1,
            AHBPrescaler::DIV2 => 2,
            #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
            AHBPrescaler::DIV3 => 3,
            AHBPrescaler::DIV4 => 4,
            #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
            AHBPrescaler::DIV5 => 5,
            #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
            AHBPrescaler::DIV6 => 6,
            AHBPrescaler::DIV8 => 8,
            #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
            AHBPrescaler::DIV10 => 10,
            AHBPrescaler::DIV16 => 16,
            #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
            AHBPrescaler::DIV32 => 32,
            #[cfg(not(rcc_wba))]
            AHBPrescaler::DIV64 => 64,
            #[cfg(not(rcc_wba))]
            AHBPrescaler::DIV128 => 128,
            #[cfg(not(rcc_wba))]
            AHBPrescaler::DIV256 => 256,
            #[cfg(not(rcc_wba))]
            AHBPrescaler::DIV512 => 512,
            _ => unreachable!(),
        };
        Hertz(self.0 / divisor)
    }
}

impl Div<APBPrescaler> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: APBPrescaler) -> Self::Output {
        let divisor = match rhs {
            APBPrescaler::DIV1 => 1,
            APBPrescaler::DIV2 => 2,
            APBPrescaler::DIV4 => 4,
            APBPrescaler::DIV8 => 8,
            APBPrescaler::DIV16 => 16,
            _ => unreachable!(),
        };
        Hertz(self.0 / divisor)
    }
}
