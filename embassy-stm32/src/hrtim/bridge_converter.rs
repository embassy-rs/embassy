//! Fixed-frequency bridge converter driver.

use core::marker::PhantomData;

use super::{AdvancedChannel, Instance};
use crate::time::Hertz;

/// Fixed-frequency bridge converter driver.
///
/// Our implementation of the bridge converter uses a single channel and three compare registers,
/// allowing implementation of a synchronous buck or boost converter in continuous or discontinuous
/// conduction mode.
///
/// It is important to remember that in synchronous topologies, energy can flow in reverse during
/// light loading conditions, and that the low-side switch must be active for a short time to drive
/// a bootstrapped high-side switch.
pub struct BridgeConverter<T: Instance, C: AdvancedChannel<T>> {
    timer: PhantomData<T>,
    channel: PhantomData<C>,
    dead_time: u16,
    primary_duty: u16,
    min_secondary_duty: u16,
    max_secondary_duty: u16,
}

impl<T: Instance, C: AdvancedChannel<T>> BridgeConverter<T, C> {
    /// Create a new HRTIM bridge converter driver.
    pub fn new(_channel: C, frequency: Hertz) -> Self {
        C::set_channel_frequency(C::index(), frequency);

        // Always enable preload
        T::regs().tim(C::index()).cr().modify(|w| {
            w.set_preen(true);
            w.set_repu(true);
            w.set_cont(true);
        });

        // Enable timer outputs
        T::regs().oenr().modify(|w| {
            w.set_t1oen(C::index(), true);
            w.set_t2oen(C::index(), true);
        });

        // The dead-time generation unit cannot be used because it forces the other output
        // to be completely complementary to the first output, which restricts certain waveforms
        // Therefore, software-implemented dead time must be used when setting the duty cycles

        // Set output 1 to active on a period event
        T::regs().tim(C::index()).setr(0).modify(|w| w.set_per(true));

        // Set output 1 to inactive on a compare 1 event
        T::regs().tim(C::index()).rstr(0).modify(|w| w.set_cmp(0, true));

        // Set output 2 to active on a compare 2 event
        T::regs().tim(C::index()).setr(1).modify(|w| w.set_cmp(1, true));

        // Set output 2 to inactive on a compare 3 event
        T::regs().tim(C::index()).rstr(1).modify(|w| w.set_cmp(2, true));

        Self {
            timer: PhantomData,
            channel: PhantomData,
            dead_time: 0,
            primary_duty: 0,
            min_secondary_duty: 0,
            max_secondary_duty: 0,
        }
    }

    /// Start HRTIM.
    pub fn start(&mut self) {
        T::regs().mcr().modify(|w| w.set_tcen(C::index(), true));
    }

    /// Stop HRTIM.
    pub fn stop(&mut self) {
        T::regs().mcr().modify(|w| w.set_tcen(C::index(), false));
    }

    /// Enable burst mode.
    pub fn enable_burst_mode(&mut self) {
        T::regs().tim(C::index()).outr().modify(|w| {
            // Enable Burst Mode
            w.set_idlem(0, true);
            w.set_idlem(1, true);

            // Set output to active during the burst
            w.set_idles(0, true);
            w.set_idles(1, true);
        })
    }

    /// Disable burst mode.
    pub fn disable_burst_mode(&mut self) {
        T::regs().tim(C::index()).outr().modify(|w| {
            // Disable Burst Mode
            w.set_idlem(0, false);
            w.set_idlem(1, false);
        })
    }

    fn update_primary_duty_or_dead_time(&mut self) {
        self.min_secondary_duty = self.primary_duty + self.dead_time;

        T::regs()
            .tim(C::index())
            .cmp(0)
            .modify(|w| w.set_cmp(self.primary_duty));
        T::regs()
            .tim(C::index())
            .cmp(1)
            .modify(|w| w.set_cmp(self.min_secondary_duty));
    }

    /// Set the dead time as a proportion of the maximum compare value
    pub fn set_dead_time(&mut self, dead_time: u16) {
        self.dead_time = dead_time;
        self.max_secondary_duty = self.get_max_compare_value() - dead_time;
        self.update_primary_duty_or_dead_time();
    }

    /// Get the maximum compare value of a duty cycle
    pub fn get_max_compare_value(&mut self) -> u16 {
        T::regs().tim(C::index()).per().read().per()
    }

    /// The primary duty is the period in which the primary switch is active
    ///
    /// In the case of a buck converter, this is the high-side switch
    /// In the case of a boost converter, this is the low-side switch
    pub fn set_primary_duty(&mut self, primary_duty: u16) {
        self.primary_duty = primary_duty;
        self.update_primary_duty_or_dead_time();
    }

    /// The secondary duty is the period in any switch is active
    ///
    /// If less than or equal to the primary duty, the secondary switch will be active for one tick
    /// If a fully complementary output is desired, the secondary duty can be set to the max compare
    pub fn set_secondary_duty(&mut self, secondary_duty: u16) {
        let secondary_duty = if secondary_duty > self.max_secondary_duty {
            self.max_secondary_duty
        } else if secondary_duty <= self.min_secondary_duty {
            self.min_secondary_duty + 1
        } else {
            secondary_duty
        };

        T::regs().tim(C::index()).cmp(2).modify(|w| w.set_cmp(secondary_duty));
    }
}
