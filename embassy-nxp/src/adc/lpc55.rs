#![macro_use]

//! ADC driver

use crate::pac::adc0::Adc0;
use crate::pac::adc0::

pub trait AdcPin {
    // TODO
    fn channel(&self) -> u8;
}

pub enum Resolution {
    Bits16,
    Bits12,
}

pub enum Averaging {
    None,
    Samples2,
    Samples4,
    Samples8,
    Samples16,
    Samples32,
    Samples64,
    Samples128,
}

pub struct Config {
    pub resolution: Resolution,
    pub averaging: Averaging,
}

impl Config {
    pub const fn default() -> Self {
        Self {
            resolution: Resolution::Bits16,
            averaging: Averaging::None,
        }
    }
}

pub struct Adc<'d> {
    adc: &'d mut Adc0,
    config: Config,
}

impl<'d> Adc<'d> {
    pub fn new(adc: &'d mut Adc0, config: Config) -> Self {
        // Enable and configure the ADC
        adc.ctrl().modify(|w| {
            w.adcen().set_bit()
        });
        adc.tctrl(0).modify(|w| {
            w.fifo_sel_a().clear_bit()
        });

        // Set resolution
        match config.resolution {
            Resolution::Bits12 => {
                adc.cmdl1().modify(|w| {
                    w.mode().clear_bit()
                });
            },
            Resolution::Bits16 => {
                adc.cmdl1().modify(|w| {
                    w.mode().set_bit()
                })
            }
        };

        // Set averaging
        let bit_value: u8 = match config.averaging {
            Averaging::None => 0,
            Averaging::Samples2 => 1,
            Averaging::Samples4 => 2,
            Averaging::Samples8 => 3,
            Averaging::Samples16 => 4,
            Averaging::Samples32 => 5,
            Averaging::Samples64 => 6,
            Averaging::Samples128 => 7,
        };

        adc.ctrl().modify(|w| {
            w.cal_avgs().bits(bit_value)
        });

        Self { adc, config }
    }

    pub fn blocking_read<P: AdcPin>(&mut self, pin: &mut P) -> u16 {
        let result_reg = self.adc.resfifo(0).read();
        
        if result_reg.valid().bit_is_clear() {
            // TODO handle error
        }

        let data_raw = result_reg.d().bits();

        let data = match self.config.resolution {
            Resolution::Bits16 => data_raw,
            Resolution::Bits12 => data_raw >> 3,
        };

        data
    }
}
