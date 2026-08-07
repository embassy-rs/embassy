//! Fixed-frequency bridge converter driver.

use core::marker::PhantomData;

use super::{AdvancedChannel, Instance, Prescaler};
use crate::time::Hertz;

/// Fixed-frequency full-bridge converter driver.
pub struct FullBridgeConverter<T: Instance, CH1: AdvancedChannel<T>, CH2: AdvancedChannel<T>> {
    timer: PhantomData<T>,
    ch1: PhantomData<CH1>,
    ch2: PhantomData<CH2>,
    dead_time: u16,
    duty: u16,
    minimum_duty: u16,
}

impl<T: Instance, CH1: AdvancedChannel<T>, CH2: AdvancedChannel<T>> FullBridgeConverter<T, CH1, CH2> {
    /// Create a new HRTIM bridge converter driver.
    pub fn new(_ch1: &mut CH1, _ch2: &mut CH2, frequency: Hertz) -> Self {
        CH1::set_channel_frequency(CH1::index(), frequency);
        CH2::set_channel_frequency(CH2::index(), frequency);

        T::regs().mcr().modify(|w| {
            w.set_preen(true);
            w.set_mrepu(true);
            w.set_cont(true);
        });

        // Always enable preload
        T::regs().tim(CH1::index()).cr().modify(|w| {
            w.set_preen(true);
            w.set_repu(true);
            w.set_cont(true);
        });

        // Always enable preload
        T::regs().tim(CH2::index()).cr().modify(|w| {
            w.set_preen(true);
            w.set_repu(true);
            w.set_cont(true);
        });

        // Enable timer outputs
        T::regs().oenr().modify(|w| {
            w.set_t1oen(CH1::index(), true);
            w.set_t2oen(CH1::index(), true);
            w.set_t1oen(CH2::index(), true);
            w.set_t2oen(CH2::index(), true);
        });

        // Set output 1 to active on a period event
        T::regs().tim(CH1::index()).setr(0).modify(|w| w.set_mstper(true));
        // Set output 1 to inactive on a cmp event
        T::regs().tim(CH1::index()).rstr(0).modify(|w| w.set_mstcmp(0, true));
        // Enable deadtime
        T::regs().tim(CH1::index()).outr().modify(|w| {
            w.set_dten(true);
            w.set_idles(0, false);
            w.set_idles(1, true);
        });

        // Set output 2 to inactive on a period event
        T::regs().tim(CH2::index()).rstr(0).modify(|w| w.set_mstper(true));
        // Set output 2 to inactive on a cmp event
        T::regs().tim(CH2::index()).setr(0).modify(|w| w.set_mstcmpx(0, true));
        // Enable deadtime
        T::regs().tim(CH2::index()).outr().modify(|w| {
            w.set_dten(true);
            w.set_idles(0, false);
            w.set_idles(1, true);
        });

        // Reset timing units on master period event
        T::regs().tim(CH1::index()).rst().modify(|w| w.set_mstper(true));
        T::regs().tim(CH2::index()).rst().modify(|w| w.set_mstper(true));

        Self {
            timer: PhantomData,
            ch1: PhantomData,
            ch2: PhantomData,
            dead_time: 0,
            duty: 0,
            minimum_duty: 0,
        }
    }

    /// Start HRTIM.
    pub fn start(&mut self) {
        T::regs().mcr().modify(|w| {
            w.set_mcen(true);
            w.set_tcen(CH1::index(), true);
            w.set_tcen(CH2::index(), true);
        });
    }

    /// Stop HRTIM.
    pub fn stop(&mut self) {
        T::regs().mcr().modify(|w| {
            w.set_mcen(false);
            w.set_tcen(CH1::index(), false);
            w.set_tcen(CH2::index(), false);
        });
    }

    /// Set duty
    fn set_duty_inner(&mut self) {
        T::regs().mcmp(0).modify(|w| w.set_mcmp(self.duty));
        T::regs().tim(CH1::index()).cmp(0).modify(|w| w.set_cmp(self.duty));

        T::regs().tim(CH2::index()).cmp(0).modify(|w| w.set_cmp(self.duty));
    }

    /// Get the maximum compare value of a duty cycle
    pub fn get_max_compare_value_master(&mut self) -> u16 {
        T::regs().mper().read().mper()
    }

    /// Get the maximum compare value of a duty cycle
    pub fn get_max_compare_value_ch1(&mut self) -> u16 {
        T::regs().tim(CH1::index()).per().read().per()
    }

    /// Get the maximum compare value of a duty cycle
    pub fn get_max_compare_value_ch2(&mut self) -> u16 {
        T::regs().tim(CH2::index()).per().read().per()
    }

    /// Set the dead time as a proportion of the maximum compare value
    pub fn set_dead_time(&mut self, value: u16) {
        self.dead_time = value;
        CH1::set_channel_dead_time(CH1::index(), value);
        CH2::set_channel_dead_time(CH2::index(), value);
    }

    /// The duty is the period in which the primary switch is active
    pub fn set_duty(&mut self, requested_duty: u16) {
        let fet_sound_duty = requested_duty
            .max(self.dead_time + self.minimum_duty)
            .min(self.get_max_compare_value_master() - (self.dead_time + self.minimum_duty));

        // TODO Use the Prescaler setting to compute the minimum
        let psc: Prescaler = T::regs().mcr().read().ckpsc().into();

        let fhrtim_3_clock_cycles = 0x60 / psc as u16;
        let hrtim_sound_duty = fet_sound_duty
            .max(fhrtim_3_clock_cycles)
            .min(self.get_max_compare_value_master() - fhrtim_3_clock_cycles);

        self.duty = hrtim_sound_duty;
        self.set_duty_inner();
    }

    /// The duty is the period in which the primary switch is active
    pub fn enable_outputs(&mut self) {
        // Enable timer outputs
        T::regs().oenr().modify(|w| {
            w.set_t1oen(CH1::index(), true);
            w.set_t2oen(CH1::index(), true);
            w.set_t1oen(CH2::index(), true);
            w.set_t2oen(CH2::index(), true);
        });
    }

    /// The duty is the period in which the primary switch is active
    pub fn disable_outputs(&mut self) {
        // Enable timer outputs
        T::regs().odisr().modify(|w| {
            w.set_t1odis(CH1::index(), true);
            w.set_t2odis(CH1::index(), true);
            w.set_t1odis(CH2::index(), true);
            w.set_t2odis(CH2::index(), true);
        });
    }

    /// Set minimum FET pulse width in counts
    pub fn set_minimum_duty(&mut self, duty: u16) {
        self.minimum_duty = duty;
    }
}
