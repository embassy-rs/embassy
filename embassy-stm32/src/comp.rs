//! Analog Comparator (COMP)
//!
//! This driver currently supports chips with the comp_u5 peripheral version
//! (STM32WBA and STM32U5 series).
#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::rcc::RccInfo;
use crate::{Peri, interrupt};

/// Power mode for the comparator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerMode {
    /// High speed / full power.
    HighSpeed,
    /// Medium speed / medium power.
    MediumSpeed,
    /// Ultra-low power / very low speed.
    UltraLowPower,
}

/// Hysteresis level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Hysteresis {
    /// No hysteresis.
    None,
    /// Low hysteresis.
    Low,
    /// Medium hysteresis.
    Medium,
    /// High hysteresis.
    High,
}

/// Output polarity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputPolarity {
    /// Output is not inverted.
    NotInverted,
    /// Output is inverted.
    Inverted,
}

/// Inverting input selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InvertingInput {
    /// 1/4 of VrefInt.
    OneQuarterVref,
    /// 1/2 of VrefInt.
    HalfVref,
    /// 3/4 of VrefInt.
    ThreeQuarterVref,
    /// VrefInt.
    Vref,
    /// DAC channel 1 output.
    Dac1,
    /// DAC channel 2 output.
    Dac2,
    /// External IO pin (INM1).
    InputPin,
}

/// Blanking source selection.
///
/// Blanking allows masking the comparator output during specific timer events
/// to avoid false triggering during switching noise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BlankingSource {
    /// No blanking.
    #[default]
    None,
    /// Timer blanking source 1 (check datasheet for specific timer mapping).
    Blank1,
    /// Timer blanking source 2 (check datasheet for specific timer mapping).
    Blank2,
    /// Timer blanking source 3 (check datasheet for specific timer mapping).
    Blank3,
}

/// Window mode configuration.
///
/// Window mode allows two comparators to work together to detect if a signal
/// is within a voltage window defined by the two comparators' thresholds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WindowMode {
    /// Window mode disabled. Each comparator works independently.
    #[default]
    Disabled,
    /// Window mode enabled. This comparator uses the non-inverting input
    /// from the other comparator in the pair (COMP1/COMP2).
    Enabled,
}

/// Window output mode for window comparisons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WindowOutput {
    /// Output is the comparator's own value.
    #[default]
    OwnValue,
    /// Output is XOR of both comparators in the pair (for window detection).
    XorValue,
}

/// Configuration for the comparator.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Power mode.
    pub power_mode: PowerMode,
    /// Hysteresis level.
    pub hysteresis: Hysteresis,
    /// Output polarity.
    pub output_polarity: OutputPolarity,
    /// Inverting input selection.
    pub inverting_input: InvertingInput,
    /// Blanking source selection.
    pub blanking_source: BlankingSource,
    /// Window mode configuration.
    pub window_mode: WindowMode,
    /// Window output mode.
    pub window_output: WindowOutput,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            power_mode: PowerMode::HighSpeed,
            hysteresis: Hysteresis::None,
            output_polarity: OutputPolarity::NotInverted,
            inverting_input: InvertingInput::HalfVref,
            blanking_source: BlankingSource::None,
            window_mode: WindowMode::Disabled,
            window_output: WindowOutput::OwnValue,
        }
    }
}

/// Comparator state for async operations.
pub struct State {
    waker: AtomicWaker,
}

impl State {
    /// Create a new state.
    pub const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

/// Interrupt handler for COMP.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // The COMP interrupt is triggered on output transition.
        // We disable the EXTI interrupt and wake the waker.
        // The async code will re-enable the interrupt when needed.
        T::disable_exti_interrupt();
        T::state().waker.wake();
    }
}

