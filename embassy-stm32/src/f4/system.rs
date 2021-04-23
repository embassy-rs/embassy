use crate::{hal::prelude::*, pac, Peripherals};

#[derive(Default)]
pub struct Config {
    pub use_hse: Option<u32>,
    pub sysclk: Option<u32>,
    pub pclk1: Option<u32>,
    pub require_pll48clk: bool,
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn use_hse(mut self, freq: u32) -> Self {
        self.use_hse = Some(freq);
        self
    }

    pub fn sysclk(mut self, freq: u32) -> Self {
        self.sysclk = Some(freq);
        self
    }

    pub fn pclk1(mut self, freq: u32) -> Self {
        self.pclk1 = Some(freq);
        self
    }

    pub fn require_pll48clk(mut self) -> Self {
        self.require_pll48clk = true;
        self
    }
}

/// safety: must only call once.
pub unsafe fn configure(config: Config) {
    let dp = pac::Peripherals::take().unwrap();
    let mut cfgr = dp.RCC.constrain().cfgr;

    if let Some(hz) = config.use_hse {
        cfgr = cfgr.use_hse(hz.mhz());
    };

    if let Some(hz) = config.sysclk {
        cfgr = cfgr.sysclk(hz.mhz());
    };

    if let Some(hz) = config.pclk1 {
        cfgr = cfgr.pclk1(hz.mhz());
    };

    if config.require_pll48clk {
        cfgr = cfgr.require_pll48clk();
    };

    let clocks = cfgr.freeze();

    unsafe { Peripherals::set_peripherals(clocks) };
}
