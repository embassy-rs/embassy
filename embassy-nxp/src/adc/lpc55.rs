//! ADC driver

#![macro_use]

use embassy_hal_internal::Peri;

use crate::pac;
use crate::pac::adc0::{Adc0, vals};
use crate::peripherals::ADC0;

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

impl Config {
    pub const fn new(resolution: Resolution, averaging: Averaging) -> Self {
        Self { resolution, averaging }
    }
}

/// The default config
impl Default for Config {
    fn default() -> Self {
        Self {
            resolution: Resolution::Bits16,
            averaging: Averaging::None,
        }
    }
}

/// The main struct
pub struct Adc<'d> {
    _peri: Peri<'d, ADC0>,
    config: Config,
}

impl<'d> Adc<'d> {
    /// Creation and initialization of ADC
    pub fn new(peri: Peri<'d, ADC0>, config: Config) -> Self {
        let adc: Adc0 = pac::ADC0;

        // Power & clocks
        Self::enable_power_clocks();

        // Reset just in case
        adc.ctrl().write(|w| w.set_rst(vals::Rst::Rst1));
        adc.ctrl().write(|w| w.set_rst(vals::Rst::Rst0));

        // Reset results
        adc.ctrl().modify(|w| {
            w.set_rstfifo0(vals::Rstfifo0::Rstfifo01);
            w.set_rstfifo1(vals::Rstfifo1::Rstfifo11)
        });

        // Enable power and select reference
        adc.cfg().write(|w| {
            w.set_pwren(1.into());
            w.set_pudly(0x80);
            w.set_refsel(1.into()); // VREFH = VDDA
            w.set_tprictrl(0.into())
        });

        // No pause
        adc.pause().write(|w| w.set_pauseen(0.into()));

        // Disable watermark
        adc.fctrl(0).write(|w| w.set_fwmark(0));
        adc.fctrl(0).write(|w| w.set_fwmark(0));

        // Enable the ADC
        adc.ctrl().modify(|w| w.set_adcen(1.into()));

        // Calibrate
        Self::calibrate(&adc);

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
            w.set_hten(0.into());
            w.set_fifo_sel_a(0.into());
            w.set_fifo_sel_b(0.into());
            w.set_tcmd(vals::Tcmd::Tcmd1)
        });

        // Set resolution
        match config.resolution {
            Resolution::Bits12 => {
                adc.cmdl1().modify(|w| w.set_mode(vals::Cmdl1Mode::Mode0));
            }
            Resolution::Bits16 => adc.cmdl1().modify(|w| w.set_mode(vals::Cmdl1Mode::Mode1)),
        };

        // Set averaging
        adc.ctrl().modify(|w| w.set_cal_avgs(avgs_bit.into()));

        Self { _peri: peri, config }
    }

    fn enable_power_clocks() {
        let syscon = pac::SYSCON;
        let pmc = pac::PMC;
        let anactrl = pac::ANACTRL;

        // Enable clocks
        // bit 27 = ADC
        // bit 13 = IOCON
        syscon.ahbclkctrlset(0).write(|w| w.set_data((1 << 27) | (1 << 13)));
        syscon.ahbclkctrlset(2).write(|w| w.set_data(1 << 27));

        // Clear reset
        syscon.presetctrlclr(0).write(|w| w.set_data(1 << 27));

        // Power up AUXBIAS
        pmc.pdruncfgclr0().write(|w| w.set_pdruncfgclr0(1 << 19));

        // Enable VREF
        anactrl.aux_bias().modify(|w| w.set_vref1venable(true));

        // Configure the clocks
        syscon.adcclksel().write(|w| w.set_sel(0));
        syscon.adcclkdiv().write(|w| w.set_div(6));
        syscon.adcclkdiv().modify(|w| w.set_reset(1.into()));
        syscon.adcclkdiv().modify(|w| w.set_reset(0.into()));
        syscon.adcclkdiv().modify(|w| w.set_halt(0.into()));
    }

    /// Helper function for calibrating
    /// Reference: https://github.com/lpc55/lpc55-hal/blob/main/examples/adc.rs#L14
    fn calibrate<'a>(adc: &Adc0) {
        // Offset trimming
        adc.ctrl().modify(|w| w.set_calofs(vals::Calofs::Calofs1));
        while adc.stat().read().cal_rdy().to_bits() == 0 {}

        // Auto calibration request
        adc.ctrl().modify(|w| {
            // Auto calibration request
            w.set_cal_req(vals::CalReq::CalReq1)
        });

        while (adc.gcc(0).read().rdy().to_bits() == 0) || (adc.gcc(1).read().rdy().to_bits() == 0) {
            // Wait for auto calibration
        }

        let gain_a = adc.gcc(0).read().gain_cal();
        let gain_b = adc.gcc(1).read().gain_cal();

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
    }

    /// Reading the channel synchronously
    pub fn blocking_read<P: AdcPin>(&mut self, pin: &mut P) -> u16 {
        let adc: Adc0 = pac::ADC0;
        pin.configure_iocon();
        adc.cmdl1().modify(|w| {
            w.set_adch(pin.channel().into());
            w.set_ctype(pin.ctype().into())
        });

        // Trigger the event
        adc.swtrig().write(|w| w.set_swt0(1.into()));

        // wait for results
        while adc.fctrl(0).read().fcount() == 0 {}

        let result_reg = adc.resfifo(0).read();
        let data_raw = result_reg.d();

        let data = match self.config.resolution {
            Resolution::Bits16 => data_raw,
            Resolution::Bits12 => data_raw >> 3,
        };

        data
    }
}

