pub(crate) const HSE_OSC_MIN: u32 = 4_000_000;
pub(crate) const HSE_OSC_MAX: u32 = 26_000_000;
pub(crate) const HSE_BYPASS_MIN: u32 = 1_000_000;
pub(crate) const HSE_BYPASS_MAX: u32 = 50_000_000;

pub(crate) const HCLK_MAX: u32 = 216_000_000;
pub(crate) const HCLK_OVERDRIVE_FREQUENCY: u32 = 180_000_000;

pub(crate) const SYSCLK_MIN: u32 = 12_500_000;
pub(crate) const SYSCLK_MAX: u32 = 216_000_000;

pub(crate) const PCLK1_MIN: u32 = SYSCLK_MIN;
pub(crate) const PCLK1_MAX: u32 = SYSCLK_MAX / 4;

pub(crate) const PCLK2_MIN: u32 = SYSCLK_MIN;
pub(crate) const PCLK2_MAX: u32 = SYSCLK_MAX / 2;

pub(crate) const PLL_48_CLK: u32 = 48_000_000;
pub(crate) const PLL_48_TOLERANCE: u32 = 120_000;
