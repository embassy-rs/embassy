//! Low-level timer driver.
mod prescaler;

use embassy_hal_internal::Peri;

#[cfg(any(lptim_v2a, lptim_v2b))]
use super::channel::Channel;
#[cfg(any(lptim_v2a, lptim_v2b))]
mod channel_direction;
#[cfg(any(lptim_v2a, lptim_v2b))]
pub use channel_direction::ChannelDirection;
use prescaler::Prescaler;

use super::Instance;
use crate::rcc;
use crate::time::Hertz;

/// Low-level timer driver.
pub struct Timer<'d, T: Instance> {
    _tim: Peri<'d, T>,
}

impl<'d, T: Instance> Timer<'d, T> {
    /// Create a new timer driver.
    pub fn new(tim: Peri<'d, T>) -> Self {
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

    /// Get max compare value. This depends on the timer frequency and the clock frequency from RCC.
    pub fn get_max_compare_value(&self) -> u16 {
        T::regs().arr().read().arr()
    }
}

#[cfg(any(lptim_v2a, lptim_v2b))]
impl<'d, T: Instance> Timer<'d, T> {
    /// Enable/disable a channel.
    pub fn enable_channel(&self, channel: Channel, enable: bool) {
        T::regs().ccmr(0).modify(|w| {
            w.set_cce(channel.index(), enable);
        });
    }

    /// Get enable/disable state of a channel
    pub fn get_channel_enable_state(&self, channel: Channel) -> bool {
        T::regs().ccmr(0).read().cce(channel.index())
    }

    /// Set compare value for a channel.
    pub fn set_compare_value(&self, channel: Channel, value: u16) {
        T::regs().ccr(channel.index()).modify(|w| w.set_ccr(value));
    }

    /// Get compare value for a channel.
    pub fn get_compare_value(&self, channel: Channel) -> u16 {
        T::regs().ccr(channel.index()).read().ccr()
    }

    /// Set channel direction.
    #[cfg(any(lptim_v2a, lptim_v2b))]
    pub fn set_channel_direction(&self, channel: Channel, direction: ChannelDirection) {
        T::regs()
            .ccmr(0)
            .modify(|w| w.set_ccsel(channel.index(), direction.into()));
    }
}

#[cfg(not(any(lptim_v2a, lptim_v2b)))]
impl<'d, T: Instance> Timer<'d, T> {
    /// Set compare value for a channel.
    pub fn set_compare_value(&self, value: u16) {
        T::regs().cmp().modify(|w| w.set_cmp(value));
    }

    /// Get compare value for a channel.
    pub fn get_compare_value(&self) -> u16 {
        T::regs().cmp().read().cmp()
    }
}
