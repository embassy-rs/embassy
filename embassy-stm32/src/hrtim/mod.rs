//! High Resolution Timer (HRTIM)

mod traits;

pub mod bridge_converter;
pub mod resonant_converter;

use core::mem::MaybeUninit;

use embassy_hal_internal::Peri;
use stm32_hrtim::control::{HrPwmControl, HrTimOngoingCalibration};
use stm32_hrtim::output::{Output1Pin, Output2Pin};
#[cfg(stm32g4)]
use stm32_hrtim::pac::HRTIM_TIMF;
use stm32_hrtim::pac::{HRTIM_TIMA, HRTIM_TIMB, HRTIM_TIMC, HRTIM_TIMD, HRTIM_TIME};
pub use stm32_hrtim::{self, Pscl1, Pscl2, Pscl4, Pscl8, Pscl16, Pscl32, Pscl64, Pscl128, PsclDefault};
use stm32_hrtim::{HrParts, HrPwmBuilder};
use traits::Instance;

use crate::gpio::{AfType, OutputType, SealedPin, Speed};
use crate::peripherals::{PA8, PA9, PA10, PA11, PB12, PB13, PB14, PB15, PC8, PC9};
#[cfg(stm32g4)]
use crate::peripherals::{PC6, PC7};
use crate::rcc;

/// Uninitialized HRTIM resources as returned by [HrControltExt::hr_control]
pub struct Parts {
    /// Control resources common for all of the HRTIM instances timers
    ///
    /// This needs to be initialized and calibrated by calling [HrTimOngoingCalibration::wait_for_calibration]
    pub control: HrTimOngoingCalibration,

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
        impl<PSCL, P1, P2> HrPwmBuilderExt<$TIMX, PSCL, P1, P2>
            for HrPwmBuilder<$TIMX, PSCL, stm32_hrtim::PreloadSource, P1, P2>
        where PSCL:
            stm32_hrtim::HrtimPrescaler,
            P1: Out1Pin<$TIMX>,
            P2: Out2Pin<$TIMX>
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
    ($TIMX:ty, $OutYPin:ident, $OutputYPin:ident, $HrOutY:ident, $CHY:ident<$CHY_AF:literal>) => {
        unsafe impl<'d> $OutputYPin<$TIMX> for Pin<Peri<'d, $CHY>> {}

        impl<'d> $OutYPin<$TIMX> for Pin<Peri<'d, $CHY>> {
            // Pin<Gpio, Index, Alternate<PushPull, AF>>
            fn connect_to_hrtim(self) {
                // It is quite important that we leave this up to the HRTIM driver. Doing this too
                // early might lead to undefined levels on the pin until the timer init is done.
                self.pin
                    .set_as_af($CHY_AF, AfType::output(OutputType::PushPull, self.speed));
            }
        }
    };
}

macro_rules! pins {
    ($($TIMX:ty: CH1: $CH1:ident<$CH1_AF:literal>, CH2: $CH2:ident<$CH2_AF:literal>),+) => {$(
        pins_helper!($TIMX, Out1Pin, Output1Pin, HrOut1, $CH1<$CH1_AF>);
        pins_helper!($TIMX, Out2Pin, Output2Pin, HrOut2, $CH2<$CH1_AF>);
    )+};
}

// TODO dont use stm32 pac types here. Impl some way to split the HRTIM1 instance into corresponding types maybe?
// or hr_control could return them all in a struct along with the calib thing?

#[cfg(stm32g4)]
pins! {
    HRTIM_TIMA: CH1: PA8<13>,  CH2: PA9<13>,
    HRTIM_TIMB: CH1: PA10<13>, CH2: PA11<13>,
    HRTIM_TIMC: CH1: PB12<13>, CH2: PB13<13>,
    HRTIM_TIMD: CH1: PB14<13>, CH2: PB15<13>,
    HRTIM_TIME: CH1: PC8<3>,   CH2: PC9<3>,
    HRTIM_TIMF: CH1: PC6<13>,  CH2: PC7<13>
}

#[cfg(stm32f3)]
pins! {
    HRTIM_TIMA: CH1: PA8<13>, CH2: PA9<13>,
    HRTIM_TIMB: CH1: PA10<13>, CH2: PA11<13>,
    HRTIM_TIMC: CH1: PB12<13>, CH2: PB13<13>,
    HRTIM_TIMD: CH1: PB14<13>, CH2: PB15<13>,
    HRTIM_TIME: CH1: PC8<3>, CH2: PC9<3>
}

/* // TODO: Figure out how to use these traits instead of hardcoded types
pins! {
    HRTIM_TIMA: CH1: ChannelAPin, CH2: ChannelAComplementaryPin,
    HRTIM_TIMB: CH1: ChannelBPin, CH2: ChannelBComplementaryPin,
    HRTIM_TIMC: CH1: ChannelCPin, CH2: ChannelCComplementaryPin,
    HRTIM_TIMD: CH1: ChannelDPin, CH2: ChannelDComplementaryPin,
    HRTIM_TIME: CH1: ChannelEPin, CH2: ChannelEComplementaryPin,
}

#[cfg(hrtim_v2)]
pins! {
    HRTIM_TIMF: CH1: ChannelFPin, CH2: ChannelFComplementaryPin
}
*/

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
