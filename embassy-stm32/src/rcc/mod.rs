use crate::time::Hertz;
use core::mem::MaybeUninit;

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy)]
pub struct Clocks {
    pub sys_clk: Hertz,
    pub ahb_clk: Hertz,
    pub apb1_clk: Hertz,
    pub apb1_tim_clk: Hertz,
    pub apb2_clk: Hertz,
    pub apb2_tim_clk: Hertz,
    pub apb1_pre: u8,
    pub apb2_pre: u8,
}

static mut CLOCK_FREQS: MaybeUninit<Clocks> = MaybeUninit::uninit();

/// Sets the clock frequencies
///
/// Safety: Sets a mutable global.
pub unsafe fn set_freqs(freqs: Clocks) {
    CLOCK_FREQS.as_mut_ptr().write(freqs);
}

/// Safety: Reads a mutable global.
pub unsafe fn get_freqs() -> &'static Clocks {
    &*CLOCK_FREQS.as_ptr()
}

cfg_if::cfg_if! {
    if #[cfg(feature = "_stm32h7")] {
        mod h7;
        pub use h7::*;
    } else if #[cfg(feature = "_stm32l0")] {
        mod l0;
        pub use l0::*;
    } else {
        #[derive(Default)]
        pub struct Config {}
        pub unsafe fn init(_config: Config) {
        }
    }
}
