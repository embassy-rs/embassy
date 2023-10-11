#![macro_use]

use core::mem::MaybeUninit;

use crate::time::Hertz;

mod bd;
mod mco;
pub use bd::*;
pub use mco::*;

#[cfg_attr(rcc_f0, path = "f0.rs")]
#[cfg_attr(any(rcc_f1, rcc_f100, rcc_f1cl), path = "f1.rs")]
#[cfg_attr(rcc_f2, path = "f2.rs")]
#[cfg_attr(any(rcc_f3, rcc_f3_v2), path = "f3.rs")]
#[cfg_attr(any(rcc_f4, rcc_f410), path = "f4.rs")]
#[cfg_attr(rcc_f7, path = "f7.rs")]
#[cfg_attr(rcc_c0, path = "c0.rs")]
#[cfg_attr(rcc_g0, path = "g0.rs")]
#[cfg_attr(rcc_g4, path = "g4.rs")]
#[cfg_attr(any(rcc_h5, rcc_h50, rcc_h7, rcc_h7rm0433, rcc_h7ab), path = "h.rs")]
#[cfg_attr(any(rcc_l0, rcc_l0_v2, rcc_l1), path = "l0l1.rs")]
#[cfg_attr(rcc_l4, path = "l4.rs")]
#[cfg_attr(rcc_l5, path = "l5.rs")]
#[cfg_attr(rcc_u5, path = "u5.rs")]
#[cfg_attr(rcc_wb, path = "wb.rs")]
#[cfg_attr(rcc_wba, path = "wba.rs")]
#[cfg_attr(any(rcc_wl5, rcc_wle), path = "wl.rs")]
mod _version;
#[cfg(feature = "low-power")]
use core::sync::atomic::{AtomicU32, Ordering};

pub use _version::*;

//  Model Clock Configuration
//
//  pub struct Clocks {
//      hse: Option<Hertz>,
//      hsi: bool,
//      lse: Option<Hertz>,
//      lsi: bool,
//      rtc: RtcSource,
//  }

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Clocks {
    pub sys: Hertz,

    // APB
    pub apb1: Hertz,
    pub apb1_tim: Hertz,
    #[cfg(not(any(rcc_c0, rcc_g0)))]
    pub apb2: Hertz,
    #[cfg(not(any(rcc_c0, rcc_g0)))]
    pub apb2_tim: Hertz,
    #[cfg(any(rcc_wl5, rcc_wle, rcc_h5, rcc_h50, rcc_h7, rcc_h7rm0433, rcc_h7ab, rcc_u5))]
    pub apb3: Hertz,
    #[cfg(any(rcc_h7, rcc_h7rm0433, rcc_h7ab))]
    pub apb4: Hertz,
    #[cfg(any(rcc_wba))]
    pub apb7: Hertz,

    // AHB
    pub ahb1: Hertz,
    #[cfg(any(
        rcc_l4,
        rcc_l5,
        rcc_f2,
        rcc_f4,
        rcc_f410,
        rcc_f7,
        rcc_h5,
        rcc_h50,
        rcc_h7,
        rcc_h7rm0433,
        rcc_h7ab,
        rcc_g4,
        rcc_u5,
        rcc_wb,
        rcc_wba,
        rcc_wl5,
        rcc_wle
    ))]
    pub ahb2: Hertz,
    #[cfg(any(
        rcc_l4,
        rcc_l5,
        rcc_f2,
        rcc_f4,
        rcc_f410,
        rcc_f7,
        rcc_h5,
        rcc_h50,
        rcc_h7,
        rcc_h7rm0433,
        rcc_h7ab,
        rcc_u5,
        rcc_wb,
        rcc_wl5,
        rcc_wle
    ))]
    pub ahb3: Hertz,
    #[cfg(any(rcc_h5, rcc_h50, rcc_h7, rcc_h7rm0433, rcc_h7ab, rcc_wba))]
    pub ahb4: Hertz,

    #[cfg(any(rcc_f2, rcc_f4, rcc_f410, rcc_f7))]
    pub pll48: Option<Hertz>,

    #[cfg(all(rcc_f4, not(stm32f410)))]
    pub plli2s: Option<Hertz>,

    #[cfg(any(stm32f427, stm32f429, stm32f437, stm32f439, stm32f446, stm32f469, stm32f479))]
    pub pllsai: Option<Hertz>,

    #[cfg(any(
        rcc_f1,
        rcc_f100,
        rcc_f1cl,
        rcc_h5,
        rcc_h50,
        rcc_h7,
        rcc_h7rm0433,
        rcc_h7ab,
        rcc_f3,
        rcc_g4
    ))]
    pub adc: Option<Hertz>,

    #[cfg(any(rcc_f3, rcc_g4))]
    pub adc34: Option<Hertz>,

    #[cfg(stm32f334)]
    pub hrtim: Option<Hertz>,

    pub rtc: Option<Hertz>,

    #[cfg(stm32h5)]
    pub mux_rcc_pclk1: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_pll2_q: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_pll3_q: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_hsi_ker: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_csi_ker: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_lse: Option<Hertz>,

    #[cfg(stm32h5)]
    pub mux_pll1_q: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_pll2_p: Option<Hertz>,
    #[cfg(rcc_h5)]
    pub mux_pll3_p: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_audioclk: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_per: Option<Hertz>,

    #[cfg(stm32h5)]
    pub mux_pll3_r: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_rcc_pclk3: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_pll3_1: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_hsi48_ker: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_lsi_ker: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_pll2_r: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_rcc_pclk2: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_rcc_pclk4: Option<Hertz>,
    #[cfg(stm32h5)]
    pub mux_hse: Option<Hertz>,

    #[cfg(stm32h5)]
    pub mux_hsi48: Option<Hertz>,
}

