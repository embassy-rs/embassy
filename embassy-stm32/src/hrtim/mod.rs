//! High Resolution Timer (HRTIM)

use core::marker::PhantomData;

pub mod advanced_pwm;
pub mod bridge_converter;
pub mod resonant_converter;
pub use bridge_converter::BridgeConverter;
pub use resonant_converter::ResonantConverter;

use embassy_hal_internal::PeripheralType;

use crate::rcc::RccPeripheral;
use crate::time::Hertz;

/// HRTIM burst controller instance.
pub struct BurstController<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM master instance.
pub struct Master<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel A instance.
pub struct ChA<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel B instance.
pub struct ChB<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel C instance.
pub struct ChC<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel D instance.
pub struct ChD<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel E instance.
pub struct ChE<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel F instance.
#[cfg(hrtim_v2)]
pub struct ChF<T: Instance> {
    phantom: PhantomData<T>,
}

trait SealedAdvancedChannel<T: Instance> {
    fn raw() -> usize;
}

/// Advanced channel instance trait.
#[allow(private_bounds)]
pub trait AdvancedChannel<T: Instance>: SealedAdvancedChannel<T> {}

macro_rules! advanced_channel_impl {
    ($new_chx:ident, $new_chx_with_config:ident, $channel:tt, $ch_num:expr, $pin_trait:ident, $complementary_pin_trait:ident) => {
        impl<T: Instance> SealedAdvancedChannel<T> for $channel<T> {
            fn raw() -> usize {
                $ch_num
            }
        }
        impl<T: Instance> AdvancedChannel<T> for $channel<T> {}
    };
}

advanced_channel_impl!(
    new_cha,
    new_cha_with_config,
    ChA,
    0,
    ChannelAPin,
    ChannelAComplementaryPin
);
advanced_channel_impl!(
    new_chb,
    new_chb_with_config,
    ChB,
    1,
    ChannelBPin,
    ChannelBComplementaryPin
);
advanced_channel_impl!(
    new_chc,
    new_chc_with_config,
    ChC,
    2,
    ChannelCPin,
    ChannelCComplementaryPin
);
advanced_channel_impl!(
    new_chd,
    new_chd_with_config,
    ChD,
    3,
    ChannelDPin,
    ChannelDComplementaryPin
);
advanced_channel_impl!(
    new_che,
    new_che_with_config,
    ChE,
    4,
    ChannelEPin,
    ChannelEComplementaryPin
);
#[cfg(hrtim_v2)]
advanced_channel_impl!(
    new_chf,
    new_chf_with_config,
    ChF,
    5,
    ChannelFPin,
    ChannelFComplementaryPin
);

pin_trait!(ChannelAPin, Instance);
pin_trait!(ChannelAComplementaryPin, Instance);
pin_trait!(ChannelBPin, Instance);
pin_trait!(ChannelBComplementaryPin, Instance);
pin_trait!(ChannelCPin, Instance);
pin_trait!(ChannelCComplementaryPin, Instance);
pin_trait!(ChannelDPin, Instance);
pin_trait!(ChannelDComplementaryPin, Instance);
pin_trait!(ChannelEPin, Instance);
pin_trait!(ChannelEComplementaryPin, Instance);
#[cfg(hrtim_v2)]
pin_trait!(ChannelFPin, Instance);
#[cfg(hrtim_v2)]
pin_trait!(ChannelFComplementaryPin, Instance);

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

pub(crate) trait SealedInstance: RccPeripheral {
    fn regs() -> crate::pac::hrtim::Hrtim;

    #[allow(unused)]
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

/// HRTIM instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {}

foreach_interrupt! {
    ($inst:ident, hrtim, HRTIM, MASTER, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::hrtim::Hrtim {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {

        }
    };
}
