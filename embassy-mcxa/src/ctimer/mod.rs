//! CTimer driver.

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitCell;

use crate::clkout::Div4;
use crate::clocks::periph_helpers::{CTimerClockSel, CTimerConfig};
use crate::clocks::{ClockError, Gate, PoweredClock, WakeGuard, enable_and_reset};
use crate::gpio::{GpioPin, SealedPin};
use crate::interrupt::typelevel::Interrupt;
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
    /// LPI2C clock source
    pub source: CTimerClockSel,
    /// LPI2C pre-divider
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
pub struct CTimer<'d> {
    info: &'static Info,
    _freq: u32,
    _wg: Option<WakeGuard>,
    _phantom: PhantomData<&'d mut ()>,
}

impl<'d> CTimer<'d> {
    /// Create a new instance of the CTimer core cdriver.
    pub fn new<T: Instance>(
        _peri: Peri<'d, T>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
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

        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { T::Interrupt::enable() };

        // Enable CTimer
        inst.info.regs().tcr().modify(|_, w| w.cen().enabled());

        Ok(inst)
    }

    /// Split the `CTimer` into its constituent channels.
    ///
    /// Consumes `self` and produces a representation of CTimer's
    /// channels which can be used to create `Pwm`, `Capture`, or
    /// `Timer`.
    pub fn split(self) -> Channels<'d> {
        Channels {
            ch0: Channel::new(self.info, 0, self._freq, self._wg.clone()),
            ch1: Channel::new(self.info, 1, self._freq, self._wg.clone()),
            ch2: Channel::new(self.info, 2, self._freq, self._wg.clone()),
            ch3: Channel::new(self.info, 3, self._freq, self._wg.clone()),
        }
    }

    /// Split the `CTimer` into its constituent channels by mutable
    /// reference.
    ///
    /// Consumes `self` and produces a representation of CTimer's
    /// channels which can be used to create `Pwm`, `Capture`, or
    /// `Timer`.
    pub fn split_ref(&mut self) -> ChannelsRef<'d> {
        ChannelsRef {
            ch0: Channel::new(self.info, 0, self._freq, self._wg.clone()),
            ch1: Channel::new(self.info, 1, self._freq, self._wg.clone()),
            ch2: Channel::new(self.info, 2, self._freq, self._wg.clone()),
            ch3: Channel::new(self.info, 3, self._freq, self._wg.clone()),
        }
    }
}

/// CTimer channels
pub struct Channels<'d> {
    ch0: Channel<'d>,
    ch1: Channel<'d>,
    ch2: Channel<'d>,
    ch3: Channel<'d>,
}

impl<'d> Channels<'d> {
    /// Consume `self` and return Channel 0
    pub fn ch0(self) -> Channel<'d> {
        self.ch0
    }

    /// Consume `self` and return Channel 0
    pub fn ch1(self) -> Channel<'d> {
        self.ch1
    }

    /// Consume `self` and return Channel 2
    pub fn ch2(self) -> Channel<'d> {
        self.ch2
    }

    /// Consume `self` and return Channel 3
    pub fn ch3(self) -> Channel<'d> {
        self.ch3
    }

    /// Split `self` into all four channels
    pub fn split(self) -> (Channel<'d>, Channel<'d>, Channel<'d>, Channel<'d>) {
        (self.ch0, self.ch1, self.ch2, self.ch3)
    }
}

/// CTimer channels by reference
pub struct ChannelsRef<'d> {
    ch0: Channel<'d>,
    ch1: Channel<'d>,
    ch2: Channel<'d>,
    ch3: Channel<'d>,
}

impl<'d> ChannelsRef<'d> {
    /// Get a reference to channel 0
    pub fn ch0(&'d mut self) -> &'d mut Channel<'d> {
        &mut self.ch0
    }

    /// Get a reference to channel 1
    pub fn ch1(&'d mut self) -> &'d mut Channel<'d> {
        &mut self.ch1
    }

    /// Get a reference to channel 2
    pub fn ch2(&'d mut self) -> &'d mut Channel<'d> {
        &mut self.ch2
    }

    /// Get a reference to channel 3
    pub fn ch3(&'d mut self) -> &'d mut Channel<'d> {
        &mut self.ch3
    }
}

/// CTimer channel
pub struct Channel<'d> {
    info: &'static Info,
    number: usize,
    freq: u32,
    _wg: Option<WakeGuard>,
    _phantom: PhantomData<&'d mut ()>,
}

impl<'d> Channel<'d> {
    /// Create a new channel instance.
    ///
    /// Private because users should not be able to create channels
    /// directly, only by calling `split` or `split_ref` on the
    /// `CTimer` object.
    fn new(info: &'static Info, number: usize, freq: u32, _wg: Option<WakeGuard>) -> Self {
        Channel {
            info,
            number,
            freq,
            _wg,
            _phantom: PhantomData,
        }
    }
}

