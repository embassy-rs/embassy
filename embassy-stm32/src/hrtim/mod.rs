//! High Resolution Timer (HRTIM)

use embassy_hal_internal::PeripheralType;

mod advanced_channel;
mod advanced_pwm;
mod bridge_converter;
mod fullbridge;
pub mod low_level;
mod resonant_converter;

use core::marker::PhantomData;
use core::mem::MaybeUninit;

pub use advanced_channel::AdvancedChannel;
pub use advanced_pwm::AdvancedPwm;
pub use bridge_converter::BridgeConverter;
use embassy_hal_internal::Peri;
pub use fullbridge::FullBridgeConverter;
pub use resonant_converter::ResonantConverter;
use stm32_hrtim::control::{HrPwmControl, HrTimOngoingCalibration};
use stm32_hrtim::output::{Output1Pin, Output2Pin};
#[cfg(hrtim_v2)]
use stm32_hrtim::pac::HRTIM_TIMF;
use stm32_hrtim::pac::{HRTIM_MASTER, HRTIM_TIMA, HRTIM_TIMB, HRTIM_TIMC, HRTIM_TIMD, HRTIM_TIME};
pub use stm32_hrtim::{self, Pscl1, Pscl2, Pscl4, Pscl8, Pscl16, Pscl32, Pscl64, Pscl128, PsclDefault};
use stm32_hrtim::{DacResetTrigger, DacStepTrigger, HrParts, HrPwmBuilder};

use crate::gpio::{AfType, Flex, OutputType, Speed};
use crate::peripherals::HRTIM1;
use crate::rcc;
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

/// Uninitialized HRTIM resources as returned by [HrControltExt::hr_control]
pub struct Parts {
    /// Control resources common for all of the HRTIM instances timers
    ///
    /// This needs to be initialized and calibrated by calling [HrTimOngoingCalibration::wait_for_calibration]
    pub control: HrTimOngoingCalibration,

    /// Uninitialized MASTER, call [HRTIM_MASTER::pwm_advanced] to set it up
    pub master: HRTIM_MASTER,

    /// Uninitialized TIMA, call [HRTIM_TIMA::pwm_advanced] to set it up
    pub tima: HRTIM_TIMA,

    /// Uninitialized TIMB, call [HRTIM_TIMB::pwm_advanced] to set it up
    pub timb: HRTIM_TIMB,

    /// Uninitialized TIMC, call [HRTIM_TIMC::pwm_advanced] to set it up
    pub timc: HRTIM_TIMC,

    /// Uninitialized TIMD, call [HRTIM_TIMD::pwm_advanced] to set it up
    pub timd: HRTIM_TIMD,

    /// Uninitialized TIME, call [HRTIM_TIME::pwm_advanced] to set it up
    pub time: HRTIM_TIME,

    /// Uninitialized TIMF, call [HRTIM_TIMF::pwm_advanced] to set it up
    #[cfg(hrtim_v2)]
    pub timf: HRTIM_TIMF,
}

/// Implemented for HRTIM peripheral block
pub trait HrControltExt {
    /// Setup HRTIM peripheral block
    fn hr_control(self) -> Parts;
}

impl<T: super::hrtim::Instance> HrControltExt for T {
    fn hr_control(self) -> Parts {
        rcc::enable_and_reset::<T>();

        // TODO: Verify that the HRTIM gets a clock of the correct speed as input
        // * 100-170MHz for g4
        // SAFETY:
        // * hr_control - We have enabled the rcc
        // * steal - We only steal these once
        unsafe {
            Parts {
                control: HrTimOngoingCalibration::hr_control(),
                master: HRTIM_MASTER::steal(),
                tima: HRTIM_TIMA::steal(),
                timb: HRTIM_TIMB::steal(),
                timc: HRTIM_TIMC::steal(),
                timd: HRTIM_TIMD::steal(),
                time: HRTIM_TIME::steal(),
                #[cfg(hrtim_v2)]
                timf: HRTIM_TIMF::steal(),
            }
        }
    }
}

/// Extension trait for initializing the HRTIM timer instance
pub trait HrPwmBuilderExt<TIM, PSCL, P1: Output1Pin<TIM>, P2: Output2Pin<TIM>> {
    /// Finalize the configuration and initialize the timer
    fn finalize(self, control: &mut HrPwmControl) -> HrParts<TIM, PSCL>;
}
macro_rules! impl_finalize {
    ($($TIMX:ident),+) => {$(
        impl<PSCL, P1, P2, DacRst, DacStp> HrPwmBuilderExt<$TIMX, PSCL, P1, P2>
            for HrPwmBuilder<$TIMX, PSCL, stm32_hrtim::PreloadSource, P1, P2, DacRst, DacStp>
        where PSCL:
            stm32_hrtim::HrtimPrescaler,
            P1: Out1Pin<$TIMX>,
            P2: Out2Pin<$TIMX>,
            DacRst: DacResetTrigger,
            DacStp: DacStepTrigger,
        {
            fn finalize(
                self,
                control: &mut HrPwmControl,
            ) -> HrParts<$TIMX, PSCL> {
                let (pin1, pin2) = self._init(control);
                pin1.connect_to_hrtim();
                pin2.connect_to_hrtim();
                unsafe { MaybeUninit::uninit().assume_init() }
            }
        }
    )+};
}

