//! CTimer driver.

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitCell;

use crate::clkout::Div4;
use crate::clocks::periph_helpers::{CTimerClockSel, CTimerConfig};
use crate::clocks::{ClockError, Gate, PoweredClock, WakeGuard, enable_and_reset};
use crate::gpio::{GpioPin, SealedPin};
use crate::{interrupt, pac};

pub mod pwm;

/// Error information type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Clock configuration error.
    ClockSetup(ClockError),

    /// Other internal errors or unexpected state.
    Other,
}

/// CTimer configuration
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub struct Config {
    /// Powered clock configuration
    pub power: PoweredClock,
    /// CTimer clock source
    pub source: CTimerClockSel,
    /// CTimer pre-divider
    pub div: Div4,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: CTimerClockSel::FroLfDiv,
            div: const { Div4::no_div() },
        }
    }
}

/// CTimer core driver.
#[derive(Clone)]
pub struct CTimer<'d> {
    info: &'static Info,
    _freq: u32,
    _wg: Option<WakeGuard>,
    _phantom: PhantomData<&'d mut ()>,
}

impl<'d> CTimer<'d> {
    /// Create a new instance of the CTimer core cdriver.
    pub fn new<T: Instance>(_peri: Peri<'d, T>, config: Config) -> Result<Self, Error> {
        // Enable clocks
        let conf = CTimerConfig {
            power: config.power,
            source: config.source,
            div: config.div,
            instance: T::CLOCK_INSTANCE,
        };

        let parts = unsafe { enable_and_reset::<T>(&conf).map_err(Error::ClockSetup)? };

        let inst = Self {
            info: T::info(),
            _freq: parts.freq,
            _wg: parts.wake_guard,
            _phantom: PhantomData,
        };

        // Enable CTimer
        inst.info.regs().tcr().modify(|w| w.set_cen(true));

        Ok(inst)
    }
}

/// CTimer interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // Clear interrupt status
        let r = T::info().regs().ir().read().0;
        T::info().regs().ir().write(|w| w.0 = r);
        T::info().wait_cell().wake();
    }
}

struct Info {
    regs: pac::ctimer::Ctimer,
    wait_cell: WaitCell,
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::ctimer::Ctimer {
        self.regs
    }

    #[inline(always)]
    fn wait_cell(&self) -> &WaitCell {
        &self.wait_cell
    }
}

unsafe impl Sync for Info {}

trait SealedInstance {
    fn info() -> &'static Info;
}

/// CTimer Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send + Gate<MrccPeriphConfig = CTimerConfig> {
    /// Interrupt for this I2C instance.
    type Interrupt: interrupt::typelevel::Interrupt;
    /// Clock instance
    const CLOCK_INSTANCE: crate::clocks::periph_helpers::CTimerInstance;
}

macro_rules! impl_instance {
    ($peri:ident, $clock:ident) => {
        impl SealedInstance for crate::peripherals::$peri {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: pac::$peri,
                    wait_cell: WaitCell::new(),
                };
                &INFO
            }
        }

        impl Instance for crate::peripherals::$peri {
            type Interrupt = crate::interrupt::typelevel::$peri;
            const CLOCK_INSTANCE: crate::clocks::periph_helpers::CTimerInstance =
                crate::clocks::periph_helpers::CTimerInstance::$clock;
        }
    };
}

impl_instance!(CTIMER0, CTimer0);
impl_instance!(CTIMER1, CTimer1);
impl_instance!(CTIMER2, CTimer2);
impl_instance!(CTIMER3, CTimer3);
impl_instance!(CTIMER4, CTimer4);

trait SealedPwmChannel<T: Instance> {
    fn number(&self) -> usize;
}

/// CTimer channel
#[allow(private_bounds)]
pub trait PwmChannel<T: Instance>: SealedPwmChannel<T> + PeripheralType + Into<AnyChannel> + 'static + Send {}

