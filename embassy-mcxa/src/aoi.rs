//! Constructor-configured AND/OR/invert logic for peripheral event signals.
//!
//! Each of AOI0's four outputs is a sum of four product terms over four routed
//! inputs. Construct the driver before enabling downstream consumers, and keep
//! it and the signal producers alive while their outputs are in use. Source
//! selections route signals; they do not enable or own the producing peripheral.
//! Borrow an [`Output`] to connect an event to a CTIMER capture input.
//!
//! AOI has no enable/stop bit. Its combinational logic does not need the register
//! interface clock. Initialization and teardown are not atomic or glitch-free:
//! quiesce consumers first. Dropping the owner forces its outputs low and gates
//! only AOI0's register interface; INPUTMUX routes are left unchanged.
//!
//! # Not yet supported
//!
//! - AOI1 and its separate input-routing bank on devices that provide it.
//! - Runtime reconfiguration, independently owned event channels, or arbitrary
//!   Boolean-expression compilation.
//! - Software output sampling or general destination routing beyond CTIMER
//!   capture. Input routes currently cover ADC0 compare, CMP0, CTIMER match and
//!   LPTMR0 signals; ADC1 compare, external triggers, GPIO events and FlexIO are
//!   not exposed by this driver.
//! - Guaranteed event propagation through deep-sleep/power transitions.
//!
//! Registers require 16-bit accesses (A2 RM 17.7; A5 RM 25.7). The PAC's typed
//! accessors supply that width; a pair of registers must not be combined into a
//! 32-bit access.

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};

use crate::clocks::periph_helpers::NoConfig;
use crate::clocks::{ClockError, Gate, disable, enable_and_reset};
use crate::pac::aoi::InputConfig;
use crate::{pac, peripherals};

/// One of the four AOI event outputs, ADC compare results, or timer matches.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
    /// Event/channel 0.
    Zero,
    /// Event/channel 1.
    One,
    /// Event/channel 2.
    Two,
    /// Event/channel 3.
    Three,
}

impl Event {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }
}

/// Contribution of an input to one product term.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Factor {
    /// Force this factor to zero, making the entire product false.
    #[default]
    Zero,
    /// Force this factor to one, ignoring this input in the product.
    One,
    /// Use the input signal unchanged.
    Direct,
    /// Use the complement of the input signal.
    Inverted,
}

impl From<Factor> for InputConfig {
    fn from(value: Factor) -> Self {
        match value {
            Factor::Zero => Self::Zero,
            Factor::One => Self::One,
            Factor::Direct => Self::Pass,
            Factor::Inverted => Self::Complement,
        }
    }
}

/// Logical AND of the four selected input factors.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ProductTerm {
    /// Factor for input A.
    pub a: Factor,
    /// Factor for input B.
    pub b: Factor,
    /// Factor for input C.
    pub c: Factor,
    /// Factor for input D.
    pub d: Factor,
}

impl ProductTerm {
    /// Constant false; also the hardware-reset product.
    pub const FALSE: Self = Self {
        a: Factor::Zero,
        b: Factor::Zero,
        c: Factor::Zero,
        d: Factor::Zero,
    };

    /// Constant true; replace individual factors to build a product.
    pub const TRUE: Self = Self {
        a: Factor::One,
        b: Factor::One,
        c: Factor::One,
        d: Factor::One,
    };
}

/// Logical OR of exactly four product terms. No expression fitting is required.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Function {
    terms: [ProductTerm; 4],
}

impl Function {
    /// Constant-low output, matching the hardware reset configuration.
    pub const LOW: Self = Self::new([ProductTerm::FALSE; 4]);
    /// Constant-high output.
    pub const HIGH: Self = Self::new([ProductTerm::TRUE; 4]);

    /// Form a sum of four products; use [`ProductTerm::FALSE`] for unused terms.
    pub const fn new(terms: [ProductTerm; 4]) -> Self {
        Self { terms }
    }
}

/// Internal signal routed to one of an event's A/B/C/D inputs.
///
/// These selections do not enable the producer or guarantee its power state.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum InputSource {
    /// ADC0 threshold-compare output.
    Adc0Compare(Event),
    /// Comparator 0 output.
    Cmp0Output,
    /// CTIMER0 match output.
    Ctimer0Match(Event),
    /// CTIMER1 match output.
    Ctimer1Match(Event),
    /// CTIMER2 match output.
    Ctimer2Match(Event),
    /// CTIMER3 match output.
    Ctimer3Match(Event),
    /// CTIMER4 match output.
    Ctimer4Match(Event),
    /// Low-power timer 0 output.
    Lptmr0,
}

impl InputSource {
    pub(crate) fn selector(self) -> pac::inputmux::AoiInputInp {
        // A2 RM 13.6.1.60 / A5 RM 21.6.1.26: these signal encodings agree.
        use pac::inputmux::AoiInputInp as S;
        match self {
            Self::Adc0Compare(Event::Zero) => S::Val1,
            Self::Adc0Compare(Event::One) => S::Val2,
            Self::Adc0Compare(Event::Two) => S::Val3,
            Self::Adc0Compare(Event::Three) => S::Val4,
            Self::Cmp0Output => S::Val5,
            Self::Ctimer0Match(Event::Zero) => S::Val8,
            Self::Ctimer0Match(Event::One) => S::Val9,
            Self::Ctimer0Match(Event::Two) => S::Val10,
            Self::Ctimer0Match(Event::Three) => S::Val11,
            Self::Ctimer1Match(Event::Zero) => S::Val12,
            Self::Ctimer1Match(Event::One) => S::Val13,
            Self::Ctimer1Match(Event::Two) => S::Val14,
            Self::Ctimer1Match(Event::Three) => S::Val15,
            Self::Ctimer2Match(Event::Zero) => S::Val16,
            Self::Ctimer2Match(Event::One) => S::Val17,
            Self::Ctimer2Match(Event::Two) => S::Val18,
            Self::Ctimer2Match(Event::Three) => S::Val19,
            Self::Lptmr0 => S::Val20,
            Self::Ctimer3Match(Event::Zero) => S::Val56,
            Self::Ctimer3Match(Event::One) => S::Val57,
            Self::Ctimer3Match(Event::Two) => S::Val58,
            Self::Ctimer3Match(Event::Three) => S::Val59,
            Self::Ctimer4Match(Event::Zero) => S::Val60,
            Self::Ctimer4Match(Event::One) => S::Val61,
            Self::Ctimer4Match(Event::Two) => S::Val62,
            Self::Ctimer4Match(Event::Three) => S::Val63,
        }
    }
}