/// CTimer interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // Clear interrupt status
        T::info().regs().ir().write(|w| {
            w.mr0int()
                .set_bit()
                .mr1int()
                .set_bit()
                .mr2int()
                .set_bit()
                .mr3int()
                .set_bit()
                .cr0int()
                .set_bit()
                .cr1int()
                .set_bit()
                .cr2int()
                .set_bit()
                .cr3int()
                .set_bit()
        });

        T::info().wait_cell().wake();
    }
}

struct Info {
    regs: *const pac::ctimer0::RegisterBlock,
    wait_cell: WaitCell,
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> &pac::ctimer0::RegisterBlock {
        unsafe { &*self.regs }
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
    ($peri:ident, $mod:ident, $clock:ident) => {
        impl SealedInstance for crate::peripherals::$peri {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: pac::$mod::ptr(),
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

impl_instance!(CTIMER0, Ctimer0, CTimer0);
impl_instance!(CTIMER1, Ctimer1, CTimer1);
impl_instance!(CTIMER2, Ctimer2, CTimer2);
impl_instance!(CTIMER3, Ctimer3, CTimer3);
impl_instance!(CTIMER4, Ctimer4, CTimer4);

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
                self.set_function(crate::pac::port0::pcr0::Mux::$fn);
                self.set_enable_input_buffer();
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
                self.set_function(crate::pac::port0::pcr0::Mux::$fn);
                self.set_enable_input_buffer();
            }
        }
    };
}

// Input pins

#[cfg(feature = "swd-as-gpio")]
impl_input_pin!(P0_0, Mux4, 0);
#[cfg(feature = "swd-as-gpio")]
impl_input_pin!(P0_1, Mux4, 1);
#[cfg(feature = "jtag-extras-as-gpio")]
impl_input_pin!(P0_6, Mux4, 2);

impl_input_pin!(P0_20, Mux4, 0);
impl_input_pin!(P0_21, Mux4, 1);
impl_input_pin!(P0_22, Mux4, 2);
impl_input_pin!(P0_23, Mux4, 3);

impl_input_pin!(P1_0, Mux4, 4);
impl_input_pin!(P1_1, Mux4, 5);
impl_input_pin!(P1_2, Mux5, 0);
impl_input_pin!(P1_3, Mux5, 1);
impl_input_pin!(P1_6, Mux4, 6);
impl_input_pin!(P1_7, Mux4, 7);
impl_input_pin!(P1_8, Mux4, 8);
impl_input_pin!(P1_9, Mux4, 9);
impl_input_pin!(P1_14, Mux4, 10);
impl_input_pin!(P1_15, Mux4, 11);

#[cfg(feature = "sosc-as-gpio")]
impl_input_pin!(P1_30, Mux4, 16);
#[cfg(feature = "sosc-as-gpio")]
impl_input_pin!(P1_31, Mux4, 17);

impl_input_pin!(P2_0, Mux4, 16);
impl_input_pin!(P2_1, Mux4, 17);
impl_input_pin!(P2_2, Mux4, 12);
impl_input_pin!(P2_3, Mux4, 13);
impl_input_pin!(P2_4, Mux4, 14);
impl_input_pin!(P2_5, Mux4, 15);
impl_input_pin!(P2_6, Mux4, 18);
impl_input_pin!(P2_7, Mux4, 19);

impl_input_pin!(P3_0, Mux4, 16);
impl_input_pin!(P3_1, Mux4, 17);
impl_input_pin!(P3_8, Mux4, 4);
impl_input_pin!(P3_9, Mux4, 5);
impl_input_pin!(P3_14, Mux4, 6);
impl_input_pin!(P3_15, Mux4, 7);
impl_input_pin!(P3_16, Mux4, 8);
impl_input_pin!(P3_17, Mux4, 9);
impl_input_pin!(P3_22, Mux4, 10);
impl_input_pin!(P3_27, Mux4, 13);
impl_input_pin!(P3_28, Mux4, 12);
impl_input_pin!(P3_29, Mux4, 3);

impl_input_pin!(P4_6, Mux4, 6);
impl_input_pin!(P4_7, Mux4, 7);

// Output pins
#[cfg(feature = "swd-swo-as-gpio")]
impl_output_pin!(P0_2, CTIMER0, Mux4, 0);
#[cfg(feature = "jtag-extras-as-gpio")]
impl_output_pin!(P0_3, CTIMER0, Mux4, 1);
impl_output_pin!(P0_16, CTIMER0, Mux4, 0);
impl_output_pin!(P0_17, CTIMER0, Mux4, 1);
impl_output_pin!(P0_18, CTIMER0, Mux4, 2);
impl_output_pin!(P0_19, CTIMER0, Mux4, 3);
impl_output_pin!(P0_22, CTIMER0, Mux5, 0);
impl_output_pin!(P0_23, CTIMER0, Mux5, 1);

