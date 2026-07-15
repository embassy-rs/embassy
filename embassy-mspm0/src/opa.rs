//! Operational Amplifier (OPA)
//!
//! The MSPM0 OPA is a zero-drift, chopper-stabilized operational amplifier
//! with an internal programmable gain ladder. This driver currently exposes
//! the buffer and non-inverting PGA topologies (TRM "OPA Amplifier Modes"):
//!
//! - **Buffer** (unity gain follower): [`Opa::buffer_ext`] / [`Opa::buffer_int`]
//! - **Non-inverting PGA** (x2..x32): [`Opa::pga_ext`] / [`Opa::pga_int`]
//!
//! `_ext` variants drive the OPAx_OUT pin; `_int` variants keep the output
//! off the pins and only route it to the ADC. Both can be sampled by the ADC
//! through the internal channel returned by
//! [`OpaOutput::adc_channel`]/[`OpaInternalOutput::adc_channel`].
//!
//! The non-inverting input can come from a pin or from an internal source
//! (DAC12, the COMP reference DAC8, VREF, ground, or the previous OPA for
//! cascading) — see [`NonInvertingInput`].
//!
//! Chopping ([`Chopping`]) removes the amplifier's input offset, but the
//! standard mode modulates ripple onto the output; it is intended to be used
//! with the ADC's hardware averaging. When sampling the output without
//! averaging, leave chopping disabled and expect a few millivolts of offset.

#![macro_use]

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;

use crate::Peri;
use crate::pac::opa::{regs, vals};

/// Gain-bandwidth selection (CFGBASE.GBW).
///
/// The high setting increases both bandwidth and current consumption; see the
/// device datasheet for values.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GainBandwidth {
    /// Low gain bandwidth, lower current.
    Low,
    /// High gain bandwidth, higher current.
    High,
}

/// Chopping mode (CFG.CHOP).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Chopping {
    /// No chopping. The raw input offset voltage (up to a few millivolts,
    /// multiplied by the gain) appears at the output.
    Disabled,
    /// Standard chopping. Removes the input offset but modulates ripple at
    /// the chop frequency onto the output.
    Standard,
    /// Chop with post-averaging. Requires the output to be sampled by the
    /// ADC in hardware averaging mode.
    AdcAveraging,
}

/// Configuration common to all OPA topologies.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// Gain-bandwidth selection. Defaults to [`GainBandwidth::High`].
    pub gain_bandwidth: GainBandwidth,
    /// Rail-to-rail input. Defaults to `true`.
    pub rail_to_rail_input: bool,
    /// Chopping mode. Defaults to [`Chopping::Disabled`].
    pub chopping: Chopping,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gain_bandwidth: GainBandwidth::High,
            rail_to_rail_input: true,
            chopping: Chopping::Disabled,
        }
    }
}

/// Gain for the buffer and non-inverting PGA topologies.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Gain {
    X1 = 0,
    X2 = 1,
    X4 = 2,
    X8 = 3,
    X16 = 4,
    X32 = 5,
}

/// A source for the non-inverting (+) input (CFG.PSEL).
///
/// Created from an `OPAx_INy+` pin (which is configured for analog mode and
/// consumed — use [`Peri::reborrow`] to keep it), or from one of the
/// internal-source constructors.
pub struct NonInvertingInput<'d, T: Instance> {
    channel: vals::Psel,
    _phantom: PhantomData<(&'d (), T)>,
}

impl<'d, T: Instance> NonInvertingInput<'d, T> {
    const fn internal(channel: vals::Psel) -> Self {
        Self {
            channel,
            _phantom: PhantomData,
        }
    }

    /// The DAC12 output, routed internally (only on devices with a DAC).
    ///
    /// This is the same channel as the `OPAx_IN2+` pad shared with DAC_OUT:
    /// with the DAC disabled, an external voltage on that pad drives it.
    pub const fn dac12() -> Self {
        Self::internal(vals::Psel::DAC12OUT)
    }

    /// The 8-bit reference DAC of the paired COMP peripheral.
    pub const fn dac8() -> Self {
        Self::internal(vals::Psel::DAC8OUT)
    }

    /// The internal voltage reference (VREF).
    pub const fn vref() -> Self {
        Self::internal(vals::Psel::VREF)
    }

    /// Analog ground.
    pub const fn ground() -> Self {
        Self::internal(vals::Psel::VSS)
    }
}

impl<'d, T: Instance, P: NonInvertingPin<T>> From<Peri<'d, P>> for NonInvertingInput<'d, T> {
    fn from(pin: Peri<'d, P>) -> Self {
        SealedNonInvertingPin::setup(&*pin);
        Self {
            channel: vals::Psel::from_bits(SealedNonInvertingPin::channel(&*pin)),
            _phantom: PhantomData,
        }
    }
}

