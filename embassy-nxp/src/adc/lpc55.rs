#![macro_use]

//! ADC driver

use crate::pac::adc0::Adc0;
use crate::pac::adc0::vals;

/// Trait that provides channel numbers for pins that support ADC
pub trait AdcPin {
    fn channel(&self) -> u8;
}

/// Resolution selection
pub enum Resolution {
    Bits16,
    Bits12,
}

/// Averaging selection
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

/// The struct to store the configuration
pub struct Config {
    pub resolution: Resolution,
    pub averaging: Averaging,
}

/// Default config implementation
impl Config {
    pub const fn default() -> Self {
        Self {
            resolution: Resolution::Bits16,
            averaging: Averaging::None,
        }
    }
}

/// The main struct
pub struct Adc<'d> {
    adc: &'d mut Adc0,
    config: Config,
}

impl<'d> Adc<'d> {
    /// Creation and initialization of ADC
    pub fn new(adc: &'d mut Adc0, config: Config) -> Self {
        // Enable and configure the ADC
        adc.ctrl().modify(|w| {
            w.set_adcen(vals::Adcen::ADCEN_1)
        });
        adc.tctrl(0).modify(|w| {
            w.set_fifo_sel_a(vals::FifoSelA::FIFO_SEL_A_0);
            w.set_fifo_sel_b(vals::FifoSelB::FIFO_SEL_B_0);
            w.set_tcmd(vals::Tcmd::TCMD_1);
            w.set_hten(vals::Hten::HTEN_1)
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

        defmt::info!("Bit value: {}", bit_value == vals::CalAvgs::CAL_AVGS_0);
        
        adc.ctrl().modify(|w| {
            w.set_cal_avgs(vals::CalAvgs::CAL_AVGS_4);
            w.set_cal_req(vals::CalReq::CAL_REQ_1)
        });

        defmt::info!("{}", adc.resfifo(0).read().valid() == vals::Valid::VALID_1);

        Self { adc, config }
    }

    /// Reading the channel synchronously
    pub fn blocking_read<P: AdcPin>(&mut self, pin: &mut P) -> u16 {
        self.adc.cmdl1().modify(|w| {
            w.set_adch(pin.channel().into())
        });

        self.adc.swtrig().write(|w| {
            w.set_swt0(1.into())
        });

        let mut result_reg = self.adc.resfifo(0).read();
        while !(result_reg.valid() == vals::Valid::VALID_1) {
            result_reg = self.adc.resfifo(0).read();
            // defmt::info!("dskjfhsghf");
        }

        let data_raw = result_reg.d();
        defmt::info!("Raw data: {}", data_raw);

        let data = match self.config.resolution {
            Resolution::Bits16 => data_raw,
            Resolution::Bits12 => data_raw >> 3,
        };

        data
    }
}

/// Macro to implement the AdcPin trait for pins
macro_rules! impl_adc_pin {
    // pin => peripheral struct
    // channel => u8 channel number
    ($pin:ident, $channel:expr) => {
        impl<'d> crate::adc::AdcPin for crate::Peri<'d, crate::peripherals::$pin> {
            fn channel(&self) -> u8 {
                $channel
            }
        }
    }
}

impl_adc_pin!(PIO0_10, 1);
impl_adc_pin!(PIO0_11, 9);
impl_adc_pin!(PIO0_12, 10);
impl_adc_pin!(PIO0_15, 2);
impl_adc_pin!(PIO0_16, 8);
impl_adc_pin!(PIO0_23, 0);
impl_adc_pin!(PIO0_31, 3);
impl_adc_pin!(PIO1_0, 11);
impl_adc_pin!(PIO1_8, 4);
impl_adc_pin!(PIO1_9, 12);
