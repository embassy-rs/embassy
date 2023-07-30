use core::ops::Div;

#[allow(unused_imports)]
use crate::pac::rcc;
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

/// AHB prescaler
#[derive(Clone, Copy, PartialEq)]
pub enum AHBPrescaler {
    NotDivided,
    Div2,
    #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
    Div3,
    Div4,
    #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
    Div5,
    #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
    Div6,
    Div8,
    #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
    Div10,
    Div16,
    #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
    Div32,
    Div64,
    Div128,
    Div256,
    Div512,
}

impl Div<AHBPrescaler> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: AHBPrescaler) -> Self::Output {
        let divisor = match rhs {
            AHBPrescaler::NotDivided => 1,
            AHBPrescaler::Div2 => 2,
            #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
            AHBPrescaler::Div3 => 3,
            AHBPrescaler::Div4 => 4,
            #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
            AHBPrescaler::Div5 => 5,
            #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
            AHBPrescaler::Div6 => 6,
            AHBPrescaler::Div8 => 8,
            #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
            AHBPrescaler::Div10 => 10,
            AHBPrescaler::Div16 => 16,
            #[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
            AHBPrescaler::Div32 => 32,
            AHBPrescaler::Div64 => 64,
            AHBPrescaler::Div128 => 128,
            AHBPrescaler::Div256 => 256,
            AHBPrescaler::Div512 => 512,
        };
        Hertz(self.0 / divisor)
    }
}

#[cfg(not(any(rcc_g4, rcc_wb, rcc_wl5, rcc_wle)))]
impl From<AHBPrescaler> for rcc::vals::Hpre {
    fn from(val: AHBPrescaler) -> rcc::vals::Hpre {
        use rcc::vals::Hpre;

        match val {
            #[cfg(not(rcc_u5))]
            AHBPrescaler::NotDivided => Hpre::DIV1,
            #[cfg(rcc_u5)]
            AHBPrescaler::NotDivided => Hpre::NONE,
            AHBPrescaler::Div2 => Hpre::DIV2,
            AHBPrescaler::Div4 => Hpre::DIV4,
            AHBPrescaler::Div8 => Hpre::DIV8,
            AHBPrescaler::Div16 => Hpre::DIV16,
            AHBPrescaler::Div64 => Hpre::DIV64,
            AHBPrescaler::Div128 => Hpre::DIV128,
            AHBPrescaler::Div256 => Hpre::DIV256,
            AHBPrescaler::Div512 => Hpre::DIV512,
        }
    }
}

#[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
impl From<AHBPrescaler> for u8 {
    fn from(val: AHBPrescaler) -> u8 {
        match val {
            AHBPrescaler::NotDivided => 0x0,
            AHBPrescaler::Div2 => 0x08,
            AHBPrescaler::Div3 => 0x01,
            AHBPrescaler::Div4 => 0x09,
            AHBPrescaler::Div5 => 0x02,
            AHBPrescaler::Div6 => 0x05,
            AHBPrescaler::Div8 => 0x0a,
            AHBPrescaler::Div10 => 0x06,
            AHBPrescaler::Div16 => 0x0b,
            AHBPrescaler::Div32 => 0x07,
            AHBPrescaler::Div64 => 0x0c,
            AHBPrescaler::Div128 => 0x0d,
            AHBPrescaler::Div256 => 0x0e,
            AHBPrescaler::Div512 => 0x0f,
        }
    }
}

/// APB prescaler
#[derive(Clone, Copy)]
pub enum APBPrescaler {
    NotDivided,
    Div2,
    Div4,
    Div8,
    Div16,
}

impl Div<APBPrescaler> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: APBPrescaler) -> Self::Output {
        let divisor = match rhs {
            APBPrescaler::NotDivided => 1,
            APBPrescaler::Div2 => 2,
            APBPrescaler::Div4 => 4,
            APBPrescaler::Div8 => 8,
            APBPrescaler::Div16 => 16,
        };
        Hertz(self.0 / divisor)
    }
}

#[cfg(not(any(rcc_f1, rcc_f100, rcc_f1cl, rcc_g4, rcc_h7, rcc_h7ab, rcc_wb, rcc_wl5, rcc_wle)))]
impl From<APBPrescaler> for rcc::vals::Ppre {
    fn from(val: APBPrescaler) -> rcc::vals::Ppre {
        use rcc::vals::Ppre;

        match val {
            #[cfg(not(rcc_u5))]
            APBPrescaler::NotDivided => Ppre::DIV1,
            #[cfg(rcc_u5)]
            APBPrescaler::NotDivided => Ppre::NONE,
            APBPrescaler::Div2 => Ppre::DIV2,
            APBPrescaler::Div4 => Ppre::DIV4,
            APBPrescaler::Div8 => Ppre::DIV8,
            APBPrescaler::Div16 => Ppre::DIV16,
        }
    }
}

#[cfg(any(rcc_wb, rcc_wl5, rcc_wle))]
impl From<APBPrescaler> for u8 {
    fn from(val: APBPrescaler) -> u8 {
        match val {
            APBPrescaler::NotDivided => 1,
            APBPrescaler::Div2 => 0x04,
            APBPrescaler::Div4 => 0x05,
            APBPrescaler::Div8 => 0x06,
            APBPrescaler::Div16 => 0x07,
        }
    }
}