macro_rules! impl_channel {
    ($ch:ident, $peri:ident, $n:literal) => {
        impl SealedPwmChannel<crate::peripherals::$peri> for crate::peripherals::$ch {
            #[inline(always)]
            fn number(&self) -> usize {
                $n
            }
        }

        impl PwmChannel<crate::peripherals::$peri> for crate::peripherals::$ch {}

        impl From<crate::peripherals::$ch> for AnyChannel {
            fn from(value: crate::peripherals::$ch) -> Self {
                Self {
                    number: value.number(),
                }
            }
        }
    };
}

impl_channel!(CTIMER0_CH0, CTIMER0, 0);
impl_channel!(CTIMER0_CH1, CTIMER0, 1);
impl_channel!(CTIMER0_CH2, CTIMER0, 2);
impl_channel!(CTIMER0_CH3, CTIMER0, 3);

impl_channel!(CTIMER1_CH0, CTIMER1, 0);
impl_channel!(CTIMER1_CH1, CTIMER1, 1);
impl_channel!(CTIMER1_CH2, CTIMER1, 2);
impl_channel!(CTIMER1_CH3, CTIMER1, 3);

impl_channel!(CTIMER2_CH0, CTIMER2, 0);
impl_channel!(CTIMER2_CH1, CTIMER2, 1);
impl_channel!(CTIMER2_CH2, CTIMER2, 2);
impl_channel!(CTIMER2_CH3, CTIMER2, 3);

impl_channel!(CTIMER3_CH0, CTIMER3, 0);
impl_channel!(CTIMER3_CH1, CTIMER3, 1);
impl_channel!(CTIMER3_CH2, CTIMER3, 2);
impl_channel!(CTIMER3_CH3, CTIMER3, 3);

impl_channel!(CTIMER4_CH0, CTIMER4, 0);
impl_channel!(CTIMER4_CH1, CTIMER4, 1);
impl_channel!(CTIMER4_CH2, CTIMER4, 2);
impl_channel!(CTIMER4_CH3, CTIMER4, 3);

/// Type-erase CTIMER channel
pub struct AnyChannel {
    number: usize,
}

impl AnyChannel {
    fn number(&self) -> usize {
        self.number
    }
}

embassy_hal_internal::impl_peripheral!(AnyChannel);

/// Seal a trait
trait SealedInputPin {
    #[allow(dead_code)]
    fn number(&self) -> usize;
}

/// Seal a trait
trait SealedOutputPin<T: Instance> {
    fn number(&self) -> usize;
}

/// CTimer input pin.
#[allow(private_bounds)]
pub trait InputPin: GpioPin + SealedInputPin + PeripheralType {
    fn mux(&self);
}

/// CTimer output pin.
#[allow(private_bounds)]
pub trait OutputPin<T: Instance>: GpioPin + SealedOutputPin<T> + PeripheralType {
    fn mux(&self);
}

macro_rules! impl_input_pin {
    ($pin:ident, $fn:ident, $n:expr) => {
        impl SealedInputPin for crate::peripherals::$pin {
            #[inline(always)]
            fn number(&self) -> usize {
                $n
            }
        }

        impl InputPin for crate::peripherals::$pin {
            #[inline(always)]
            fn mux(&self) {
                self.set_pull(crate::gpio::Pull::Disabled);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Double.into());
                self.set_function(crate::pac::port::vals::Mux::$fn);
                self.set_enable_input_buffer(true);
            }
        }
    };
}

macro_rules! impl_output_pin {
    ($pin:ident, $peri:ident, $fn:ident, $n:expr) => {
        impl SealedOutputPin<crate::peripherals::$peri> for crate::peripherals::$pin {
            #[inline(always)]
            fn number(&self) -> usize {
                $n
            }
        }

        impl OutputPin<crate::peripherals::$peri> for crate::peripherals::$pin {
            #[inline(always)]
            fn mux(&self) {
                self.set_pull(crate::gpio::Pull::Disabled);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Normal.into());
                self.set_function(crate::pac::port::vals::Mux::$fn);
                self.set_enable_input_buffer(false);
            }
        }
    };
}

// Input pins

