pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Ppre as APBPrescaler, Sw as Sysclk};

/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Clone, Copy, Default)]
pub struct Config {}

pub(crate) unsafe fn init(_config: Config) {}
