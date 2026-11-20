//! Low-level timer driver.

use crate::lptim::vals;

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
            Prescaler::Div1 => vals::Presc::Div1,
            Prescaler::Div2 => vals::Presc::Div2,
            Prescaler::Div4 => vals::Presc::Div4,
            Prescaler::Div8 => vals::Presc::Div8,
            Prescaler::Div16 => vals::Presc::Div16,
            Prescaler::Div32 => vals::Presc::Div32,
            Prescaler::Div64 => vals::Presc::Div64,
            Prescaler::Div128 => vals::Presc::Div128,
        }
    }
}

impl From<vals::Presc> for Prescaler {
    fn from(prescaler: vals::Presc) -> Self {
        match prescaler {
            vals::Presc::Div1 => Prescaler::Div1,
            vals::Presc::Div2 => Prescaler::Div2,
            vals::Presc::Div4 => Prescaler::Div4,
            vals::Presc::Div8 => Prescaler::Div8,
            vals::Presc::Div16 => Prescaler::Div16,
            vals::Presc::Div32 => Prescaler::Div32,
            vals::Presc::Div64 => Prescaler::Div64,
            vals::Presc::Div128 => Prescaler::Div128,
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
    /// Smallest prescaler in /1, /2, ..., /128 for which `ticks / prescaler` fits in the 16-bit reload.
    ///
    /// Panics if even /128 leaves the reload above [`u16::MAX`].
    pub fn from_ticks(ticks: u32) -> Self {
        // `(ticks >> 16).next_power_of_two()` is one step too small when that
        // shift is 1, 2, or already a power of two: /2 of 160000 ticks is 80000.
        let mut div = 1u32;
        while div <= 128 && ticks / div > u32::from(u16::MAX) {
            div *= 2;
        }
        assert!(
            div <= 128,
            "LPTIM period of {} clock ticks does not fit in the 16-bit reload",
            ticks
        );
        div.into()
    }

    pub fn scale_down(&self, ticks: u32) -> u16 {
        (ticks / u32::from(self)).try_into().unwrap()
    }

    pub fn scale_up(&self, ticks: u16) -> u32 {
        u32::from(self) * ticks as u32
    }
}