/// OPA driver.
///
/// Power to the peripheral is enabled on construction and removed on drop.
/// Use the topology methods to configure and enable the amplifier.
pub struct Opa<'d, T: Instance> {
    _peri: Peri<'d, T>,
    chop: vals::Chop,
}

/// An enabled OPA whose output drives the OPAx_OUT pin.
///
/// Can also be sampled by the ADC via [`OpaOutput::adc_channel`]. The
/// amplifier is disabled when this is dropped.
pub struct OpaOutput<'d, T: Instance> {
    _inner: &'d Opa<'d, T>,
}

/// An enabled OPA whose output is only routed internally (to the ADC).
///
/// Sample it via [`OpaInternalOutput::adc_channel`]. The amplifier is
/// disabled when this is dropped.
pub struct OpaInternalOutput<'d, T: Instance> {
    _inner: &'d Opa<'d, T>,
}

impl<'d, T: Instance> Opa<'d, T> {
    /// Create a new OPA driver.
    ///
    /// Resets and powers up the peripheral and applies `config`. The
    /// amplifier itself stays disabled until a topology method is called.
    pub fn new(peri: Peri<'d, T>, config: Config) -> Self {
        let r = T::regs();

        r.gprcm().rstctl().write(|w| {
            w.set_key(vals::ResetKey::KEY);
            w.set_resetassert(true);
            w.set_resetstkyclr(true);
        });
        r.gprcm().pwren().write(|w| {
            w.set_key(vals::PwrenKey::KEY);
            w.set_enable(true);
        });
        // A few bus cycles are required after the power switch before
        // touching peripheral registers.
        cortex_m::asm::delay(16);

        r.cfgbase().write(|w| {
            w.set_gbw(match config.gain_bandwidth {
                GainBandwidth::Low => vals::Gbw::LOWGAIN,
                GainBandwidth::High => vals::Gbw::HIGHGAIN,
            });
            w.set_rri(config.rail_to_rail_input);
        });

        let chop = match config.chopping {
            Chopping::Disabled => vals::Chop::OFF,
            Chopping::Standard => vals::Chop::ON,
            Chopping::AdcAveraging => vals::Chop::AVGON,
        };

        Self { _peri: peri, chop }
    }

    fn enable(&self, mut cfg: regs::Cfg) {
        let r = T::regs();
        cfg.set_chop(self.chop);
        r.cfg().write_value(cfg);
        r.ctl().write(|w| w.set_enable(true));
        while !r.stat().read().rdy() {}
    }

    /// Unity-gain buffer of `input`, driving the output pin.
    pub fn buffer_ext<'a>(
        &'a mut self,
        input: impl Into<NonInvertingInput<'a, T>>,
        output: Peri<'a, impl OutputPin<T>>,
    ) -> OpaOutput<'a, T> {
        SealedOutputPin::setup(&*output);
        let mut cfg = Self::buffer_cfg(input.into());
        cfg.set_outpin(true);
        self.enable(cfg);
        OpaOutput { _inner: self }
    }

    /// Unity-gain buffer of `input`, output routed only to the ADC.
    pub fn buffer_int<'a>(&'a mut self, input: impl Into<NonInvertingInput<'a, T>>) -> OpaInternalOutput<'a, T> {
        self.enable(Self::buffer_cfg(input.into()));
        OpaInternalOutput { _inner: self }
    }

    fn buffer_cfg(input: NonInvertingInput<'_, T>) -> regs::Cfg {
        // Feedback from the ladder top; the ladder bottom is left open so no
        // current flows and the ladder acts as a plain wire.
        let mut cfg = regs::Cfg(0);
        cfg.set_psel(input.channel);
        cfg.set_nsel(vals::Nsel::OANRTOP);
        cfg
    }

    /// Non-inverting PGA: output is `gain * input`, driving the output pin.
    pub fn pga_ext<'a>(
        &'a mut self,
        input: impl Into<NonInvertingInput<'a, T>>,
        output: Peri<'a, impl OutputPin<T>>,
        gain: Gain,
    ) -> OpaOutput<'a, T> {
        SealedOutputPin::setup(&*output);
        let mut cfg = Self::pga_cfg(input.into(), gain);
        cfg.set_outpin(true);
        self.enable(cfg);
        OpaOutput { _inner: self }
    }

    /// Non-inverting PGA: output is `gain * input`, routed only to the ADC.
    pub fn pga_int<'a>(
        &'a mut self,
        input: impl Into<NonInvertingInput<'a, T>>,
        gain: Gain,
    ) -> OpaInternalOutput<'a, T> {
        self.enable(Self::pga_cfg(input.into(), gain));
        OpaInternalOutput { _inner: self }
    }

    fn pga_cfg(input: NonInvertingInput<'_, T>, gain: Gain) -> regs::Cfg {
        // Ladder bottom to ground, feedback from the tap.
        let mut cfg = regs::Cfg(0);
        cfg.set_psel(input.channel);
        cfg.set_nsel(vals::Nsel::OANRTAP);
        cfg.set_msel(vals::Msel::VSS);
        cfg.set_gain(gain as u8);
        cfg
    }
}

