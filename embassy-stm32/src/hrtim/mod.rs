//! High Resolution Timer (HRTIM)

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;

mod advanced_channel;
mod advanced_pwm;
mod bridge_converter;
mod fullbridge;
pub mod low_level;
mod resonant_converter;

use core::mem::MaybeUninit;

pub use advanced_channel::AdvancedChannel;
pub use advanced_pwm::{AdvancedPwm, ComplementaryPwmPin, PwmPin};
pub use bridge_converter::BridgeConverter;
use embassy_hal_internal::Peri;
pub use fullbridge::FullBridgeConverter;
pub use resonant_converter::ResonantConverter;
use stm32_hrtim::control::{HrPwmControl, HrTimOngoingCalibration};
use stm32_hrtim::output::{Output1Pin, Output2Pin};

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
use stm32_hrtim::pac::HRTIM_TIMF;
use stm32_hrtim::pac::{HRTIM_MASTER, HRTIM_TIMA, HRTIM_TIMB, HRTIM_TIMC, HRTIM_TIMD, HRTIM_TIME};
pub use stm32_hrtim::{self, Pscl1, Pscl2, Pscl4, Pscl8, Pscl16, Pscl32, Pscl64, Pscl128, PsclDefault};
use stm32_hrtim::{DacResetTrigger, DacStepTrigger, HrParts, HrPwmBuilder};

use crate::gpio::{AfType, OutputType, Speed};
use crate::peripherals::HRTIM1;
use crate::rcc;

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
    ($TIMX:ty, $ChannelXPin:ident, $OutYPin:ident, $OutputYPin:ident, $HrOutY:ident) => {
        unsafe impl<'d, T: $ChannelXPin<HRTIM1>> $OutputYPin<$TIMX> for Pin<Peri<'d, T>> {}
        impl<'d, T: $ChannelXPin<HRTIM1>> $OutYPin<$TIMX> for Pin<Peri<'d, T>> {
            fn connect_to_hrtim(self) {
                self.pin.set_as_af(
                    self.pin.af_num(),
                    AfType::output(OutputType::PushPull, self.speed),
                );
            }
        }
    };
}

pins_helper!(HRTIM_TIMA, ChannelAPin, Out1Pin, Output1Pin, HrOut1);
pins_helper!(HRTIM_TIMA, ChannelAComplementaryPin, Out2Pin, Output2Pin, HrOut2);
pins_helper!(HRTIM_TIMB, ChannelBPin, Out1Pin, Output1Pin, HrOut1);
pins_helper!(HRTIM_TIMB, ChannelBComplementaryPin, Out2Pin, Output2Pin, HrOut2);
pins_helper!(HRTIM_TIMC, ChannelCPin, Out1Pin, Output1Pin, HrOut1);
pins_helper!(HRTIM_TIMC, ChannelCComplementaryPin, Out2Pin, Output2Pin, HrOut2);
pins_helper!(HRTIM_TIMD, ChannelDPin, Out1Pin, Output1Pin, HrOut1);
pins_helper!(HRTIM_TIMD, ChannelDComplementaryPin, Out2Pin, Output2Pin, HrOut2);
pins_helper!(HRTIM_TIME, ChannelEPin, Out1Pin, Output1Pin, HrOut1);
pins_helper!(HRTIM_TIME, ChannelEComplementaryPin, Out2Pin, Output2Pin, HrOut2);

#[cfg(stm32g4)]
pins_helper!(HRTIM_TIMF, ChannelFPin, Out1Pin, Output1Pin, HrOut1);
#[cfg(stm32g4)]
pins_helper!(HRTIM_TIMF, ChannelFComplementaryPin, Out2Pin, Output2Pin, HrOut2);

// macro_rules! pins {
//     ($($TIMX:ty: CH1: $CH1:ident<$CH1_AF:literal>, CH2: $CH2:ident<$CH2_AF:literal>),+) => {$(
//         pins_helper!($TIMX, Out1Pin, Output1Pin, HrOut1, $CH1<$CH1_AF>);
//         pins_helper!($TIMX, Out2Pin, Output2Pin, HrOut2, $CH2<$CH1_AF>);
//     )+};
// }

// #[cfg(stm32g4)]
// pins! {
//     HRTIM_TIMA: CH1: PA8<13>,  CH2: PA9<13>,
//     HRTIM_TIMB: CH1: PA10<13>, CH2: PA11<13>,
//     HRTIM_TIMC: CH1: PB12<13>, CH2: PB13<13>,
//     HRTIM_TIMD: CH1: PB14<13>, CH2: PB15<13>,
//     HRTIM_TIME: CH1: PC8<3>,   CH2: PC9<3>,
//     HRTIM_TIMF: CH1: PC6<13>,  CH2: PC7<13>
// }

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
