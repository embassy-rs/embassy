#![macro_use]

//! ADC driver

use crate::pac::adc0::Adc0;
use crate::pac::adc0::vals;

/// Trait that provides channel numbers for pins that support ADC
pub trait AdcPin {
    /// Channel number
    fn channel(&self) -> u8;
    /// Channel side (A / B), 0 = A, 1 = B
    fn ctype(&self) -> u8;
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
        defmt::info!("Starting ADC init sequence");

        // Reset just in case
        adc.ctrl().write(|w| w.set_rst(vals::Rst::RST_1));
        adc.ctrl().write(|w| w.set_rst(vals::Rst::RST_0)); 

        // Reset results
        adc.ctrl().modify(|w| {
            w.set_rstfifo0(vals::Rstfifo0::RSTFIFO0_1);
            w.set_rstfifo1(vals::Rstfifo1::RSTFIFO1_1)
        });

        // Enable power and select reference
        adc.cfg().write(|w| {
            w.set_pwren(1.into());
            w.set_refsel(1.into()); // VREFH = VDDA
            w.set_tprictrl(0.into())
        });

        // No pause
        adc.pause().write(|w| {
            w.set_pauseen(0.into())
        });

        // Disable watermark
        adc.fctrl(0).write(|w| {
            w.set_fwmark(0)
        });
        adc.fctrl(0).write(|w| {
            w.set_fwmark(0)
        });

        // Enable the ADC
        adc.ctrl().modify(|w| {
            w.set_adcen(vals::Adcen::ADCEN_1)
        });

        // Calibrate
        Self::calibrate(adc);

        let avgs_bit = match config.averaging {
            Averaging::None => 0,
            Averaging::Samples2 => 1,
            Averaging::Samples4 => 2,
            Averaging::Samples8 => 3,
            Averaging::Samples16 => 4,
            Averaging::Samples32 => 5,
            Averaging::Samples64 => 6,
            Averaging::Samples128 => 7,
        };
        adc.cmdh1().write(|w| {
            w.set_avgs(avgs_bit.into());
            w.set_cmpen(0.into()); // disable compare
            w.set_loop_(0.into()); // no loop
            w.set_next(0.into()); // no next command
            w.set_sts(7.into())
        });

        // Set up trigger control
        adc.tctrl(0).modify(|w| {
            w.set_hten(1.into());
            w.set_fifo_sel_a(vals::FifoSelA::FIFO_SEL_A_0);
            w.set_fifo_sel_b(vals::FifoSelB::FIFO_SEL_B_0);
            w.set_tcmd(vals::Tcmd::TCMD_1)
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

    /// Helper function to calibrate
    /// Reference: https://github.com/lpc55/lpc55-hal/blob/main/examples/adc.rs#L14
    fn calibrate<'a>(adc: &'a mut Adc0) {
        defmt::info!("Starting the calibration...");
        
        // Offset trimming
        adc.ctrl().modify(|w| w.set_calofs(vals::Calofs::CALOFS_1));
        while adc.stat().read().cal_rdy().to_bits() == 0 {}
        defmt::info!("Offset calibration completed");

        // Auto calibration request
        adc.ctrl().modify(|w| {
            // Auto calibration request
            w.set_cal_req(vals::CalReq::CAL_REQ_1)
        });

        while (adc.gcc(0).read().rdy().to_bits() == 0) || (adc.gcc(1).read().rdy().to_bits() == 0) {
            // Wait for auto calibration
            defmt::info!("GCC0: {}", adc.gcc(0).read().rdy().to_bits());
            defmt::info!("GCC1: {}", adc.gcc(1).read().rdy().to_bits());
        }

        let gain_a = adc.gcc(0).read().gain_cal();
        let gain_b = adc.gcc(1).read().gain_cal();
        defmt::info!("gain_a raw = {:#x}, gain_b raw = {:#x}", gain_a, gain_b);

        let gcr_a = (((gain_a as u32) << 16) / (0x1FFFFu32 - gain_a as u32)) as u16;
        let gcr_b = (((gain_b as u32) << 16) / (0x1FFFFu32 - gain_b as u32)) as u16;

        adc.gcr(0).write(|w| { 
            w.set_gcalr(gcr_a);
            w.set_rdy(1.into())
        });
        adc.gcr(1).write(|w| { 
            w.set_gcalr(gcr_b);
            w.set_rdy(1.into())
        });

        while !(adc.stat().read().cal_rdy().to_bits() != 0) {}

        defmt::info!("Calibration completed");
    }

    /// Reading the channel synchronously
    pub fn blocking_read<P: AdcPin>(&mut self, pin: &mut P) -> u16 {
        self.adc.cmdl1().modify(|w| {
            w.set_adch(pin.channel().into());
            w.set_ctype(pin.ctype().into())
        });

        let cmdl_readback = self.adc.cmdl1().read();
        defmt::info!(
            "CMDL1 readback: adch={} ctype={} mode={}",
            cmdl_readback.adch().to_bits(),
            cmdl_readback.ctype().to_bits(),
            cmdl_readback.mode().to_bits()
        );

        // Trigger the event
        self.adc.swtrig().write(|w| {
            w.set_swt0(1.into())
        });

        let mut result_reg = self.adc.resfifo(0).read();
        defmt::info!("Waiting for FIFO0 to become valid");
        while !(result_reg.valid() == vals::Valid::VALID_1) {
            result_reg = self.adc.resfifo(0).read();
        }

        defmt::info!(
            "RESFIFO0: d=0x{:04x} tsrc={} loopcnt={} cmdsrc={} valid={}",
            result_reg.d(),
            result_reg.tsrc().to_bits(),
            result_reg.loopcnt().to_bits(),
            result_reg.cmdsrc().to_bits(),
            result_reg.valid().to_bits()
        );
        
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
    // pin     => peripheral struct
    // channel => u8 channel number
    // ctype   => channel side (0=A, 1=B)
    ($pin:ident, $channel:expr, $ctype:expr) => {
        impl<'d> crate::adc::AdcPin for crate::Peri<'d, crate::peripherals::$pin> {
            fn channel(&self) -> u8 {
                $channel
            }

            fn ctype(&self) -> u8 {
                $ctype
            }
        }
    }
}

impl_adc_pin!(PIO0_23, 0, 0);
impl_adc_pin!(PIO0_16, 0, 1);
impl_adc_pin!(PIO0_10, 1, 0);
impl_adc_pin!(PIO0_11, 1, 1);
impl_adc_pin!(PIO0_15, 2, 0);
impl_adc_pin!(PIO0_12, 2, 1);
impl_adc_pin!(PIO0_31, 3, 0);
impl_adc_pin!(PIO1_0,  3, 1);
impl_adc_pin!(PIO1_8,  4, 0);
impl_adc_pin!(PIO1_9,  4, 1);