#[cfg(feature = "low-power")]
static CLOCK_REFCOUNT: AtomicU32 = AtomicU32::new(0);

#[cfg(feature = "low-power")]
pub fn low_power_ready() -> bool {
    // trace!("clock refcount: {}", CLOCK_REFCOUNT.load(Ordering::SeqCst));
    CLOCK_REFCOUNT.load(Ordering::SeqCst) == 0
}

#[cfg(feature = "low-power")]
pub(crate) fn clock_refcount_add(_cs: critical_section::CriticalSection) {
    // We don't check for overflow because constructing more than u32 peripherals is unlikely
    let n = CLOCK_REFCOUNT.load(Ordering::Relaxed);
    CLOCK_REFCOUNT.store(n + 1, Ordering::Relaxed);
}

#[cfg(feature = "low-power")]
pub(crate) fn clock_refcount_sub(_cs: critical_section::CriticalSection) {
    let n = CLOCK_REFCOUNT.load(Ordering::Relaxed);
    assert!(n != 0);
    CLOCK_REFCOUNT.store(n - 1, Ordering::Relaxed);
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
static mut CLOCK_FREQS: MaybeUninit<Clocks> = MaybeUninit::uninit();

/// Sets the clock frequencies
///
/// Safety: Sets a mutable global.
pub(crate) unsafe fn set_freqs(freqs: Clocks) {
    debug!("rcc: {:?}", freqs);
    CLOCK_FREQS = MaybeUninit::new(freqs);
}

/// Safety: Reads a mutable global.
pub(crate) unsafe fn get_freqs() -> &'static Clocks {
    CLOCK_FREQS.assume_init_ref()
}

#[cfg(feature = "unstable-pac")]
pub mod low_level {
    pub use super::sealed::*;
}

pub(crate) mod sealed {
    pub trait RccPeripheral {
        fn frequency() -> crate::time::Hertz;
        fn reset_and_enable();
        fn disable();
    }
}

pub trait RccPeripheral: sealed::RccPeripheral + 'static {}