#[cfg(feature = "swd-as-gpio")]
impl_input_pin!(P0_0, MUX4, 0);
#[cfg(feature = "swd-as-gpio")]
impl_input_pin!(P0_1, MUX4, 1);
#[cfg(feature = "jtag-extras-as-gpio")]
impl_input_pin!(P0_6, MUX4, 2);

impl_input_pin!(P0_20, MUX4, 0);
impl_input_pin!(P0_21, MUX4, 1);
impl_input_pin!(P0_22, MUX4, 2);
impl_input_pin!(P0_23, MUX4, 3);

impl_input_pin!(P1_0, MUX4, 4);
impl_input_pin!(P1_1, MUX4, 5);
impl_input_pin!(P1_2, MUX5, 0);
impl_input_pin!(P1_3, MUX5, 1);
impl_input_pin!(P1_6, MUX4, 6);
impl_input_pin!(P1_7, MUX4, 7);
impl_input_pin!(P1_8, MUX4, 8);
impl_input_pin!(P1_9, MUX4, 9);
impl_input_pin!(P1_14, MUX4, 10);
impl_input_pin!(P1_15, MUX4, 11);

#[cfg(feature = "sosc-as-gpio")]
impl_input_pin!(P1_30, MUX4, 16);
#[cfg(feature = "sosc-as-gpio")]
impl_input_pin!(P1_31, MUX4, 17);

impl_input_pin!(P2_0, MUX4, 16);
impl_input_pin!(P2_1, MUX4, 17);
impl_input_pin!(P2_2, MUX4, 12);
impl_input_pin!(P2_3, MUX4, 13);
impl_input_pin!(P2_4, MUX4, 14);
impl_input_pin!(P2_5, MUX4, 15);
impl_input_pin!(P2_6, MUX4, 18);
impl_input_pin!(P2_7, MUX4, 19);

impl_input_pin!(P3_0, MUX4, 16);
impl_input_pin!(P3_1, MUX4, 17);
impl_input_pin!(P3_8, MUX4, 4);
impl_input_pin!(P3_9, MUX4, 5);
impl_input_pin!(P3_14, MUX4, 6);
impl_input_pin!(P3_15, MUX4, 7);
impl_input_pin!(P3_16, MUX4, 8);
impl_input_pin!(P3_17, MUX4, 9);
impl_input_pin!(P3_22, MUX4, 10);
impl_input_pin!(P3_27, MUX4, 13);
impl_input_pin!(P3_28, MUX4, 12);
impl_input_pin!(P3_29, MUX4, 3);

impl_input_pin!(P4_6, MUX4, 6);
impl_input_pin!(P4_7, MUX4, 7);

// Output pins
#[cfg(feature = "swd-swo-as-gpio")]
impl_output_pin!(P0_2, CTIMER0, MUX4, 0);
#[cfg(feature = "jtag-extras-as-gpio")]
impl_output_pin!(P0_3, CTIMER0, MUX4, 1);
impl_output_pin!(P0_16, CTIMER0, MUX4, 0);
impl_output_pin!(P0_17, CTIMER0, MUX4, 1);
impl_output_pin!(P0_18, CTIMER0, MUX4, 2);
impl_output_pin!(P0_19, CTIMER0, MUX4, 3);
impl_output_pin!(P0_22, CTIMER0, MUX5, 0);
impl_output_pin!(P0_23, CTIMER0, MUX5, 1);

impl_output_pin!(P1_0, CTIMER0, MUX5, 2);
impl_output_pin!(P1_1, CTIMER0, MUX5, 3);
impl_output_pin!(P1_2, CTIMER1, MUX4, 0);
impl_output_pin!(P1_3, CTIMER1, MUX4, 1);
impl_output_pin!(P1_4, CTIMER1, MUX4, 2);
impl_output_pin!(P1_5, CTIMER1, MUX4, 3);
impl_output_pin!(P1_6, CTIMER4, MUX5, 0);
impl_output_pin!(P1_7, CTIMER4, MUX5, 1);
impl_output_pin!(P1_8, CTIMER0, MUX5, 2);
impl_output_pin!(P1_9, CTIMER0, MUX5, 3);
impl_output_pin!(P1_10, CTIMER2, MUX4, 0);
impl_output_pin!(P1_11, CTIMER2, MUX4, 1);
impl_output_pin!(P1_12, CTIMER2, MUX4, 2);
impl_output_pin!(P1_13, CTIMER2, MUX4, 3);
impl_output_pin!(P1_14, CTIMER3, MUX5, 0);
impl_output_pin!(P1_15, CTIMER3, MUX5, 1);

