use core::mem::MaybeUninit;

use ::stm32_hrtim::control::{HrPwmControl, HrTimOngoingCalibration};
use ::stm32_hrtim::output::{NoPin, Output1Pin, Output2Pin};
#[cfg(hrtim_v2)]
use ::stm32_hrtim::pac::HRTIM_TIMF;
use ::stm32_hrtim::pac::{HRTIM_MASTER, HRTIM_TIMA, HRTIM_TIMB, HRTIM_TIMC, HRTIM_TIMD, HRTIM_TIME};
use ::stm32_hrtim::{DacResetTrigger, DacStepTrigger, HrParts, HrPwmBuilder};
use embassy_hal_internal::Peri;

#[cfg(hrtim_v2)]
use crate::hrtim::ChF;
use crate::hrtim::{AfType, ChA, ChB, ChC, ChD, ChE, HRTIM1, HRTimerComplementaryPin, HRTimerPin, OutputType, Pin};
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

impl<T: crate::hrtim::Instance> HrControltExt for T {
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
            for HrPwmBuilder<$TIMX, PSCL, ::stm32_hrtim::PreloadSource, P1, P2, DacRst, DacStp>
        where PSCL:
            ::stm32_hrtim::HrtimPrescaler,
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

impl<T> Out1Pin<T> for NoPin {
    fn connect_to_hrtim(self) {}
}

impl<T> Out2Pin<T> for NoPin {
    fn connect_to_hrtim(self) {}
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
