use crate::pac::RCC;

mod pll;
pub use pll::PllConfig;

const HSI: u32 = 64_000_000; // Hz
const CSI: u32 = 4_000_000; // Hz
const HSI48: u32 = 48_000_000; // Hz
const LSI: u32 = 32_000; // Hz

/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Default)]
pub struct Config {
    pub hse: Option<u32>,
    pub bypass_hse: bool,
    pub sys_ck: Option<u32>,
    pub per_ck: Option<u32>,
    pub hclk: Option<u32>,
    pub pclk1: Option<u32>,
    pub pclk2: Option<u32>,
    pub pclk3: Option<u32>,
    pub pclk4: Option<u32>,
    pub pll1: PllConfig,
    pub pll2: PllConfig,
    pub pll3: PllConfig,
}
