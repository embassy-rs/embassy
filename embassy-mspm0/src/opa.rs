//! Operational Amplifier (OPA)
//!
//! The MSPM0 OPA is a zero-drift, chopper-stabilized operational amplifier
//! with an internal programmable gain ladder. This driver currently exposes
//! the buffer and non-inverting PGA topologies (TRM "OPA Amplifier Modes"):
//!
//! - **Buffer** (unity gain follower): [`Opa::buffer_ext`] / [`Opa::buffer_int`]
//! - **Non-inverting PGA** (x2..x32): [`Opa::pga_ext`] / [`Opa::pga_int`]
//! - **Non-inverting PGA about a reference**: [`Opa::pga_biased_ext`] /
//!   [`Opa::pga_biased_int`], which drive the bottom of the gain ladder from a
//!   [`LadderBottom`] source instead of grounding it
//!
//! `_ext` variants drive the OPAx_OUT pin; `_int` variants keep the output
//! off the pins and only route it to the ADC. Both can be sampled by the ADC
//! by passing a mutable reference to the output handle to
//! [`crate::adc::Adc::blocking_read`] or [`crate::adc::Adc::irq_read`].
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

/// Gain for the non-inverting PGA topology.
///
/// The discriminants are the CFG.GAIN encoding. The ladder starts at x2:
/// GAIN=0x0 is listed as not valid for this topology in the TRM, and unity
/// gain is reached through the buffer topology instead, which does not go
/// through the ladder at all.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Gain {
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

/// A source for the bottom of the gain ladder (CFG.MSEL).
///
/// In the non-inverting PGA the ladder bottom is the point the gain pivots
/// about, not merely a return path:
///
/// ```text
/// Vout = gain * Vin + (1 - gain) * Vladder
/// ```
///
/// Grounding it gives the plain `Vout = gain * Vin`, which also means the
/// input's own DC is multiplied by the gain: at x32 an input sitting 50 mV
/// away from where it needs to be moves the output by 1.6 V. Driving the
/// ladder bottom from the DAC12 instead makes the DC operating point of the
/// output a free variable, settable independently of the gain, which is what
/// lets a high gain be used on a signal whose DC is not already placed for it.
///
/// Only the sources that are unambiguous for this topology are exposed. The
/// remaining CFG.MSEL values are an external `OPAx_INy-` pin, for which this
/// driver has no pin trait yet, and the previous instance's ladder top for
/// cascading, which has no meaning on the first instance.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum LadderBottom {
    /// Analog ground, giving `Vout = gain * Vin`.
    #[default]
    Ground,
    /// The DAC12 output, giving `Vout = gain * Vin + (1 - gain) * Vdac`.
    ///
    /// The DAC must be enabled and settled before the amplifier is: it is the
    /// reference the whole transfer function is written against, so bringing
    /// it up afterwards means the first output the ADC sees is referred to
    /// whatever the DAC pin happened to be sitting at.
    Dac12,
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
/// Can also be sampled by passing a mutable reference to this handle to the
/// ADC. The amplifier is disabled when this is dropped.
pub struct OpaOutput<'d, T: Instance> {
    _inner: &'d Opa<'d, T>,
}

/// An enabled OPA whose output is only routed internally (to the ADC).
///
/// Sample it by passing a mutable reference to this handle to the ADC. The
/// amplifier is disabled when this is dropped.
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
    ///
    /// The gain ladder is grounded. Use [`Opa::pga_biased_ext`] to pivot the
    /// gain about a reference instead.
    pub fn pga_ext<'a>(
        &'a mut self,
        input: impl Into<NonInvertingInput<'a, T>>,
        output: Peri<'a, impl OutputPin<T>>,
        gain: Gain,
    ) -> OpaOutput<'a, T> {
        self.pga_biased_ext(input, output, gain, LadderBottom::Ground)
    }

    /// Non-inverting PGA: output is `gain * input`, routed only to the ADC.
    ///
    /// The gain ladder is grounded. Use [`Opa::pga_biased_int`] to pivot the
    /// gain about a reference instead.
    pub fn pga_int<'a>(
        &'a mut self,
        input: impl Into<NonInvertingInput<'a, T>>,
        gain: Gain,
    ) -> OpaInternalOutput<'a, T> {
        self.pga_biased_int(input, gain, LadderBottom::Ground)
    }

    /// Non-inverting PGA about `ladder`, driving the output pin.
    ///
    /// Output is `gain * input + (1 - gain) * ladder`; see [`LadderBottom`].
    pub fn pga_biased_ext<'a>(
        &'a mut self,
        input: impl Into<NonInvertingInput<'a, T>>,
        output: Peri<'a, impl OutputPin<T>>,
        gain: Gain,
        ladder: LadderBottom,
    ) -> OpaOutput<'a, T> {
        SealedOutputPin::setup(&*output);
        let mut cfg = Self::pga_cfg(input.into(), gain, ladder);
        cfg.set_outpin(true);
        self.enable(cfg);
        OpaOutput { _inner: self }
    }

    /// Non-inverting PGA about `ladder`, routed only to the ADC.
    ///
    /// Output is `gain * input + (1 - gain) * ladder`; see [`LadderBottom`].
    pub fn pga_biased_int<'a>(
        &'a mut self,
        input: impl Into<NonInvertingInput<'a, T>>,
        gain: Gain,
        ladder: LadderBottom,
    ) -> OpaInternalOutput<'a, T> {
        self.enable(Self::pga_cfg(input.into(), gain, ladder));
        OpaInternalOutput { _inner: self }
    }

    fn pga_cfg(input: NonInvertingInput<'_, T>, gain: Gain, ladder: LadderBottom) -> regs::Cfg {
        // Feedback from the tap, with the ladder bottom held at `ladder`.
        let mut cfg = regs::Cfg(0);
        cfg.set_psel(input.channel);
        cfg.set_nsel(vals::Nsel::OANRTAP);
        cfg.set_msel(match ladder {
            LadderBottom::Ground => vals::Msel::VSS,
            LadderBottom::Dac12 => vals::Msel::DAC12OUT,
        });
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
        impl<'d> crate::adc::AdcChannel<crate::peripherals::$adc>
            for crate::opa::OpaOutput<'d, crate::peripherals::$inst>
        {
        }
        impl<'d> crate::adc::SealedAdcChannel<crate::peripherals::$adc>
            for crate::opa::OpaOutput<'d, crate::peripherals::$inst>
        {
            fn channel(&self) -> u8 {
                $ch
            }
        }

        impl<'d> crate::adc::AdcChannel<crate::peripherals::$adc>
            for crate::opa::OpaInternalOutput<'d, crate::peripherals::$inst>
        {
        }
        impl<'d> crate::adc::SealedAdcChannel<crate::peripherals::$adc>
            for crate::opa::OpaInternalOutput<'d, crate::peripherals::$inst>
        {
            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}