/// PWM pin wrapper.
///
/// This wraps a pin to make it usable with PWM.
pub struct PwmPin<'d, T, C, #[cfg(afio)] A> {
    #[allow(unused)]
    pub(crate) pin: Flex<'d>,
    phantom: PhantomData<if_afio!((T, C, A))>,
}

impl<'d, T: Instance, C: HRTimerChannel, #[cfg(afio)] A> if_afio!(PwmPin<'d, T, C, A>) {
    /// Create a new PWM pin instance.
    pub fn new(pin: Peri<'d, if_afio!(impl HRTimerPin<T, C, A>)>, output_type: OutputType) -> Self {
        critical_section::with(|_| {
            pin.set_low();
            set_as_af!(pin, AfType::output(output_type, Speed::VeryHigh));
        });
        PwmPin {
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
        PwmPin {
            pin: Flex::new(pin),
            phantom: PhantomData,
        }
    }
}

/// PWM pin wrapper.
///
/// This wraps a pin to make it usable with PWM.
pub struct ComplementaryPwmPin<'d, T, C, #[cfg(afio)] A> {
    #[allow(unused)]
    pub(crate) pin: Flex<'d>,
    phantom: PhantomData<if_afio!((T, C, A))>,
}

impl<'d, T: Instance, C: HRTimerChannel, #[cfg(afio)] A> if_afio!(ComplementaryPwmPin<'d, T, C, A>) {
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

impl_finalize! {
    HRTIM_TIMA,
    HRTIM_TIMB,
    HRTIM_TIMC,
    HRTIM_TIMD,
    HRTIM_TIME
}

#[cfg(hrtim_v2)]
impl_finalize! {
    HRTIM_TIMF
}

/// Implemented for types that can be used as output1 for HRTIM timer instances
pub trait Out1Pin<TIM>: Output1Pin<TIM> {
    /// Connect pin to hrtim timer
    fn connect_to_hrtim(self);
}

/// Implemented for types that can be used as output2 for HRTIM timer instances
pub trait Out2Pin<TIM>: Output2Pin<TIM> {
    /// Connect pin to hrtim timer
    fn connect_to_hrtim(self);
}

macro_rules! pins_helper {
    ($TIMX:ty, $ChannelXPin:ident<$Channel:ident>, $OutYPin:ident, $OutputYPin:ident, $HrOutY:ident) => {
        unsafe impl<'d, T: $ChannelXPin<HRTIM1, $Channel>> $OutputYPin<$TIMX> for Pin<Peri<'d, T>> {}
        impl<'d, T: $ChannelXPin<HRTIM1, $Channel>> $OutYPin<$TIMX> for Pin<Peri<'d, T>> {
            fn connect_to_hrtim(self) {
                self.pin.set_as_af(
                    self.pin.af_num(),
                    AfType::output(OutputType::PushPull, self.speed),
                );
            }
        }
    };
}

macro_rules! pins {
     ($($TIMX:ty: CH1: $CH1:ident<$CH1_AF:ident>, CH2: $CH2:ident<$CH2_AF:ident>),+) => {$(
         pins_helper!($TIMX, $CH1<$CH1_AF>, Out1Pin, Output1Pin, HrOut1);
         pins_helper!($TIMX, $CH2<$CH2_AF>, Out2Pin, Output2Pin, HrOut2);
     )+};
}

pins! {
    HRTIM_TIMA: CH1: HRTimerPin<ChA>, CH2: HRTimerComplementaryPin<ChA>,
    HRTIM_TIMB: CH1: HRTimerPin<ChB>, CH2: HRTimerComplementaryPin<ChB>,
    HRTIM_TIMC: CH1: HRTimerPin<ChC>, CH2: HRTimerComplementaryPin<ChC>,
    HRTIM_TIMD: CH1: HRTimerPin<ChD>, CH2: HRTimerComplementaryPin<ChD>,
    HRTIM_TIME: CH1: HRTimerPin<ChE>, CH2: HRTimerComplementaryPin<ChE>
}

#[cfg(hrtim_v2)]
pins! {
    HRTIM_TIMF: CH1: HRTimerPin<ChF>, CH2: HRTimerComplementaryPin<ChF>
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
