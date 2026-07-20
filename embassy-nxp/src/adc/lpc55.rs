#![macro_use]

//! ADC driver

use crate::pac::adc0::Adc0;
use crate::pac::adc0::vals;

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
            w.set_adcen(vals::Adcen::ADCEN_1)
        });
        adc.tctrl(0).modify(|w| {
            w.set_fifo_sel_a(vals::FifoSelA::FIFO_SEL_A_0)
        });

        // Set resolution
        match config.resolution {
            Resolution::Bits12 => {
                adc.cmdl1().modify(|w| {
                    w.set_mode(vals::Cmdl1Mode::MODE_0)
                });
            },
            Resolution::Bits16 => {
                adc.cmdl1().modify(|w| {
                    w.set_mode(vals::Cmdl1Mode::MODE_1)
                })
            }
        };

        // Set averaging
        let bit_value = match config.averaging {
            Averaging::None => vals::CalAvgs::CAL_AVGS_0,
            Averaging::Samples2 => vals::CalAvgs::CAL_AVGS_1,
            Averaging::Samples4 => vals::CalAvgs::CAL_AVGS_2,
            Averaging::Samples8 => vals::CalAvgs::CAL_AVGS_3,
            Averaging::Samples16 => vals::CalAvgs::CAL_AVGS_4,
            Averaging::Samples32 => vals::CalAvgs::CAL_AVGS_5,
            Averaging::Samples64 => vals::CalAvgs::CAL_AVGS_6,
            Averaging::Samples128 => vals::CalAvgs::CAL_AVGS_7,
        };

        adc.ctrl().modify(|w| {
            w.set_cal_avgs(bit_value)
        });

        Self { adc, config }
    }

    pub fn blocking_read<P: AdcPin>(&mut self, pin: &mut P) -> u16 {
        self.adc.cmdl1().modify(|w| {
            w.set_adch(pin.channel().into())
        });

        let result_reg = self.adc.resfifo(0).read();
        
        if result_reg.valid() == vals::Valid::VALID_0 {
            // TODO handle error
        }

        let data_raw = result_reg.d();

        let data = match self.config.resolution {
            Resolution::Bits16 => data_raw,
            Resolution::Bits12 => data_raw >> 3,
        };

        data
    }
}
