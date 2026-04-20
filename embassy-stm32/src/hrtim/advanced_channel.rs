//! AdvancedChannel

use super::{Instance, Prescaler};
use crate::time::Hertz;

trait SealedAdvancedChannel<T: Instance> {
    fn raw() -> usize;
}

/// Advanced channel instance trait.
#[allow(private_bounds)]
pub trait AdvancedChannel<T: Instance>: SealedAdvancedChannel<T> {
    /// Channel index
    fn index() -> usize {
        Self::raw()
    }

    /// Set channel prescaler
    fn set_channel_prescaler(channel: usize, ckpsc: Prescaler) {
        T::regs().tim(channel).cr().modify(|w| w.set_ckpsc(ckpsc.into()))
    }

    /// Set channel period
    fn set_channel_period(channel: usize, per: u16) {
        T::regs().tim(channel).per().modify(|w| w.set_per(per));
    }

    /// Set channel frequency
    fn set_channel_frequency(channel: usize, frequency: Hertz) {
        let f = frequency.0;

        // TODO: wire up HRTIM to the RCC mux infra.
        //#[cfg(stm32f334)]
        //let timer_f = unsafe { crate::rcc::get_freqs() }.hrtim.unwrap_or(T::frequency()).0;
        //#[cfg(not(stm32f334))]
        let timer_f = T::frequency().0;

        let psc_min = (timer_f / f) / (u16::MAX as u32 / 32);
        let psc = if T::regs().isr().read().dllrdy() {
            Prescaler::compute_min_high_res(psc_min)
        } else {
            Prescaler::compute_min_low_res(psc_min)
        };

        let timer_f = 32 * (timer_f as u64 / psc as u64);
        let per: u16 = (timer_f / f as u64) as u16;

        Self::set_channel_prescaler(channel, psc);
        Self::set_channel_period(channel, per);
    }

    /// Set the dead time as a proportion of max_duty
    fn set_channel_dead_time(channel: usize, dead_time: u16) {
        let regs = T::regs();

        let channel_psc: Prescaler = regs.tim(channel).cr().read().ckpsc().into();

        // The dead-time base clock runs 4 times slower than the hrtim base clock
        // u9::MAX = 511
        let psc_min = (channel_psc as u32 * dead_time as u32) / (4 * 511);
        let psc = if T::regs().isr().read().dllrdy() {
            Prescaler::compute_min_high_res(psc_min)
        } else {
            Prescaler::compute_min_low_res(psc_min)
        };

        let dt_val = (psc as u32 * dead_time as u32) / (4 * channel_psc as u32);

        regs.tim(channel).dt().modify(|w| {
            w.set_dtprsc(psc.into());
            w.set_dtf(dt_val as u16);
            w.set_dtr(dt_val as u16);
        });
    }
}

#[allow(unused)]
trait SealedAdvancedChannelMaster<T: Instance> {}

/// Advanced channel instance trait.
#[allow(private_bounds)]
#[allow(unused)]
pub trait AdvancedChannelMaster<T: Instance>: SealedAdvancedChannelMaster<T> {
    /// Set master frequency
    fn set_master_frequency(&mut self, frequency: Hertz) {
        let f = frequency.0;

        // TODO: wire up HRTIM to the RCC mux infra.
        //#[cfg(stm32f334)]
        //let timer_f = unsafe { crate::rcc::get_freqs() }.hrtim.unwrap_or(T::frequency()).0;
        //#[cfg(not(stm32f334))]
        let timer_f = T::frequency().0;

        let psc_min = (timer_f / f) / (u16::MAX as u32 / 32);
        let psc = if T::regs().isr().read().dllrdy() {
            Prescaler::compute_min_high_res(psc_min)
        } else {
            Prescaler::compute_min_low_res(psc_min)
        };

        let timer_f = 32 * (timer_f as u64 / psc as u64);
        let per: u16 = (timer_f / f as u64) as u16;

        let regs = T::regs();

        regs.mcr().modify(|w| w.set_ckpsc(psc.into()));
        regs.mper().modify(|w| w.set_mper(per));
    }
}
