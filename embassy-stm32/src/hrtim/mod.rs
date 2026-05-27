//! High Resolution Timer (HRTIM)

use embassy_hal_internal::PeripheralType;

mod advanced_channel;
mod advanced_pwm;
mod bridge_converter;
mod fullbridge;
pub mod low_level;
mod resonant_converter;
#[cfg(feature = "stm32-hrtim")]
/// provides an adapter for the stm32-hrtim crate
pub mod stm32_hrtim;

use core::marker::PhantomData;

pub use advanced_channel::AdvancedChannel;
pub use advanced_pwm::AdvancedPwm;
pub use bridge_converter::BridgeConverter;
use embassy_hal_internal::Peri;
pub use fullbridge::FullBridgeConverter;
pub use resonant_converter::ResonantConverter;

use crate::gpio::{AfType, Flex, OutputType, Speed};
use crate::peripherals::HRTIM1;
use crate::rcc::RccPeripheral;
use crate::time::Hertz;
use crate::timer::simple_pwm::PwmPinConfig;

/// Timer channel.
#[derive(Clone, Copy)]
pub enum Channel {
    /// Channel A.
    ChA,
    /// Channel B.
    ChB,
    /// Channel C.
    ChC,
    /// Channel D.
    ChD,
    /// Channel E.
    ChE,
    #[cfg(hrtim_v2)]
    /// Channel F.
    ChF,
}

/// Burst Controller marker type.
pub enum BurstController {}

/// Master marker type.
pub enum Master {}

/// Channel A marker type.
pub enum ChA {}
/// Channel B marker type.
pub enum ChB {}
/// Channel C marker type.
pub enum ChC {}
/// Channel D marker type.
pub enum ChD {}

/// Channel E marker type.
pub enum ChE {}

#[cfg(hrtim_v2)]
/// Channel F marker type.
pub enum ChF {}

/// Timer channel trait.
#[allow(private_bounds)]
pub trait HRTimerChannel: SealedTimerChannel {
    /// The runtime channel.
    const CHANNEL: Channel;
}

trait SealedTimerChannel {}

impl HRTimerChannel for ChA {
    const CHANNEL: Channel = Channel::ChA;
}

impl HRTimerChannel for ChB {
    const CHANNEL: Channel = Channel::ChB;
}

impl HRTimerChannel for ChC {
    const CHANNEL: Channel = Channel::ChC;
}

impl HRTimerChannel for ChD {
    const CHANNEL: Channel = Channel::ChD;
}

impl HRTimerChannel for ChE {
    const CHANNEL: Channel = Channel::ChE;
}

#[cfg(hrtim_v2)]
impl HRTimerChannel for ChF {
    const CHANNEL: Channel = Channel::ChF;
}

impl SealedTimerChannel for ChA {}
impl SealedTimerChannel for ChB {}
impl SealedTimerChannel for ChC {}
impl SealedTimerChannel for ChD {}
impl SealedTimerChannel for ChE {}
#[cfg(hrtim_v2)]
impl SealedTimerChannel for ChF {}

/// PWM pin wrapper.
///
/// This wraps a pin to make it usable with PWM.
pub struct HRPin<'d, T, C, #[cfg(afio)] A> {
    #[allow(unused)]
    pub(crate) pin: Flex<'d>,
    phantom: PhantomData<if_afio!((T, C, A))>,
}

impl<'d, T: Instance, C: HRTimerChannel, #[cfg(afio)] A> if_afio!(HRPin<'d, T, C, A>) {
    /// Create a new PWM pin instance.
    pub fn new(pin: Peri<'d, if_afio!(impl HRTimerPin<T, C, A>)>, output_type: OutputType) -> Self {
        critical_section::with(|_| {
            pin.set_low();
            set_as_af!(pin, AfType::output(output_type, Speed::VeryHigh));
        });
        Self {
            pin: Flex::new(pin),
            phantom: PhantomData,
        }
    }

    /// Create a new PWM pin instance with a specific configuration.
    pub fn new_with_config(pin: Peri<'d, if_afio!(impl HRTimerPin<T, C, A>)>, pin_config: PwmPinConfig) -> Self {
        critical_section::with(|_| {
            pin.set_low();
            #[cfg(gpio_v1)]
            set_as_af!(pin, AfType::output(pin_config.output_type, pin_config.speed));
            #[cfg(gpio_v2)]
            set_as_af!(
                pin,
                AfType::output_pull(pin_config.output_type, pin_config.speed, pin_config.pull)
            );
        });

        Self {
            pin: Flex::new(pin),
            phantom: PhantomData,
        }
    }
}

/// PWM pin wrapper.
///
/// This wraps a pin to make it usable with PWM.
pub struct ComplementaryHRPin<'d, T, C, #[cfg(afio)] A> {
    #[allow(unused)]
    pub(crate) pin: Flex<'d>,
    phantom: PhantomData<if_afio!((T, C, A))>,
}

impl<'d, T: Instance, C: HRTimerChannel, #[cfg(afio)] A> if_afio!(ComplementaryHRPin<'d, T, C, A>) {
    /// Create a new PWM pin instance.
    pub fn new(pin: Peri<'d, if_afio!(impl HRTimerComplementaryPin<T, C, A>)>, output_type: OutputType) -> Self {
        critical_section::with(|_| {
            pin.set_low();
            set_as_af!(pin, AfType::output(output_type, Speed::VeryHigh));
        });
        Self {
            pin: Flex::new(pin),
            phantom: PhantomData,
        }
    }

    /// Create a new PWM pin instance with a specific configuration.
    pub fn new_with_config(
        pin: Peri<'d, if_afio!(impl HRTimerComplementaryPin<T, C, A>)>,
        pin_config: PwmPinConfig,
    ) -> Self {
        critical_section::with(|_| {
            pin.set_low();
            #[cfg(gpio_v1)]
            set_as_af!(pin, AfType::output(pin_config.output_type, pin_config.speed));
            #[cfg(gpio_v2)]
            set_as_af!(
                pin,
                AfType::output_pull(pin_config.output_type, pin_config.speed, pin_config.pull)
            );
        });

        Self {
            pin: Flex::new(pin),
            phantom: PhantomData,
        }
    }
}

/// Wrapper a pin that can be used as output to one of the HRTIM timer instances
pub struct Pin<P> {
    /// The speed setting of the pin
    pub speed: Speed,

    /// The pin
    pub pin: P,
}

pin_trait!(HRTimerPin, Instance, HRTimerChannel, @A);
pin_trait!(HRTimerComplementaryPin, Instance, HRTimerChannel, @A);

/// Prescaler
#[repr(u8)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Prescaler {
    /// Prescaler ratio 1
    Div1 = 1,
    /// Prescaler ratio 2
    Div2 = 2,
    /// Prescaler ratio 4
    Div4 = 4,
    /// Prescaler ratio 8
    Div8 = 8,
    /// Prescaler ratio 16
    Div16 = 16,
    /// Prescaler ratio 32
    Div32 = 32,
    /// Prescaler ratio 64
    Div64 = 64,
    /// Prescaler ratio 128
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
    /// Computer the minium prescaler for high resolution
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

    /// Compute the minium prescaler for low resolution
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
