//! Variable-frequency resonant converter driver.

use core::marker::PhantomData;

use super::{AdvancedChannel, Instance};
use crate::time::Hertz;

/// Variable-frequency resonant converter driver.
///
/// This implementation of a resonsant converter is appropriate for a half or full bridge,
/// but does not include secondary rectification, which is appropriate for applications
/// with a low-voltage on the secondary side.
pub struct ResonantConverter<T: Instance, C: AdvancedChannel<T>> {
    timer: PhantomData<T>,
    channel: PhantomData<C>,
    min_period: u16,
    max_period: u16,
}

impl<T: Instance, C: AdvancedChannel<T>> ResonantConverter<T, C> {
    /// Create a new variable-frequency resonant converter driver.
    pub fn new(_channel: C, min_frequency: Hertz, max_frequency: Hertz) -> Self {
        C::set_channel_frequency(C::index(), min_frequency);

        // Always enable preload
        T::regs().tim(C::index()).cr().modify(|w| {
            w.set_preen(true);
            w.set_repu(true);

            w.set_cont(true);
            w.set_half(true);
        });

        // Enable timer outputs
        T::regs().oenr().modify(|w| {
            w.set_t1oen(C::index(), true);
            w.set_t2oen(C::index(), true);
        });

        // Dead-time generator can be used in this case because the primary fets
        // of a resonant converter are always complementary
        T::regs().tim(C::index()).outr().modify(|w| w.set_dten(true));

        let max_period = T::regs().tim(C::index()).per().read().per();
        let min_period = max_period * (min_frequency.0 / max_frequency.0) as u16;

        Self {
            timer: PhantomData,
            channel: PhantomData,
            min_period: min_period,
            max_period: max_period,
        }
    }

    /// Set the dead time as a proportion of the maximum compare value
    pub fn set_dead_time(&mut self, value: u16) {
        C::set_channel_dead_time(C::index(), value);
    }

    /// Set the timer period.
    pub fn set_period(&mut self, period: u16) {
        assert!(period < self.max_period);
        assert!(period > self.min_period);

        T::regs().tim(C::index()).per().modify(|w| w.set_per(period));
    }

    /// Get the minimum compare value of a duty cycle
    pub fn get_min_period(&mut self) -> u16 {
        self.min_period
    }

    /// Get the maximum compare value of a duty cycle
    pub fn get_max_period(&mut self) -> u16 {
        self.max_period
    }
}
