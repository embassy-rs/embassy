//! Low-level timer driver.

use crate::pac::lptim::vals;

pub enum Prescaler {
    Div1,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
}

impl From<&Prescaler> for vals::Presc {
    fn from(prescaler: &Prescaler) -> Self {
        match prescaler {
            Prescaler::Div1 => vals::Presc::DIV1,
            Prescaler::Div2 => vals::Presc::DIV2,
            Prescaler::Div4 => vals::Presc::DIV4,
            Prescaler::Div8 => vals::Presc::DIV8,
            Prescaler::Div16 => vals::Presc::DIV16,
            Prescaler::Div32 => vals::Presc::DIV32,
            Prescaler::Div64 => vals::Presc::DIV64,
            Prescaler::Div128 => vals::Presc::DIV128,
        }
    }
}

impl From<vals::Presc> for Prescaler {
    fn from(prescaler: vals::Presc) -> Self {
        match prescaler {
            vals::Presc::DIV1 => Prescaler::Div1,
            vals::Presc::DIV2 => Prescaler::Div2,
            vals::Presc::DIV4 => Prescaler::Div4,
            vals::Presc::DIV8 => Prescaler::Div8,
            vals::Presc::DIV16 => Prescaler::Div16,
            vals::Presc::DIV32 => Prescaler::Div32,
            vals::Presc::DIV64 => Prescaler::Div64,
            vals::Presc::DIV128 => Prescaler::Div128,
        }
    }
}

impl From<&Prescaler> for u32 {
    fn from(prescaler: &Prescaler) -> Self {
        match prescaler {
            Prescaler::Div1 => 1,
            Prescaler::Div2 => 2,
            Prescaler::Div4 => 4,
            Prescaler::Div8 => 8,
            Prescaler::Div16 => 16,
            Prescaler::Div32 => 32,
            Prescaler::Div64 => 64,
            Prescaler::Div128 => 128,
        }
    }
}

impl From<u32> for Prescaler {
    fn from(prescaler: u32) -> Self {
        match prescaler {
            1 => Prescaler::Div1,
            2 => Prescaler::Div2,
            4 => Prescaler::Div4,
            8 => Prescaler::Div8,
            16 => Prescaler::Div16,
            32 => Prescaler::Div32,
            64 => Prescaler::Div64,
            128 => Prescaler::Div128,
            _ => unreachable!(),
        }
    }
}

impl Prescaler {
    pub fn from_ticks(ticks: u32) -> Self {
        // We need to scale down to a 16-bit range
        (ticks >> 16).next_power_of_two().into()
    }

    pub fn scale_down(&self, ticks: u32) -> u16 {
        (ticks / u32::from(self)).try_into().unwrap()
    }

    pub fn scale_up(&self, ticks: u16) -> u32 {
        u32::from(self) * ticks as u32
    }
}