/// Comparator driver.
pub struct Comp<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> Comp<'d, T> {
    /// Create a new comparator driver.
    ///
    /// The comparator is configured but not enabled. Use [`enable`](Self::enable) to enable it.
    ///
    /// The non-inverting input is connected to the provided pin. The inverting input
    /// is configured via the `config.inverting_input` parameter.
    pub fn new(
        peri: Peri<'d, T>,
        inp: Peri<'_, impl InputPlusPin<T> + crate::gpio::Pin>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        config: Config,
    ) -> Self {
        T::info().rcc.enable_and_reset();
        inp.set_as_analog();

        Self::configure(inp.channel(), config);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self { _peri: peri }
    }

    /// Create a new comparator driver with an external inverting input pin.
    ///
    /// The comparator is configured but not enabled. Use [`enable`](Self::enable) to enable it.
    ///
    /// Both non-inverting and inverting inputs are connected to the provided pins.
    /// The `config.inverting_input` parameter is ignored; the pin determines the input.
    pub fn new_with_input_minus_pin(
        peri: Peri<'d, T>,
        inp: Peri<'_, impl InputPlusPin<T> + crate::gpio::Pin>,
        inm: Peri<'_, impl InputMinusPin<T> + crate::gpio::Pin>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        config: Config,
    ) -> Self {
        T::info().rcc.enable_and_reset();
        inp.set_as_analog();
        inm.set_as_analog();

        // Configure with the pin's channel
        Self::configure_with_input_minus_pin(inp.channel(), inm.channel(), config);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self { _peri: peri }
    }

    #[cfg(comp_u5)]
    fn configure(inp_channel: u8, config: Config) {
        use crate::pac::comp::vals;

        let pwrmode = match config.power_mode {
            PowerMode::HighSpeed => vals::PowerMode::HIGH_SPEED,
            PowerMode::MediumSpeed => vals::PowerMode::MEDIUM_SPEED,
            PowerMode::UltraLowPower => vals::PowerMode::ULTRA_LOW,
        };

        let hyst = match config.hysteresis {
            Hysteresis::None => vals::Hysteresis::NONE,
            Hysteresis::Low => vals::Hysteresis::LOW,
            Hysteresis::Medium => vals::Hysteresis::MEDIUM,
            Hysteresis::High => vals::Hysteresis::HIGH,
        };

        let polarity = match config.output_polarity {
            OutputPolarity::NotInverted => vals::Polarity::NOT_INVERTED,
            OutputPolarity::Inverted => vals::Polarity::INVERTED,
        };

        let inmsel = match config.inverting_input {
            InvertingInput::OneQuarterVref => vals::Inm::QUARTER_VREF,
            InvertingInput::HalfVref => vals::Inm::HALF_VREF,
            InvertingInput::ThreeQuarterVref => vals::Inm::THREE_QUARTER_VREF,
            InvertingInput::Vref => vals::Inm::VREF,
            InvertingInput::Dac1 => vals::Inm::DAC1,
            InvertingInput::Dac2 => vals::Inm::DAC2,
            InvertingInput::InputPin => vals::Inm::INM1,
        };

        let blanksel = match config.blanking_source {
            BlankingSource::None => vals::Blanking::NO_BLANKING,
            BlankingSource::Blank1 => vals::Blanking::BLANK1,
            BlankingSource::Blank2 => vals::Blanking::BLANK2,
            BlankingSource::Blank3 => vals::Blanking::BLANK3,
        };

        let winmode = match config.window_mode {
            WindowMode::Disabled => vals::WindowMode::THIS_INPSEL,
            WindowMode::Enabled => vals::WindowMode::OTHER_INPSEL,
        };

        let winout = match config.window_output {
            WindowOutput::OwnValue => vals::WindowOut::COMP1_VALUE,
            WindowOutput::XorValue => vals::WindowOut::COMP1_VALUE_XOR_COMP2_VALUE,
        };

        T::regs().csr().modify(|w| {
            w.set_inpsel(inp_channel);
            w.set_inmsel(inmsel);
            w.set_pwrmode(pwrmode);
            w.set_hyst(hyst);
            w.set_polarity(polarity);
            w.set_blanksel(blanksel);
            w.set_winmode(winmode);
            w.set_winout(winout);
        });
    }

    #[cfg(comp_u5)]
    fn configure_with_input_minus_pin(inp_channel: u8, inm_channel: u8, config: Config) {
        use crate::pac::comp::vals;

        let pwrmode = match config.power_mode {
            PowerMode::HighSpeed => vals::PowerMode::HIGH_SPEED,
            PowerMode::MediumSpeed => vals::PowerMode::MEDIUM_SPEED,
            PowerMode::UltraLowPower => vals::PowerMode::ULTRA_LOW,
        };

        let hyst = match config.hysteresis {
            Hysteresis::None => vals::Hysteresis::NONE,
            Hysteresis::Low => vals::Hysteresis::LOW,
            Hysteresis::Medium => vals::Hysteresis::MEDIUM,
            Hysteresis::High => vals::Hysteresis::HIGH,
        };

        let polarity = match config.output_polarity {
            OutputPolarity::NotInverted => vals::Polarity::NOT_INVERTED,
            OutputPolarity::Inverted => vals::Polarity::INVERTED,
        };

        // Map the channel to the INM enum value
        // INM1 = 0x06, INM2 = 0x07
        let inmsel = vals::Inm::from_bits(0x06 + inm_channel);

        let blanksel = match config.blanking_source {
            BlankingSource::None => vals::Blanking::NO_BLANKING,
            BlankingSource::Blank1 => vals::Blanking::BLANK1,
            BlankingSource::Blank2 => vals::Blanking::BLANK2,
            BlankingSource::Blank3 => vals::Blanking::BLANK3,
        };

        let winmode = match config.window_mode {
            WindowMode::Disabled => vals::WindowMode::THIS_INPSEL,
            WindowMode::Enabled => vals::WindowMode::OTHER_INPSEL,
        };

        let winout = match config.window_output {
            WindowOutput::OwnValue => vals::WindowOut::COMP1_VALUE,
            WindowOutput::XorValue => vals::WindowOut::COMP1_VALUE_XOR_COMP2_VALUE,
        };

        T::regs().csr().modify(|w| {
            w.set_inpsel(inp_channel);
            w.set_inmsel(inmsel);
            w.set_pwrmode(pwrmode);
            w.set_hyst(hyst);
            w.set_polarity(polarity);
            w.set_blanksel(blanksel);
            w.set_winmode(winmode);
            w.set_winout(winout);
        });
    }

    /// Enable the comparator.
    pub fn enable(&mut self) {
        T::regs().csr().modify(|w| {
            w.set_en(true);
        });
    }

    /// Disable the comparator.
    pub fn disable(&mut self) {
        T::regs().csr().modify(|w| {
            w.set_en(false);
        });
    }

    /// Check if the comparator is enabled.
    pub fn is_enabled(&self) -> bool {
        T::regs().csr().read().en()
    }

    /// Get the current output level.
    ///
    /// Returns `true` if the non-inverting input is higher than the inverting input
    /// (or the opposite if polarity is inverted).
    #[cfg(comp_u5)]
    pub fn output_level(&self) -> bool {
        T::regs().csr().read().value()
    }

    /// Set the blanking source.
    #[cfg(comp_u5)]
    pub fn set_blanking_source(&mut self, source: BlankingSource) {
        use crate::pac::comp::vals;

        let blanksel = match source {
            BlankingSource::None => vals::Blanking::NO_BLANKING,
            BlankingSource::Blank1 => vals::Blanking::BLANK1,
            BlankingSource::Blank2 => vals::Blanking::BLANK2,
            BlankingSource::Blank3 => vals::Blanking::BLANK3,
        };

        T::regs().csr().modify(|w| {
            w.set_blanksel(blanksel);
        });
    }

    /// Wait for the comparator output to go high.
    ///
    /// This method enables the comparator if it's not already enabled,
    /// then waits asynchronously for the output to transition high.
    /// If the output is already high, it returns immediately.
    pub async fn wait_for_high(&mut self) {
        self.enable();

        if self.output_level() {
            return;
        }

        self.wait_for_rising_edge().await;
    }

    /// Wait for the comparator output to go low.
    ///
    /// This method enables the comparator if it's not already enabled,
    /// then waits asynchronously for the output to transition low.
    /// If the output is already low, it returns immediately.
    pub async fn wait_for_low(&mut self) {
        self.enable();

        if !self.output_level() {
            return;
        }

        self.wait_for_falling_edge().await;
    }

    /// Wait for a rising edge on the comparator output.
    ///
    /// This method waits asynchronously for the output to transition from low to high.
    pub async fn wait_for_rising_edge(&mut self) {
        self.enable();

        // Configure EXTI for rising edge
        T::configure_exti(true, false);

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            // Check if interrupt already fired (IMR was cleared by handler)
            if !T::is_exti_interrupt_enabled() {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// Wait for a falling edge on the comparator output.
    ///
    /// This method waits asynchronously for the output to transition from high to low.
    pub async fn wait_for_falling_edge(&mut self) {
        self.enable();

        // Configure EXTI for falling edge
        T::configure_exti(false, true);

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            // Check if interrupt already fired (IMR was cleared by handler)
            if !T::is_exti_interrupt_enabled() {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// Wait for any edge (rising or falling) on the comparator output.
    ///
    /// This method waits asynchronously for any output transition.
    pub async fn wait_for_any_edge(&mut self) {
        self.enable();

        // Configure EXTI for both edges
        T::configure_exti(true, true);

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            // Check if interrupt already fired (IMR was cleared by handler)
            if !T::is_exti_interrupt_enabled() {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }
}

impl<'d, T: Instance> Drop for Comp<'d, T> {
    fn drop(&mut self) {
        T::regs().csr().modify(|w| {
            w.set_en(false);
        });
        T::disable_exti_interrupt();
        T::info().rcc.disable();
    }
}

pub(crate) struct Info {
    rcc: RccInfo,
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
    fn regs() -> crate::pac::comp::Comp;
    fn state() -> &'static State;
    fn exti_line() -> u8;
    fn configure_exti(rising: bool, falling: bool);
    fn enable_exti_interrupt();
    fn disable_exti_interrupt();
    fn is_exti_interrupt_enabled() -> bool;
    fn clear_exti_pending();
}

pub(crate) trait SealedInputPlusPin<T: Instance> {
    fn channel(&self) -> u8;
}

pub(crate) trait SealedInputMinusPin<T: Instance> {
    fn channel(&self) -> u8;
}

/// Comparator instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Interrupt type for this instance.
    type Interrupt: Interrupt;
}

/// Non-inverting input pin trait.
#[allow(private_bounds)]
pub trait InputPlusPin<T: Instance>: SealedInputPlusPin<T> {}

/// Inverting input pin trait.
#[allow(private_bounds)]
pub trait InputMinusPin<T: Instance>: SealedInputMinusPin<T> {}

macro_rules! impl_comp {
    ($inst:ident, $exti_line:expr) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn info() -> &'static Info {
                use crate::rcc::SealedRccPeripheral;
                static INFO: Info = Info {
                    rcc: crate::peripherals::$inst::RCC_INFO,
                };
                &INFO
            }

            fn regs() -> crate::pac::comp::Comp {
                crate::pac::$inst
            }

            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }

            fn exti_line() -> u8 {
                $exti_line
            }

            fn configure_exti(rising: bool, falling: bool) {
                use crate::pac::EXTI;

                let line = Self::exti_line() as usize;

                critical_section::with(|_| {
                    // Configure rising/falling edge triggers
                    EXTI.rtsr(0).modify(|w| w.set_line(line, rising));
                    EXTI.ftsr(0).modify(|w| w.set_line(line, falling));

                    // Clear any pending interrupt
                    Self::clear_exti_pending();

                    // Enable the interrupt
                    Self::enable_exti_interrupt();
                });
            }

            fn enable_exti_interrupt() {
                use crate::pac::EXTI;
                let line = Self::exti_line() as usize;

                #[cfg(any(
                    exti_c0, exti_g0, exti_u0, exti_l5, exti_u5, exti_u3, exti_h5, exti_h50, exti_n6
                ))]
                EXTI.imr(0).modify(|w| w.set_line(line, true));

                #[cfg(not(any(
                    exti_c0, exti_g0, exti_u0, exti_l5, exti_u5, exti_u3, exti_h5, exti_h50, exti_n6
                )))]
                EXTI.imr(0).modify(|w| w.set_line(line, true));
            }

            fn disable_exti_interrupt() {
                use crate::pac::EXTI;
                let line = Self::exti_line() as usize;
                EXTI.imr(0).modify(|w| w.set_line(line, false));
            }

            fn is_exti_interrupt_enabled() -> bool {
                use crate::pac::EXTI;
                let line = Self::exti_line() as usize;
                EXTI.imr(0).read().line(line)
            }

            fn clear_exti_pending() {
                use crate::pac::EXTI;
                let line = Self::exti_line() as usize;

                #[cfg(not(any(
                    exti_c0, exti_g0, exti_u0, exti_l5, exti_u5, exti_u3, exti_h5, exti_h50, exti_n6
                )))]
                EXTI.pr(0).write(|w| w.set_line(line, true));

                #[cfg(any(
                    exti_c0, exti_g0, exti_u0, exti_l5, exti_u5, exti_u3, exti_h5, exti_h50, exti_n6
                ))]
                {
                    EXTI.rpr(0).write(|w| w.set_line(line, true));
                    EXTI.fpr(0).write(|w| w.set_line(line, true));
                }
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::_generated::peripheral_interrupts::$inst::WKUP;
        }
    };
}

// COMP1 uses EXTI line 17, COMP2 uses EXTI line 18
foreach_peripheral! {
    (comp, COMP1) => {
        impl_comp!(COMP1, 17);
    };
    (comp, COMP2) => {
        impl_comp!(COMP2, 18);
    };
}

#[allow(unused_macros)]
macro_rules! impl_comp_inp_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl crate::comp::InputPlusPin<crate::peripherals::$inst> for crate::peripherals::$pin {}
        impl crate::comp::SealedInputPlusPin<crate::peripherals::$inst> for crate::peripherals::$pin {
            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! impl_comp_inm_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl crate::comp::InputMinusPin<crate::peripherals::$inst> for crate::peripherals::$pin {}
        impl crate::comp::SealedInputMinusPin<crate::peripherals::$inst> for crate::peripherals::$pin {
            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}