/// Trait that provides channel numbers for pins that support ADC
pub trait AdcPin {
    /// Channel number
    fn channel(&self) -> u8;
    /// Channel side (A / B), 0 = A, 1 = B
    fn ctype(&self) -> u8;
    /// Set up the iocon register for that pin
    fn configure_iocon(&mut self);
}

#[repr(u8)]
enum ChannelSide {
    SideA = 0,
    SideB = 1,
}

/// Macro to implement the AdcPin trait for pins
macro_rules! impl_adc_pin {
    // pin          => pin peripheral struct (e.g.`PIO1_8`)
    // pin_port     => pin port number (e.g. `1` for `PIO1_8`)
    // pin_num      => pin number (e.g. `8` for `PIO1_8`)
    // adc_channel  => ADC channel number (as u8)
    // channel_side => channel side (0=A, 1=B)
    ($pin:ident, $pin_port:expr, $pin_num:expr, $adc_channel:expr, $channel_side:expr) => {
        impl<'d> crate::adc::AdcPin for crate::Peri<'d, crate::peripherals::$pin> {
            fn channel(&self) -> u8 {
                $adc_channel
            }

            fn ctype(&self) -> u8 {
                $channel_side as u8
            }

            fn configure_iocon(&mut self) {
                let iocon = pac::IOCON;
                let port = $pin_port;
                match port {
                    0 => {
                        iocon.pio0($pin_num).modify(|w| {
                            w.set_func(0.into());
                            w.set_mode(0.into());
                            w.set_digimode(0.into());
                            w.set_asw(1.into())
                        });
                    }
                    1 => {
                        iocon.pio1($pin_num).modify(|w| {
                            w.set_func(0.into());
                            w.set_mode(0.into());
                            w.set_digimode(0.into());
                            w.set_asw(1.into())
                        });
                    }
                    _ => unreachable!(),
                }
            }
        }
    };
}

// https://www.nxp.com/docs/en/data-sheet/LPC55S6x.pdf
impl_adc_pin!(PIO0_23, 0, 23, 0, ChannelSide::SideA);
impl_adc_pin!(PIO0_16, 0, 16, 0, ChannelSide::SideB);
impl_adc_pin!(PIO0_10, 0, 10, 1, ChannelSide::SideA);
impl_adc_pin!(PIO0_11, 0, 11, 1, ChannelSide::SideB);
impl_adc_pin!(PIO0_15, 0, 15, 2, ChannelSide::SideA);
impl_adc_pin!(PIO0_12, 0, 12, 2, ChannelSide::SideB);
impl_adc_pin!(PIO0_31, 0, 31, 3, ChannelSide::SideA);
impl_adc_pin!(PIO1_0, 1, 0, 3, ChannelSide::SideB);
impl_adc_pin!(PIO1_8, 1, 8, 4, ChannelSide::SideA);
impl_adc_pin!(PIO1_9, 1, 9, 4, ChannelSide::SideB);
