use core::marker::PhantomData;
use crate::hrtim::Instance;

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
        T::set_channel_frequency(C::raw(), frequency);

        // Always enable preload
        T::regs().tim(C::raw()).cr().modify(|w| {
            w.set_preen(true);
            w.set_repu(true);
            w.set_cont(true);
        });

        // Enable timer outputs
        T::regs().oenr().modify(|w| {
            w.set_t1oen(C::raw(), true);
            w.set_t2oen(C::raw(), true);
        });

        // The dead-time generation unit cannot be used because it forces the other output
        // to be completely complementary to the first output, which restricts certain waveforms
        // Therefore, software-implemented dead time must be used when setting the duty cycles

        // Set output 1 to active on a period event
        T::regs().tim(C::raw()).setr(0).modify(|w| w.set_per(true));

        // Set output 1 to inactive on a compare 1 event
        T::regs().tim(C::raw()).rstr(0).modify(|w| w.set_cmp(0, true));

        // Set output 2 to active on a compare 2 event
        T::regs().tim(C::raw()).setr(1).modify(|w| w.set_cmp(1, true));

        // Set output 2 to inactive on a compare 3 event
        T::regs().tim(C::raw()).rstr(1).modify(|w| w.set_cmp(2, true));

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
        T::regs().mcr().modify(|w| w.set_tcen(C::raw(), true));
    }

    /// Stop HRTIM.
    pub fn stop(&mut self) {
        T::regs().mcr().modify(|w| w.set_tcen(C::raw(), false));
    }

    /*
    /// Enable burst mode.
    pub fn enable_burst_mode(&mut self) {
        T::regs().tim(C::raw()).outr().modify(|w| {
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
        T::regs().tim(C::raw()).outr().modify(|w| {
            // Disable Burst Mode
            w.set_idlem(0, false);
            w.set_idlem(1, false);
        })
    }*/

    fn update_primary_duty_or_dead_time(&mut self) {
        self.min_secondary_duty = self.primary_duty + self.dead_time;

        T::regs().tim(C::raw()).cmp(0).modify(|w| w.set_cmp(self.primary_duty));
        T::regs()
            .tim(C::raw())
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
        T::regs().tim(C::raw()).per().read().per()
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

        T::regs().tim(C::raw()).cmp(2).modify(|w| w.set_cmp(secondary_duty));
    }
}

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
        T::set_channel_frequency(C::raw(), min_frequency);

        // Always enable preload
        T::regs().tim(C::raw()).cr().modify(|w| {
            w.set_preen(true);
            w.set_repu(true);

            w.set_cont(true);
            w.set_half(true);
        });

        // Enable timer outputs
        T::regs().oenr().modify(|w| {
            w.set_t1oen(C::raw(), true);
            w.set_t2oen(C::raw(), true);
        });

        // Dead-time generator can be used in this case because the primary fets
        // of a resonant converter are always complementary
        T::regs().tim(C::raw()).outr().modify(|w| w.set_dten(true));

        let max_period = T::regs().tim(C::raw()).per().read().per();
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
        T::set_channel_dead_time(C::raw(), value);
    }

    /// Set the timer period.
    pub fn set_period(&mut self, period: u16) {
        assert!(period < self.max_period);
        assert!(period > self.min_period);

        T::regs().tim(C::raw()).per().modify(|w| w.set_per(period));
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
