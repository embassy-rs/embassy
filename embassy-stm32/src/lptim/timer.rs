//! Low-level timer driver.

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

use super::{Channel, Instance};
use crate::pac::lptim::vals;
use crate::rcc;
use crate::time::Hertz;

/// Direction of a low-power timer channel
pub enum ChannelDirection {
    /// Use channel as a PWM output
    OutputPwm,
    /// Use channel as an input capture
    InputCapture,
}

impl From<ChannelDirection> for vals::Ccsel {
    fn from(direction: ChannelDirection) -> Self {
        match direction {
            ChannelDirection::OutputPwm => vals::Ccsel::OUTPUTCOMPARE,
            ChannelDirection::InputCapture => vals::Ccsel::INPUTCAPTURE,
        }
    }
}

enum Prescaler {
    Div1,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
}

impl From<&Prescaler> for vals::Presc {
    fn from(prescaler: &Prescaler) -> Self {
        match prescaler {
            Prescaler::Div1 => vals::Presc::DIV1,
            Prescaler::Div2 => vals::Presc::DIV2,
            Prescaler::Div4 => vals::Presc::DIV4,
            Prescaler::Div8 => vals::Presc::DIV8,
            Prescaler::Div16 => vals::Presc::DIV16,
            Prescaler::Div32 => vals::Presc::DIV32,
            Prescaler::Div64 => vals::Presc::DIV64,
            Prescaler::Div128 => vals::Presc::DIV128,
        }
    }
}

impl From<vals::Presc> for Prescaler {
    fn from(prescaler: vals::Presc) -> Self {
        match prescaler {
            vals::Presc::DIV1 => Prescaler::Div1,
            vals::Presc::DIV2 => Prescaler::Div2,
            vals::Presc::DIV4 => Prescaler::Div4,
            vals::Presc::DIV8 => Prescaler::Div8,
            vals::Presc::DIV16 => Prescaler::Div16,
            vals::Presc::DIV32 => Prescaler::Div32,
            vals::Presc::DIV64 => Prescaler::Div64,
            vals::Presc::DIV128 => Prescaler::Div128,
        }
    }
}

impl From<&Prescaler> for u32 {
    fn from(prescaler: &Prescaler) -> Self {
        match prescaler {
            Prescaler::Div1 => 1,
            Prescaler::Div2 => 2,
            Prescaler::Div4 => 4,
            Prescaler::Div8 => 8,
            Prescaler::Div16 => 16,
            Prescaler::Div32 => 32,
            Prescaler::Div64 => 64,
            Prescaler::Div128 => 128,
        }
    }
}

impl From<u32> for Prescaler {
    fn from(prescaler: u32) -> Self {
        match prescaler {
            1 => Prescaler::Div1,
            2 => Prescaler::Div2,
            4 => Prescaler::Div4,
            8 => Prescaler::Div8,
            16 => Prescaler::Div16,
            32 => Prescaler::Div32,
            64 => Prescaler::Div64,
            128 => Prescaler::Div128,
            _ => unreachable!(),
        }
    }
}

impl Prescaler {
    pub fn from_ticks(ticks: u32) -> Self {
        // We need to scale down to a 16-bit range
        (ticks >> 16).next_power_of_two().into()
    }

    pub fn scale_down(&self, ticks: u32) -> u16 {
        (ticks / u32::from(self)).try_into().unwrap()
    }

    pub fn scale_up(&self, ticks: u16) -> u32 {
        u32::from(self) * ticks as u32
    }
}

/// Low-level timer driver.
pub struct Timer<'d, T: Instance> {
    _tim: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Timer<'d, T> {
    /// Create a new timer driver.
    pub fn new(tim: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(tim);

        rcc::enable_and_reset::<T>();

        Self { _tim: tim }
    }

    /// Enable the timer.
    pub fn enable(&self) {
        T::regs().cr().modify(|w| w.set_enable(true));
    }

    /// Disable the timer.
    pub fn disable(&self) {
        T::regs().cr().modify(|w| w.set_enable(false));
    }

    /// Start the timer in single pulse mode.
    pub fn single_mode_start(&self) {
        T::regs().cr().modify(|w| w.set_sngstrt(true));
    }

    /// Start the timer in continuous mode.
    pub fn continuous_mode_start(&self) {
        T::regs().cr().modify(|w| w.set_cntstrt(true));
    }

    /// Set channel direction.
    pub fn set_channel_direction(&self, channel: Channel, direction: ChannelDirection) {
        T::regs()
            .ccmr()
            .modify(|w| w.set_ccsel(channel.index(), direction.into()));
    }

    /// Set the frequency of how many times per second the timer counts up to the max value or down to 0.
    pub fn set_frequency(&self, frequency: Hertz) {
        let f = frequency.0;
        assert!(f > 0);

        let pclk_f = T::frequency().0;

        let pclk_ticks_per_timer_period = pclk_f / f;

        let psc = Prescaler::from_ticks(pclk_ticks_per_timer_period);
        let arr = psc.scale_down(pclk_ticks_per_timer_period);

        T::regs().cfgr().modify(|r| r.set_presc((&psc).into()));
        T::regs().arr().modify(|r| r.set_arr(arr.into()));
    }

    /// Get the timer frequency.
    pub fn get_frequency(&self) -> Hertz {
        let pclk_f = T::frequency();
        let arr = T::regs().arr().read().arr();
        let psc = Prescaler::from(T::regs().cfgr().read().presc());

        pclk_f / psc.scale_up(arr)
    }

    /// Get the clock frequency of the timer (before prescaler is applied).
    pub fn get_clock_frequency(&self) -> Hertz {
        T::frequency()
    }

    /// Enable/disable a channel.
    pub fn enable_channel(&self, channel: Channel, enable: bool) {
        T::regs().ccmr().modify(|w| {
            w.set_cce(channel.index(), enable);
        });
    }

    /// Get enable/disable state of a channel
    pub fn get_channel_enable_state(&self, channel: Channel) -> bool {
        T::regs().ccmr().read().cce(channel.index())
    }

    /// Set compare value for a channel.
    pub fn set_compare_value(&self, channel: Channel, value: u16) {
        T::regs().ccr(channel.index()).modify(|w| w.set_ccr(value));
    }

    /// Get compare value for a channel.
    pub fn get_compare_value(&self, channel: Channel) -> u16 {
        T::regs().ccr(channel.index()).read().ccr()
    }

    /// Get max compare value. This depends on the timer frequency and the clock frequency from RCC.
    pub fn get_max_compare_value(&self) -> u16 {
        T::regs().arr().read().arr()
    }
}
