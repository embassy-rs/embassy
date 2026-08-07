//! AdvancedPwm

use embassy_hal_internal::Peri;

#[cfg(hrtim_v2)]
use super::ChF;
use super::low_level::HrTimer;
use super::{ChA, ChB, ChC, ChD, ChE, Instance};
use crate::hrtim::HRPin;

/// Struct used to divide a high resolution timer into multiple channels
pub struct AdvancedPwm<'d, T: Instance> {
    _inner: HrTimer<'d, T>,
}

impl<'d, T: Instance> AdvancedPwm<'d, T> {
    /// Create a new HRTIM driver.
    ///
    /// This splits the HRTIM into its constituent parts, which you can then use individually.
    pub fn new(
        tim: Peri<'d, T>,
        _cha: Option<if_afio!(HRPin<'d, T, ChA, A>)>,
        _chb: Option<if_afio!(HRPin<'d, T, ChB, A>)>,
        _chc: Option<if_afio!(HRPin<'d, T, ChC, A>)>,
        _chd: Option<if_afio!(HRPin<'d, T, ChD, A>)>,
        _che: Option<if_afio!(HRPin<'d, T, ChE, A>)>,
        #[cfg(hrtim_v2)] _chf: Option<if_afio!(HRPin<'d, T, ChF, A>)>,
    ) -> Self {
        Self::new_inner(tim)
    }

    fn new_inner(tim: Peri<'d, T>) -> Self {
        let mut tim = HrTimer::new(tim);

        #[cfg(stm32f334)]
        if crate::pac::RCC.cfgr3().read().hrtim1sw() == crate::pac::rcc::vals::Timsw::Pll1P {
            tim.calibrate();
        }

        #[cfg(not(stm32f334))]
        tim.calibrate();

        Self { _inner: tim }
    }
}
