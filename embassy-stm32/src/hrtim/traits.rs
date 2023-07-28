use crate::rcc::sealed::RccPeripheral;
use crate::time::Hertz;

#[derive(Clone, Copy)]
pub(crate) enum Prescaler {
    Div1,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
}

impl From<Prescaler> for u32 {
    fn from(val: Prescaler) -> Self {
        match val {
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
        .skip_while(|psc| <Prescaler as Into<u32>>::into(**psc) <= val)
        .next()
        .unwrap()
    }

    pub fn compute_min_low_res(val: u32) -> Self {
        *[Prescaler::Div32, Prescaler::Div64, Prescaler::Div128]
            .iter()
            .skip_while(|psc| <Prescaler as Into<u32>>::into(**psc) <= val)
            .next()
            .unwrap()
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance: RccPeripheral {
        fn regs() -> crate::pac::hrtim::Hrtim;

        fn set_master_frequency(frequency: Hertz);

        fn set_channel_frequency(channnel: usize, frequency: Hertz);

        /// Set the dead time as a proportion of max_duty
        fn set_channel_dead_time(channnel: usize, dead_time: u16);

        //        fn enable_outputs(enable: bool);
        //
        //        fn enable_channel(&mut self, channel: usize, enable: bool);
    }
}

pub trait Instance: sealed::Instance + 'static {}

foreach_interrupt! {
    ($inst:ident, hrtim, HRTIM, MASTER, $irq:ident) => {
        impl sealed::Instance for crate::peripherals::$inst {
            fn regs() -> crate::pac::hrtim::Hrtim {
                crate::pac::$inst
            }

            fn set_master_frequency(frequency: Hertz) {
                use crate::rcc::sealed::RccPeripheral;

                let f = frequency.0;
                let timer_f = Self::frequency().0;
                let psc_min = (timer_f / f) / (u16::MAX as u32 / 32);
                let psc = if Self::regs().isr().read().dllrdy() {
                    Prescaler::compute_min_high_res(psc_min)
                } else {
                    Prescaler::compute_min_low_res(psc_min)
                };

                let psc_val: u32 = psc.into();
                let timer_f = 32 * (timer_f / psc_val);
                let per: u16 = (timer_f / f) as u16;

                let regs = Self::regs();

                regs.mcr().modify(|w| w.set_ckpsc(psc.into()));
                regs.mper().modify(|w| w.set_mper(per));
            }

            fn set_channel_frequency(channel: usize, frequency: Hertz) {
                use crate::rcc::sealed::RccPeripheral;

                let f = frequency.0;
                let timer_f = Self::frequency().0;
                let psc_min = (timer_f / f) / (u16::MAX as u32 / 32);
                let psc = if Self::regs().isr().read().dllrdy() {
                    Prescaler::compute_min_high_res(psc_min)
                } else {
                    Prescaler::compute_min_low_res(psc_min)
                };

                let psc_val: u32 = psc.into();
                let timer_f = 32 * (timer_f / psc_val);
                let per: u16 = (timer_f / f) as u16;

                let regs = Self::regs();

                regs.tim(channel).cr().modify(|w| w.set_ckpsc(psc.into()));
                regs.tim(channel).per().modify(|w| w.set_per(per));
            }

            fn set_channel_dead_time(channel: usize, dead_time: u16) {

                let regs = Self::regs();

                let channel_psc: Prescaler = regs.tim(channel).cr().read().ckpsc().into();
                let psc_val: u32 = channel_psc.into();


                // The dead-time base clock runs 4 times slower than the hrtim base clock
                // u9::MAX = 511
                let psc_min = (psc_val * dead_time as u32) / (4 * 511);
                let psc = if Self::regs().isr().read().dllrdy() {
                    Prescaler::compute_min_high_res(psc_min)
                } else {
                    Prescaler::compute_min_low_res(psc_min)
                };

                let dt_psc_val: u32 = psc.into();
                let dt_val = (dt_psc_val * dead_time as u32) / (4 * psc_val);

                regs.tim(channel).dt().modify(|w| {
                    w.set_dtprsc(psc.into());
                    w.set_dtf(dt_val as u16);
                    w.set_dtr(dt_val as u16);
                });
            }
        }

        impl Instance for crate::peripherals::$inst {

        }
    };
}
