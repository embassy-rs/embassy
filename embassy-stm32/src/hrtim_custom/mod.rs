use core::mem::MaybeUninit;

use embassy_hal_internal::Peri;
use stm32_hrtim::control::{HrPwmControl, HrTimOngoingCalibration};
use stm32_hrtim::output::{HrOut1, HrOut2, ToHrOut};
use stm32_hrtim::pac::{HRTIM_TIMA, HRTIM_TIMB, HRTIM_TIMC, HRTIM_TIMD, HRTIM_TIME, HRTIM_TIMF};
pub use stm32_hrtim::{self, Pscl1, Pscl2, Pscl4, Pscl8, Pscl16, Pscl32, Pscl64, Pscl128, PsclDefault};
use stm32_hrtim::{HrParts, HrPwmBuilder};

use crate::gpio::{AfType, OutputType, SealedPin, Speed};
use crate::peripherals::{PA8, PA9, PA10, PA11, PB12, PB13, PB14, PB15, PC6, PC7, PC8, PC9};
use crate::rcc;

pub struct Parts {
    pub control: HrTimOngoingCalibration,
    pub tima: HRTIM_TIMA,
    pub timb: HRTIM_TIMB,
    pub timc: HRTIM_TIMC,
    pub timd: HRTIM_TIMD,
    pub time: HRTIM_TIME,

    #[cfg(hrtim_v2)]
    pub timf: HRTIM_TIMF,
}

pub trait HrControltExt {
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

pub trait HrPwmBuilderExt<TIM, PSCL, PINS: ToHrOut<TIM>> {
    fn finalize(self, control: &mut HrPwmControl) -> HrParts<TIM, PSCL, PINS::Out<PSCL>>;
}
macro_rules! impl_finalize {
    ($($TIMX:ident),+) => {$(
        impl<PSCL: stm32_hrtim::HrtimPrescaler, PINS: HrtimPin<$TIMX>> HrPwmBuilderExt<$TIMX, PSCL, PINS>
            for HrPwmBuilder<$TIMX, PSCL, stm32_hrtim::PreloadSource, PINS>
        {
            fn finalize(
                self,
                control: &mut HrPwmControl,
            ) -> HrParts<$TIMX, PSCL, <PINS as ToHrOut<$TIMX>>::Out<PSCL>> {
                let pins = self._init(control);
                pins.connect_to_hrtim();
                unsafe { MaybeUninit::uninit().assume_init() }
            }
        }
    )+};
}

pub struct Pin<P> {
    pub speed: Speed,
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

pub trait HrtimPin<TIM>: ToHrOut<TIM> {
    fn connect_to_hrtim(self);
}

impl<TIM, PA, PB> HrtimPin<TIM> for (PA, PB)
where
    PA: HrtimPin<TIM>,
    PB: HrtimPin<TIM>,
{
    fn connect_to_hrtim(self) {
        self.0.connect_to_hrtim();
        self.1.connect_to_hrtim();
    }
}

macro_rules! pins_helper {
    ($TIMX:ty, $HrOutY:ident, $CHY:ident<$CHY_AF:literal>) => {
        unsafe impl<'d> ToHrOut<$TIMX> for Pin<Peri<'d, $CHY>> {
            type Out<PSCL> = $HrOutY<$TIMX, PSCL>;
        }

        impl<'d> HrtimPin<$TIMX> for Pin<Peri<'d, $CHY>> {
            // Pin<Gpio, Index, Alternate<PushPull, AF>>
            fn connect_to_hrtim(self) {
                self.pin
                    .set_as_af($CHY_AF, AfType::output(OutputType::PushPull, self.speed));
            }
        }
    };
}

macro_rules! pins {
    ($($TIMX:ty: CH1: $CH1:ident<$CH1_AF:literal>, CH2: $CH2:ident<$CH2_AF:literal>),+) => {$(
        pins_helper!($TIMX, HrOut1, $CH1<$CH1_AF>);
        pins_helper!($TIMX, HrOut2, $CH2<$CH1_AF>);
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
