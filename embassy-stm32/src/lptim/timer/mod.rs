//! Low-level timer driver.
mod prescaler;

use embassy_hal_internal::Peri;

#[cfg(any(lptim_v2a, lptim_v2b, lptim_n6))]
use super::channel::Channel;
#[cfg(any(lptim_v2a, lptim_v2b, lptim_n6))]
mod channel_direction;
#[cfg(any(lptim_v2a, lptim_v2b, lptim_n6))]
pub use channel_direction::ChannelDirection;
use prescaler::Prescaler;

use super::Instance;
use crate::lptim::vals::{self, Filter, Trigen};
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
        let presc = vals::Presc::from(&psc);

        T::regs().cfgr().modify(|r| {
            #[cfg(not(lptim_n6))]
            r.set_presc(presc);
            #[cfg(lptim_n6)]
            r.set_presc(presc.to_bits());
        });
        T::regs().arr().modify(|r| r.set_arr(arr.into()));
    }

    /// Get the timer frequency.
    pub fn get_frequency(&self) -> Hertz {
        let pclk_f = T::frequency();
        let arr = T::regs().arr().read().arr();
        let presc = {
            #[cfg(not(lptim_n6))]
            {
                T::regs().cfgr().read().presc()
            }
            #[cfg(lptim_n6)]
            {
                vals::Presc::from_bits(T::regs().cfgr().read().presc())
            }
        };
        let psc = Prescaler::from(presc);

        pclk_f / psc.scale_up(arr)
    }

    /// Get the clock frequency of the timer (before prescaler is applied).
    pub fn get_clock_frequency(&self) -> Hertz {
        T::frequency()
    }

    /// Select the trigger source used when external trigger start is enabled.
    ///
    /// The source index maps to device-specific `lptim_ext_trigX` inputs (0..=7).
    pub fn set_trigger_source(&self, source: u8) {
        assert!(source < 8, "LPTIM trigger source must be in range 0..8");
        T::regs().cfgr().modify(|r| r.set_trigsel(source));
    }

    /// Configure how trigger edges start the counter.
    ///
    /// Use [`Trigen::Software`] for software start. Any edge mode enables
    /// external trigger start.
    pub fn set_trigger_mode(&self, mode: Trigen) {
        T::regs().cfgr().modify(|r| {
            #[cfg(not(lptim_n6))]
            r.set_trigen(mode);
            #[cfg(lptim_n6)]
            r.set_trigen(mode.to_bits());
        });
    }

    /// Configure the digital filter applied to trigger input transitions.
    pub fn set_trigger_filter(&self, filter: Filter) {
        T::regs().cfgr().modify(|r| {
            #[cfg(not(lptim_n6))]
            r.set_trgflt(filter);
            #[cfg(lptim_n6)]
            r.set_trgflt(filter.to_bits());
        });
    }

    /// Convenience helper to enable external trigger start with source, edge and filter.
    pub fn configure_external_trigger(&self, source: u8, edge: Trigen, filter: Filter) {
        self.set_trigger_source(source);
        self.set_trigger_filter(filter);
        self.set_trigger_mode(edge);
    }

    /// Get max compare value. This depends on the timer frequency and the clock frequency from RCC.
    pub fn get_max_compare_value(&self) -> u16 {
        T::regs().arr().read().arr()
    }
}

#[cfg(any(lptim_v2a, lptim_v2b, lptim_n6))]
impl<'d, T: Instance> Timer<'d, T> {
    /// Enable/disable a channel.
    pub fn enable_channel(&self, channel: Channel, enable: bool) {
        #[cfg(lptim_n6)]
        T::regs().ccmr1().modify(|w| {
            w.set_cce(channel.index(), enable);
        });
        #[cfg(not(lptim_n6))]
        T::regs().ccmr(0).modify(|w| {
            w.set_cce(channel.index(), enable);
        });
    }

    /// Get enable/disable state of a channel
    pub fn get_channel_enable_state(&self, channel: Channel) -> bool {
        #[cfg(lptim_n6)]
        {
            T::regs().ccmr1().read().cce(channel.index())
        }
        #[cfg(not(lptim_n6))]
        {
            T::regs().ccmr(0).read().cce(channel.index())
        }
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
    pub fn set_channel_direction(&self, channel: Channel, direction: ChannelDirection) {
        #[cfg(lptim_n6)]
        T::regs().ccmr1().modify(|w| {
            w.set_ccsel(channel.index(), direction.ccsel_bool());
        });
        #[cfg(not(lptim_n6))]
        T::regs().ccmr(0).modify(|w| {
            w.set_ccsel(channel.index(), direction.ccsel());
        });
    }

    /// Enable the timer interrupt.
    pub fn enable_interrupt(&self) {
        #[cfg(lptim_n6)]
        T::regs().dier_output().modify(|w| w.set_arrmie(true));
        #[cfg(not(lptim_n6))]
        T::regs().dier().modify(|w| w.set_arrmie(true));
    }

    /// Disable the timer interrupt.
    pub fn disable_interrupt(&self) {
        #[cfg(lptim_n6)]
        T::regs().dier_output().modify(|w| w.set_arrmie(false));
        #[cfg(not(lptim_n6))]
        T::regs().dier().modify(|w| w.set_arrmie(false));
    }

    /// Check if the timer interrupt is enabled.
    pub fn is_interrupt_enabled(&self) -> bool {
        #[cfg(lptim_n6)]
        {
            T::regs().dier_output().read().arrmie()
        }
        #[cfg(not(lptim_n6))]
        {
            T::regs().dier().read().arrmie()
        }
    }

    /// Check if the timer interrupt is pending.
    pub fn is_interrupt_pending(&self) -> bool {
        #[cfg(lptim_n6)]
        {
            T::regs().isr_output().read().arrm()
        }
        #[cfg(not(lptim_n6))]
        {
            T::regs().isr().read().arrm()
        }
    }

    /// Clear the timer interrupt.
    pub fn clear_interrupt(&self) {
        #[cfg(lptim_n6)]
        T::regs().icr_output().write(|w| w.set_arrmcf(true));
        #[cfg(not(lptim_n6))]
        T::regs().icr().write(|w| w.set_arrmcf(true));
    }
}

#[cfg(not(any(lptim_v2a, lptim_v2b, lptim_n6)))]
impl<'d, T: Instance> Timer<'d, T> {
    /// Set compare value for a channel.
    pub fn set_compare_value(&self, value: u16) {
        T::regs().cmp().modify(|w| w.set_cmp(value));
    }

    /// Get compare value for a channel.
    pub fn get_compare_value(&self) -> u16 {
        T::regs().cmp().read().cmp()
    }

    /// Enable the timer interrupt.
    pub fn enable_interrupt(&self) {
        T::regs().ier().modify(|w| w.set_arrmie(true));
    }

    /// Disable the timer interrupt.
    pub fn disable_interrupt(&self) {
        T::regs().ier().modify(|w| w.set_arrmie(false));
    }

    /// Check if the timer interrupt is enabled.
    pub fn is_interrupt_enabled(&self) -> bool {
        T::regs().ier().read().arrmie()
    }

    /// Check if the timer interrupt is pending.
    pub fn is_interrupt_pending(&self) -> bool {
        T::regs().isr().read().arrm()
    }

    /// Clear the timer interrupt.
    pub fn clear_interrupt(&self) {
        T::regs().icr().write(|w| w.set_arrmcf(true));
    }
}