impl_output_pin!(P2_0, CTIMER2, MUX5, 0);
impl_output_pin!(P2_1, CTIMER2, MUX5, 1);
impl_output_pin!(P2_2, CTIMER2, MUX5, 2);
impl_output_pin!(P2_3, CTIMER2, MUX5, 3);
impl_output_pin!(P2_4, CTIMER1, MUX5, 0);
impl_output_pin!(P2_5, CTIMER1, MUX5, 1);
impl_output_pin!(P2_6, CTIMER1, MUX5, 2);
impl_output_pin!(P2_7, CTIMER1, MUX5, 3);
impl_output_pin!(P2_10, CTIMER3, MUX4, 2);
impl_output_pin!(P2_11, CTIMER3, MUX4, 3);
impl_output_pin!(P2_12, CTIMER4, MUX4, 0);
impl_output_pin!(P2_12, CTIMER0, MUX5, 0);
impl_output_pin!(P2_13, CTIMER4, MUX4, 1);
impl_output_pin!(P2_13, CTIMER0, MUX5, 1);
impl_output_pin!(P2_15, CTIMER4, MUX5, 3);
impl_output_pin!(P2_15, CTIMER0, MUX5, 2);
impl_output_pin!(P2_16, CTIMER3, MUX5, 0);
impl_output_pin!(P2_16, CTIMER0, MUX5, 2);
impl_output_pin!(P2_17, CTIMER3, MUX5, 1);
impl_output_pin!(P2_17, CTIMER0, MUX5, 3);
impl_output_pin!(P2_19, CTIMER3, MUX4, 3);
impl_output_pin!(P2_20, CTIMER2, MUX4, 0);
impl_output_pin!(P2_21, CTIMER2, MUX4, 1);
impl_output_pin!(P2_23, CTIMER2, MUX4, 3);

impl_output_pin!(P3_2, CTIMER4, MUX4, 0);
impl_output_pin!(P3_6, CTIMER4, MUX4, 2);
impl_output_pin!(P3_7, CTIMER4, MUX4, 3);
impl_output_pin!(P3_10, CTIMER1, MUX4, 0);
impl_output_pin!(P3_11, CTIMER1, MUX4, 1);
impl_output_pin!(P3_12, CTIMER1, MUX4, 2);
impl_output_pin!(P3_13, CTIMER1, MUX4, 3);
impl_output_pin!(P3_18, CTIMER2, MUX4, 0);
impl_output_pin!(P3_19, CTIMER2, MUX4, 1);
impl_output_pin!(P3_20, CTIMER2, MUX4, 2);
impl_output_pin!(P3_21, CTIMER2, MUX4, 3);
impl_output_pin!(P3_27, CTIMER3, MUX5, 1);
impl_output_pin!(P3_28, CTIMER3, MUX5, 2);
#[cfg(feature = "dangerous-reset-as-gpio")]
impl_output_pin!(P3_29, CTIMER3, MUX5, 3);
#[cfg(feature = "sosc-as-gpio")]
impl_output_pin!(P3_30, CTIMER0, MUX4, 2);
#[cfg(feature = "sosc-as-gpio")]
impl_output_pin!(P3_31, CTIMER0, MUX4, 3);

impl_output_pin!(P4_2, CTIMER4, MUX4, 0);
impl_output_pin!(P4_3, CTIMER4, MUX4, 1);
impl_output_pin!(P4_4, CTIMER4, MUX4, 2);
impl_output_pin!(P4_5, CTIMER4, MUX4, 3);