impl<'d, T: Instance> Drop for Opa<'d, T> {
    fn drop(&mut self) {
        let r = T::regs();
        r.ctl().write(|w| w.set_enable(false));
        r.gprcm().pwren().write(|w| {
            w.set_key(vals::PwrenKey::KEY);
            w.set_enable(false);
        });
    }
}

impl<'d, T: Instance> OpaOutput<'d, T> {
    /// Change the PGA gain while the amplifier is running.
    ///
    /// Only meaningful for outputs created by [`Opa::pga_ext`]; useful for
    /// auto-ranging. The output settles within a couple of milliseconds.
    pub fn set_gain(&mut self, gain: Gain) {
        T::regs().cfg().modify(|w| w.set_gain(gain as u8));
    }
}

impl<'d, T: Instance> OpaInternalOutput<'d, T> {
    /// Change the PGA gain while the amplifier is running.
    ///
    /// Only meaningful for outputs created by [`Opa::pga_int`]; useful for
    /// auto-ranging. The output settles within a couple of milliseconds.
    pub fn set_gain(&mut self, gain: Gain) {
        T::regs().cfg().modify(|w| w.set_gain(gain as u8));
    }
}

impl<'d, T: Instance> Drop for OpaOutput<'d, T> {
    fn drop(&mut self) {
        T::regs().ctl().write(|w| w.set_enable(false));
    }
}

impl<'d, T: Instance> Drop for OpaInternalOutput<'d, T> {
    fn drop(&mut self) {
        T::regs().ctl().write(|w| w.set_enable(false));
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> crate::pac::opa::Opa;
}

/// OPA instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {}

pub(crate) trait SealedNonInvertingPin<T> {
    fn setup(&self);
    fn channel(&self) -> u8;
}

pub(crate) trait SealedOutputPin<T> {
    fn setup(&self);
}

/// A pin that can be used as the OPA non-inverting (+) input.
#[allow(private_bounds)]
pub trait NonInvertingPin<T: Instance>: PeripheralType + SealedNonInvertingPin<T> + Sized {}

/// The OPAx_OUT pin.
#[allow(private_bounds)]
pub trait OutputPin<T: Instance>: PeripheralType + SealedOutputPin<T> + Sized {}

macro_rules! impl_opa_instance {
    ($inst:ident) => {
        impl crate::opa::SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::opa::Opa {
                crate::pac::$inst
            }
        }
        impl crate::opa::Instance for crate::peripherals::$inst {}
    };
}

macro_rules! impl_opa_pin {
    ($trait:ident, $sealed:ident, $inst:ident, $pin:ident, $ch:expr) => {
        impl crate::opa::$trait<crate::peripherals::$inst> for crate::peripherals::$pin {}
        impl crate::opa::$sealed<crate::peripherals::$inst> for crate::peripherals::$pin {
            fn setup(&self) {
                crate::gpio::SealedPin::set_as_analog(self);
            }

            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}

macro_rules! impl_opa_non_inverting_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl_opa_pin!(NonInvertingPin, SealedNonInvertingPin, $inst, $pin, $ch);
    };
}

macro_rules! impl_opa_output_pin {
    ($inst:ident, $pin:ident) => {
        impl crate::opa::OutputPin<crate::peripherals::$inst> for crate::peripherals::$pin {}
        impl crate::opa::SealedOutputPin<crate::peripherals::$inst> for crate::peripherals::$pin {
            fn setup(&self) {
                crate::gpio::SealedPin::set_as_analog(self);
            }
        }
    };
}

macro_rules! impl_opa_adc_channel {
    ($inst:ident, $adc:ident, $ch:expr) => {
        impl<'d> crate::opa::OpaOutput<'d, crate::peripherals::$inst> {
            /// The internal ADC channel for this OPA's output.
            ///
            /// The returned channel borrows the output, so the amplifier
            /// stays enabled while the channel is in use.
            pub fn adc_channel(&self) -> crate::Peri<'_, crate::adc::AnyAdcChannel<crate::peripherals::$adc>> {
                unsafe {
                    crate::Peri::new_unchecked(crate::adc::AnyAdcChannel {
                        channel: $ch,
                        _phantom: core::marker::PhantomData,
                    })
                }
            }
        }

        impl<'d> crate::opa::OpaInternalOutput<'d, crate::peripherals::$inst> {
            /// The internal ADC channel for this OPA's output.
            ///
            /// The returned channel borrows the output, so the amplifier
            /// stays enabled while the channel is in use.
            pub fn adc_channel(&self) -> crate::Peri<'_, crate::adc::AnyAdcChannel<crate::peripherals::$adc>> {
                unsafe {
                    crate::Peri::new_unchecked(crate::adc::AnyAdcChannel {
                        channel: $ch,
                        _phantom: core::marker::PhantomData,
                    })
                }
            }
        }
    };
}