impl_output_pin!(P1_0, CTIMER0, Mux5, 2);
impl_output_pin!(P1_1, CTIMER0, Mux5, 3);
impl_output_pin!(P1_2, CTIMER1, Mux4, 0);
impl_output_pin!(P1_3, CTIMER1, Mux4, 1);
impl_output_pin!(P1_4, CTIMER1, Mux4, 2);
impl_output_pin!(P1_5, CTIMER1, Mux4, 3);
impl_output_pin!(P1_6, CTIMER4, Mux5, 0);
impl_output_pin!(P1_7, CTIMER4, Mux5, 1);
impl_output_pin!(P1_8, CTIMER0, Mux5, 2);
impl_output_pin!(P1_9, CTIMER0, Mux5, 3);
impl_output_pin!(P1_10, CTIMER2, Mux4, 0);
impl_output_pin!(P1_11, CTIMER2, Mux4, 1);
impl_output_pin!(P1_12, CTIMER2, Mux4, 2);
impl_output_pin!(P1_13, CTIMER2, Mux4, 3);
impl_output_pin!(P1_14, CTIMER3, Mux5, 0);
impl_output_pin!(P1_15, CTIMER3, Mux5, 1);

impl_output_pin!(P2_0, CTIMER2, Mux5, 0);
impl_output_pin!(P2_1, CTIMER2, Mux5, 1);
impl_output_pin!(P2_2, CTIMER2, Mux5, 2);
impl_output_pin!(P2_3, CTIMER2, Mux5, 3);
impl_output_pin!(P2_4, CTIMER1, Mux5, 0);
impl_output_pin!(P2_5, CTIMER1, Mux5, 1);
impl_output_pin!(P2_6, CTIMER1, Mux5, 2);
impl_output_pin!(P2_7, CTIMER1, Mux5, 3);
impl_output_pin!(P2_10, CTIMER3, Mux4, 2);
impl_output_pin!(P2_11, CTIMER3, Mux4, 3);
impl_output_pin!(P2_12, CTIMER4, Mux4, 0);
impl_output_pin!(P2_12, CTIMER0, Mux5, 0);
impl_output_pin!(P2_13, CTIMER4, Mux4, 1);
impl_output_pin!(P2_13, CTIMER0, Mux5, 1);
impl_output_pin!(P2_15, CTIMER4, Mux5, 3);
impl_output_pin!(P2_15, CTIMER0, Mux5, 2);
impl_output_pin!(P2_16, CTIMER3, Mux5, 0);
impl_output_pin!(P2_16, CTIMER0, Mux5, 2);
impl_output_pin!(P2_17, CTIMER3, Mux5, 1);
impl_output_pin!(P2_17, CTIMER0, Mux5, 3);
impl_output_pin!(P2_19, CTIMER3, Mux4, 3);
impl_output_pin!(P2_20, CTIMER2, Mux4, 0);
impl_output_pin!(P2_21, CTIMER2, Mux4, 1);
impl_output_pin!(P2_23, CTIMER2, Mux4, 3);

impl_output_pin!(P3_2, CTIMER4, Mux4, 0);
impl_output_pin!(P3_6, CTIMER4, Mux4, 2);
impl_output_pin!(P3_7, CTIMER4, Mux4, 3);
impl_output_pin!(P3_10, CTIMER1, Mux4, 0);
impl_output_pin!(P3_11, CTIMER1, Mux4, 1);
impl_output_pin!(P3_12, CTIMER1, Mux4, 2);
impl_output_pin!(P3_13, CTIMER1, Mux4, 3);
impl_output_pin!(P3_18, CTIMER2, Mux4, 0);
impl_output_pin!(P3_19, CTIMER2, Mux4, 1);
impl_output_pin!(P3_20, CTIMER2, Mux4, 2);
impl_output_pin!(P3_21, CTIMER2, Mux4, 3);
impl_output_pin!(P3_27, CTIMER3, Mux5, 1);
impl_output_pin!(P3_28, CTIMER3, Mux5, 2);
#[cfg(feature = "dangerous-reset-as-gpio")]
impl_output_pin!(P3_29, CTIMER3, Mux5, 3);
#[cfg(feature = "sosc-as-gpio")]
impl_output_pin!(P3_30, CTIMER0, Mux4, 2);
#[cfg(feature = "sosc-as-gpio")]
impl_output_pin!(P3_31, CTIMER0, Mux4, 3);

impl_output_pin!(P4_2, CTIMER4, Mux4, 0);
impl_output_pin!(P4_3, CTIMER4, Mux4, 1);
impl_output_pin!(P4_4, CTIMER4, Mux4, 2);
impl_output_pin!(P4_5, CTIMER4, Mux4, 3);
