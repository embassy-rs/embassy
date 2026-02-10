//! High Resolution Timer (HRTIM)

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;

mod advanced_channel;
mod advanced_pwm;
mod bridge_converter;
mod resonant_converter;

pub use advanced_channel::AdvancedChannel;
pub use advanced_pwm::{AdvancedPwm, ComplementaryPwmPin, PwmPin};
pub use bridge_converter::BridgeConverter;
pub use resonant_converter::ResonantConverter;

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
