//! CTimer driver.

use core::marker::PhantomData;
use core::sync::atomic::AtomicU8;

use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitCell;

use crate::clocks::periph_helpers::{CTimerClockSel, CTimerConfig, Div4};
use crate::clocks::{ClockError, Gate, PoweredClock, WakeGuard, enable_and_reset};
use crate::gpio::GpioPin;
use crate::{interrupt, pac};

pub mod capture;
pub mod pwm;

/// CTimer channel
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Channel {
    /// Channel 0
    Zero,
    /// Channel 1
    One,
    /// Channel 2
    Two,
    /// Channel 3
    Three,
}

impl From<Channel> for usize {
    fn from(value: Channel) -> usize {
        match value {
            Channel::Zero => 0,
            Channel::One => 1,
            Channel::Two => 2,
            Channel::Three => 3,
        }
    }
}

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
            _freq: parts.freq,
            _wg: parts.wake_guard,
            _phantom: PhantomData,
        };

        Ok(inst)
    }
}

pub(crate) struct Info {
    pub(crate) regs: pac::ctimer::Ctimer,
    pub(crate) wait_cell: WaitCell,
    pub(crate) irq_flags: AtomicU8,
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

    #[inline(always)]
    fn irq_flags(&self) -> &AtomicU8 {
        &self.irq_flags
    }
}

unsafe impl Sync for Info {}

pub(crate) trait SealedInstance: Gate<MrccPeriphConfig = CTimerConfig> {
    fn info() -> &'static Info;

    /// Clock instance
    const CLOCK_INSTANCE: crate::clocks::periph_helpers::CTimerInstance;
    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
}

/// CTimer Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this I2C instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_ctimer_instance {
    ($n:literal) => {
        paste::paste! {
            impl crate::ctimer::SealedInstance for crate::peripherals::[<CTIMER $n>] {
                fn info() -> &'static crate::ctimer::Info {
                    static INFO: crate::ctimer::Info = crate::ctimer::Info {
                        regs: crate::pac::[<CTIMER $n>],
                        wait_cell: maitake_sync::WaitCell::new(),
                        irq_flags: core::sync::atomic::AtomicU8::new(0),
                    };
                    &INFO
                }

                const CLOCK_INSTANCE: crate::clocks::periph_helpers::CTimerInstance =
                    crate::clocks::periph_helpers::CTimerInstance::[<CTimer $n>];
                    const PERF_INT_INCR: fn() = crate::perf_counters::[<incr_interrupt_ctimer $n>];
                    const PERF_INT_WAKE_INCR: fn() = crate::perf_counters::[<incr_interrupt_ctimer $n _wake>];
            }

            impl crate::ctimer::Instance for crate::peripherals::[<CTIMER $n>] {
                type Interrupt = crate::interrupt::typelevel::[<CTIMER $n>];
            }

            crate::impl_ctimer_channel!([<CTIMER $n _CH0>], [<CTIMER $n>], Zero);
            crate::impl_ctimer_channel!([<CTIMER $n _CH1>], [<CTIMER $n>], One);
            crate::impl_ctimer_channel!([<CTIMER $n _CH2>], [<CTIMER $n>], Two);
            crate::impl_ctimer_channel!([<CTIMER $n _CH3>], [<CTIMER $n>], Three);
        }
    };
}

pub(crate) trait SealedCTimerChannel<T: Instance> {
    fn number(&self) -> Channel;
}

/// CTimer channel
#[allow(private_bounds)]
pub trait CTimerChannel<T: Instance>:
    SealedCTimerChannel<T> + PeripheralType + Into<AnyChannel> + 'static + Send
{
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_ctimer_channel {
    ($ch:ident, $peri:ident, $n:ident) => {
        impl crate::ctimer::SealedCTimerChannel<crate::peripherals::$peri> for crate::peripherals::$ch {
            #[inline(always)]
            fn number(&self) -> crate::ctimer::Channel {
                crate::ctimer::Channel::$n
            }
        }

        impl crate::ctimer::CTimerChannel<crate::peripherals::$peri> for crate::peripherals::$ch {}

        impl From<crate::peripherals::$ch> for crate::ctimer::AnyChannel {
            fn from(value: crate::peripherals::$ch) -> Self {
                use crate::ctimer::SealedCTimerChannel;
                Self::new(value.number())
            }
        }
    };
}

/// Type-erase CTIMER channel
pub struct AnyChannel {
    number: Channel,
}

impl AnyChannel {
    pub(crate) const fn new(number: Channel) -> Self {
        Self { number }
    }

    fn number(&self) -> Channel {
        self.number
    }
}

embassy_hal_internal::impl_peripheral!(AnyChannel);

/// Seal a trait
pub(crate) trait SealedInputPin<T: Instance> {}

/// Seal a trait
pub(crate) trait SealedOutputPin<T: Instance> {}

/// CTimer input pin.
#[allow(private_bounds)]
pub trait InputPin<T: Instance>: GpioPin + SealedInputPin<T> + PeripheralType {
    fn mux(&self);
}

/// CTimer output pin.
#[allow(private_bounds)]
pub trait OutputPin<T: Instance>: GpioPin + SealedOutputPin<T> + PeripheralType {
    fn mux(&self);
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_ctimer_input_pin {
    ($pin:ident, $peri:ident, $fn:ident) => {
        impl crate::ctimer::SealedInputPin<crate::peripherals::$peri> for crate::peripherals::$pin {}

        impl crate::ctimer::InputPin<crate::peripherals::$peri> for crate::peripherals::$pin {
            #[inline(always)]
            fn mux(&self) {
                use crate::gpio::SealedPin;
                self.set_pull(crate::gpio::Pull::Disabled);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Double.into());
                self.set_function(crate::pac::port::Mux::$fn);
                self.set_enable_input_buffer(true);
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_ctimer_output_pin {
    ($pin:ident, $peri:ident, $fn:ident) => {
        impl crate::ctimer::SealedOutputPin<crate::peripherals::$peri> for crate::peripherals::$pin {}

        impl crate::ctimer::OutputPin<crate::peripherals::$peri> for crate::peripherals::$pin {
            #[inline(always)]
            fn mux(&self) {
                use crate::gpio::SealedPin;
                self.set_pull(crate::gpio::Pull::Disabled);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Normal.into());
                self.set_function(crate::pac::port::Mux::$fn);
                self.set_enable_input_buffer(false);
            }
        }
    };
}
