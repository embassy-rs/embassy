use crate::rcc::sealed::RccPeripheral;
use crate::time::Hertz;

#[repr(u8)]
#[derive(Clone, Copy)]
pub(crate) enum Prescaler {
    Div1 = 1,
    Div2 = 2,
    Div4 = 4,
    Div8 = 8,
    Div16 = 16,
    Div32 = 32,
    Div64 = 64,
    Div128 = 128,
}

impl From<Prescaler> for u8 {
    fn from(val: Prescaler) -> Self {
        match val {
            Prescaler::Div1 => 0b000,
            Prescaler::Div2 => 0b001,
            Prescaler::Div4 => 0b010,
            Prescaler::Div8 => 0b011,
            Prescaler::Div16 => 0b100,
            Prescaler::Div32 => 0b101,
            Prescaler::Div64 => 0b110,
            Prescaler::Div128 => 0b111,
        }
    }
}

impl From<u8> for Prescaler {
    fn from(val: u8) -> Self {
        match val {
            0b000 => Prescaler::Div1,
            0b001 => Prescaler::Div2,
            0b010 => Prescaler::Div4,
            0b011 => Prescaler::Div8,
            0b100 => Prescaler::Div16,
            0b101 => Prescaler::Div32,
            0b110 => Prescaler::Div64,
            0b111 => Prescaler::Div128,
            _ => unreachable!(),
        }
    }
}

impl Prescaler {
    pub fn compute_min_high_res(val: u32) -> Self {
        *[
            Prescaler::Div1,
            Prescaler::Div2,
            Prescaler::Div4,
            Prescaler::Div8,
            Prescaler::Div16,
            Prescaler::Div32,
            Prescaler::Div64,
            Prescaler::Div128,
        ]
        .iter()
        .skip_while(|psc| **psc as u32 <= val)
        .next()
        .unwrap()
    }

    pub fn compute_min_low_res(val: u32) -> Self {
        *[Prescaler::Div32, Prescaler::Div64, Prescaler::Div128]
            .iter()
            .skip_while(|psc| **psc as u32 <= val)
            .next()
            .unwrap()
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance: RccPeripheral {
        fn regs() -> crate::pac::hrtim::Hrtim;

        fn set_master_frequency(frequency: Hertz) {
            let f = frequency.0;

            // TODO: wire up HRTIM to the RCC mux infra.
            //#[cfg(stm32f334)]
            //let timer_f = unsafe { crate::rcc::get_freqs() }.hrtim.unwrap_or(Self::frequency()).0;
            //#[cfg(not(stm32f334))]
            let timer_f = Self::frequency().0;

            let psc_min = (timer_f / f) / (u16::MAX as u32 / 32);
            let psc = if Self::regs().isr().read().dllrdy() {
                Prescaler::compute_min_high_res(psc_min)
            } else {
                Prescaler::compute_min_low_res(psc_min)
            };

            let timer_f = 32 * (timer_f / psc as u32);
            let per: u16 = (timer_f / f) as u16;

            let regs = Self::regs();

            regs.mcr().modify(|w| w.set_ckpsc(psc.into()));
            regs.mper().modify(|w| w.set_mper(per));
        }

        fn set_channel_frequency(channel: usize, frequency: Hertz) {
            let f = frequency.0;

            // TODO: wire up HRTIM to the RCC mux infra.
            //#[cfg(stm32f334)]
            //let timer_f = unsafe { crate::rcc::get_freqs() }.hrtim.unwrap_or(Self::frequency()).0;
            //#[cfg(not(stm32f334))]
            let timer_f = Self::frequency().0;

            let psc_min = (timer_f / f) / (u16::MAX as u32 / 32);
            let psc = if Self::regs().isr().read().dllrdy() {
                Prescaler::compute_min_high_res(psc_min)
            } else {
                Prescaler::compute_min_low_res(psc_min)
            };

            let timer_f = 32 * (timer_f / psc as u32);
            let per: u16 = (timer_f / f) as u16;

            let regs = Self::regs();

            regs.tim(channel).cr().modify(|w| w.set_ckpsc(psc.into()));
            regs.tim(channel).per().modify(|w| w.set_per(per));
        }

        /// Set the dead time as a proportion of max_duty
        fn set_channel_dead_time(channel: usize, dead_time: u16) {
            let regs = Self::regs();

            let channel_psc: Prescaler = regs.tim(channel).cr().read().ckpsc().into();

            // The dead-time base clock runs 4 times slower than the hrtim base clock
            // u9::MAX = 511
            let psc_min = (channel_psc as u32 * dead_time as u32) / (4 * 511);
            let psc = if Self::regs().isr().read().dllrdy() {
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
}

/// HRTIM instance trait.
pub trait Instance: sealed::Instance + 'static {}

foreach_interrupt! {
    ($inst:ident, hrtim, HRTIM, MASTER, $irq:ident) => {
        impl sealed::Instance for crate::peripherals::$inst {
            fn regs() -> crate::pac::hrtim::Hrtim {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {

        }
    };
}