/// Four input routes (A, B, C, D) and the function of one event output.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct EventConfig {
    /// Input routes in A, B, C, D order.
    pub inputs: [InputSource; 4],
    /// Sum-of-products function.
    pub function: Function,
}

impl EventConfig {
    /// Configure an event without a board-specific default input selection.
    pub const fn new(inputs: [InputSource; 4], function: Function) -> Self {
        Self { inputs, function }
    }
}

/// Complete constructor-only configuration, indexed by event number.
#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// `None` keeps the event low and leaves its INPUTMUX routes untouched.
    pub events: [Option<EventConfig>; 4],
}

/// AOI initialization failed before its functions were made active.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Peripheral or routing clock setup failed.
    ClockSetup(ClockError),
}

pub(crate) struct Info {
    regs: pac::aoi::Aoi,
    disable_clock: unsafe fn(),
}

unsafe impl Sync for Info {}

pub(crate) trait SealedInstance: Gate<MrccPeriphConfig = NoConfig> {
    fn info() -> &'static Info;
}

/// Supported AOI instance. Currently only AOI0 has complete PAC routing support.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {}

impl SealedInstance for peripherals::AOI0 {
    fn info() -> &'static Info {
        static INFO: Info = Info {
            regs: pac::AOI0,
            disable_clock: disable::<peripherals::AOI0>,
        };
        &INFO
    }
}

impl Instance for peripherals::AOI0 {}

/// Exclusive owner of an AOI instance with an immutable configuration.
pub struct Aoi<'d> {
    info: &'static Info,
    _lifetime: PhantomData<&'d mut ()>,
}

impl<'d> Aoi<'d> {
    /// Configure AOI before enabling its downstream consumers.
    ///
    /// The caller must keep input producers configured while using the outputs.
    /// This does not reset INPUTMUX or provide glitch-free initialization.
    pub fn new<T: Instance>(_peri: Peri<'d, T>, config: Config) -> Result<Self, SetupError> {
        let info = T::info();
        critical_section::with(|_| {
            // SAFETY: the token proves exclusive ownership before initialization.
            let _parts = unsafe { enable_and_reset::<T>(&NoConfig) }.map_err(SetupError::ClockSetup)?;
            let inst = Self {
                info,
                _lifetime: PhantomData,
            };
            for (event, conf) in config.events.iter().enumerate() {
                if let Some(conf) = conf {
                    crate::inputmux::configure_aoi0(event, conf.inputs).map_err(SetupError::ClockSetup)?;
                }
            }
            for (event, conf) in config.events.iter().enumerate() {
                inst.write_function(event, conf.map_or(Function::LOW, |conf| conf.function));
            }
            Ok(inst)
        })
    }

    /// Borrow an event output for a downstream peripheral.
    ///
    /// The borrow keeps AOI alive while the consumer retains the signal. It does
    /// not grant software access to the output's instantaneous logic level.
    pub fn output(&self, event: Event) -> Output<'_> {
        Output {
            event,
            _owner: PhantomData,
        }
    }

    fn write_function(&self, event: usize, function: Function) {
        let (first, second) = match event {
            0 => (self.info.regs.bfcrt010(), self.info.regs.bfcrt230()),
            1 => (self.info.regs.bfcrt011(), self.info.regs.bfcrt231()),
            2 => (self.info.regs.bfcrt012(), self.info.regs.bfcrt232()),
            3 => (self.info.regs.bfcrt013(), self.info.regs.bfcrt233()),
            _ => unreachable!(),
        };
        let [a, b, c, d] = function.terms;
        first.write(|w| {
            w.set_pt0_ac(a.a.into());
            w.set_pt0_bc(a.b.into());
            w.set_pt0_cc(a.c.into());
            w.set_pt0_dc(a.d.into());
            w.set_pt1_ac(b.a.into());
            w.set_pt1_bc(b.b.into());
            w.set_pt1_cc(b.c.into());
            w.set_pt1_dc(b.d.into());
        });
        second.write(|w| {
            w.set_pt2_ac(c.a.into());
            w.set_pt2_bc(c.b.into());
            w.set_pt2_cc(c.c.into());
            w.set_pt2_dc(c.d.into());
            w.set_pt3_ac(d.a.into());
            w.set_pt3_bc(d.b.into());
            w.set_pt3_cc(d.c.into());
            w.set_pt3_dc(d.d.into());
        });
    }
}

impl Drop for Aoi<'_> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            for event in 0..4 {
                self.write_function(event, Function::LOW);
            }
            cortex_m::asm::dsb();
            // SAFETY: all borrowed outputs have been released before the owner.
            unsafe { (self.info.disable_clock)() };
        });
    }
}

/// Borrowed AOI0 event signal; may fan out to multiple consumers.
#[derive(Clone, Copy)]
pub struct Output<'a> {
    pub(crate) event: Event,
    _owner: PhantomData<&'a ()>,
}
