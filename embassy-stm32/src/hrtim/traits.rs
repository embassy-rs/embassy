use crate::time::Hertz;

#[derive(Clone, Copy)]
pub(crate) enum HighResolutionControlPrescaler {
    Div1,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
}

impl From<HighResolutionControlPrescaler> for u32 {
    fn from(val: HighResolutionControlPrescaler) -> Self {
        match val {
            HighResolutionControlPrescaler::Div1 => 1,
            HighResolutionControlPrescaler::Div2 => 2,
            HighResolutionControlPrescaler::Div4 => 4,
            HighResolutionControlPrescaler::Div8 => 8,
            HighResolutionControlPrescaler::Div16 => 16,
            HighResolutionControlPrescaler::Div32 => 32,
            HighResolutionControlPrescaler::Div64 => 64,
            HighResolutionControlPrescaler::Div128 => 128,
        }
    }
}

impl From<HighResolutionControlPrescaler> for u8 {
    fn from(val: HighResolutionControlPrescaler) -> Self {
        match val {
            HighResolutionControlPrescaler::Div1 => 0b000,
            HighResolutionControlPrescaler::Div2 => 0b001,
            HighResolutionControlPrescaler::Div4 => 0b010,
            HighResolutionControlPrescaler::Div8 => 0b011,
            HighResolutionControlPrescaler::Div16 => 0b100,
            HighResolutionControlPrescaler::Div32 => 0b101,
            HighResolutionControlPrescaler::Div64 => 0b110,
            HighResolutionControlPrescaler::Div128 => 0b111,
        }
    }
}

impl From<u8> for HighResolutionControlPrescaler {
    fn from(val: u8) -> Self {
        match val {
            0b000 => HighResolutionControlPrescaler::Div1,
            0b001 => HighResolutionControlPrescaler::Div2,
            0b010 => HighResolutionControlPrescaler::Div4,
            0b011 => HighResolutionControlPrescaler::Div8,
            0b100 => HighResolutionControlPrescaler::Div16,
            0b101 => HighResolutionControlPrescaler::Div32,
            0b110 => HighResolutionControlPrescaler::Div64,
            0b111 => HighResolutionControlPrescaler::Div128,
            _ => unreachable!(),
        }
    }
}

impl HighResolutionControlPrescaler {
    pub fn compute_min_high_res(val: u32) -> Self {
        *[
            HighResolutionControlPrescaler::Div1,
            HighResolutionControlPrescaler::Div2,
            HighResolutionControlPrescaler::Div4,
            HighResolutionControlPrescaler::Div8,
            HighResolutionControlPrescaler::Div16,
            HighResolutionControlPrescaler::Div32,
            HighResolutionControlPrescaler::Div64,
            HighResolutionControlPrescaler::Div128,
        ]
        .iter()
        .skip_while(|psc| <HighResolutionControlPrescaler as Into<u32>>::into(**psc) <= val)
        .next()
        .unwrap()
    }

    pub fn compute_min_low_res(val: u32) -> Self {
        *[
            HighResolutionControlPrescaler::Div32,
            HighResolutionControlPrescaler::Div64,
            HighResolutionControlPrescaler::Div128,
        ]
        .iter()
        .skip_while(|psc| <HighResolutionControlPrescaler as Into<u32>>::into(**psc) <= val)
        .next()
        .unwrap()
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait HighResolutionCaptureCompare16bitInstance: crate::timer::sealed::HighResolutionControlInstance {
        fn set_master_frequency(frequency: Hertz);

        fn set_channel_frequency(channnel: usize, frequency: Hertz);

        /// Set the dead time as a proportion of max_duty
        fn set_channel_dead_time(channnel: usize, dead_time: u16);

        //        fn enable_outputs(enable: bool);
        //
        //        fn enable_channel(&mut self, channel: usize, enable: bool);
    }
}

pub trait HighResolutionCaptureCompare16bitInstance:
    sealed::HighResolutionCaptureCompare16bitInstance + 'static
{
}

foreach_interrupt! {
    ($inst:ident, hrtim, HRTIM, MASTER, $irq:ident) => {
        impl sealed::HighResolutionCaptureCompare16bitInstance for crate::peripherals::$inst {
            fn set_master_frequency(frequency: Hertz) {
                use crate::rcc::sealed::RccPeripheral;
                use crate::timer::sealed::HighResolutionControlInstance;

                let f = frequency.0;
                let timer_f = Self::frequency().0;
                let psc_min = (timer_f / f) / (u16::MAX as u32 / 32);
                let psc = if Self::regs().isr().read().dllrdy() {
                    HighResolutionControlPrescaler::compute_min_high_res(psc_min)
                } else {
                    HighResolutionControlPrescaler::compute_min_low_res(psc_min)
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
                use crate::timer::sealed::HighResolutionControlInstance;

                let f = frequency.0;
                let timer_f = Self::frequency().0;
                let psc_min = (timer_f / f) / (u16::MAX as u32 / 32);
                let psc = if Self::regs().isr().read().dllrdy() {
                    HighResolutionControlPrescaler::compute_min_high_res(psc_min)
                } else {
                    HighResolutionControlPrescaler::compute_min_low_res(psc_min)
                };

                let psc_val: u32 = psc.into();
                let timer_f = 32 * (timer_f / psc_val);
                let per: u16 = (timer_f / f) as u16;

                let regs = Self::regs();

                regs.tim(channel).cr().modify(|w| w.set_ckpsc(psc.into()));
                regs.tim(channel).per().modify(|w| w.set_per(per));
            }

            fn set_channel_dead_time(channel: usize, dead_time: u16) {
                use crate::timer::sealed::HighResolutionControlInstance;

                let regs = Self::regs();

                let channel_psc: HighResolutionControlPrescaler = regs.tim(channel).cr().read().ckpsc().into();
                let psc_val: u32 = channel_psc.into();


                // The dead-time base clock runs 4 times slower than the hrtim base clock
                // u9::MAX = 511
                let psc_min = (psc_val * dead_time as u32) / (4 * 511);
                let psc = if Self::regs().isr().read().dllrdy() {
                    HighResolutionControlPrescaler::compute_min_high_res(psc_min)
                } else {
                    HighResolutionControlPrescaler::compute_min_low_res(psc_min)
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

        impl HighResolutionCaptureCompare16bitInstance for crate::peripherals::$inst {

        }
    };
}
