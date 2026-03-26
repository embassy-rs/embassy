use crate::time::Hertz;
#[cfg(dsihost_v1)]
use crate::time::Prescaler;

pub static mut DSI_CONFIG: Option<DsiHostPllConfig> = None;

/// DSI PLL Input Divisor of HSE clock
#[cfg(dsihost_v1)]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DsiPllInput {
    /// HSE
    Div1 = 1,
    /// HSE / 2
    Div2 = 2,
    /// HSE / 3
    Div3 = 3,
    /// HSE / 4
    Div4 = 4,
    /// HSE / 5
    Div5 = 5,
    /// HSE / 6
    Div6 = 6,
    /// HSE / 7
    Div7 = 7,
}

#[cfg(dsihost_v1)]
impl Prescaler for DsiPllInput {
    fn num(&self) -> u32 {
        1
    }

    fn denom(&self) -> u32 {
        *self as _
    }
}

/// DSI PLL Output Divisor of the VCO frequency
#[cfg(dsihost_v1)]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DsiPllOutput {
    /// VCO
    Div1 = 0,
    /// VCO / 2
    Div2 = 1,
    /// VCO / 4
    Div4 = 2,
    /// VCO / 8
    Div8 = 3,
}

#[cfg(dsihost_v1)]
impl Prescaler for DsiPllOutput {
    fn num(&self) -> u32 {
        1
    }

    fn denom(&self) -> u32 {
        match self {
            DsiPllOutput::Div1 => 1,
            DsiPllOutput::Div2 => 2,
            DsiPllOutput::Div4 => 4,
            DsiPllOutput::Div8 => 8,
        }
    }
}

/// DSI PLL Output divisor of the VCO clock from 1-511
#[cfg(dsihost_u5)]
pub type DsiPllOutput = u16;

/// DSI PLL Input divisor of the HSE clock from 1-511
#[cfg(dsihost_u5)]
pub type DsiPllInput = u16;

#[cfg(dsihost_v1)]
pub type DsiPllNdiv = u8;

#[cfg(dsihost_u5)]
pub type DsiPllNdiv = u16;

/// DSI PLL Configuration
#[derive(Clone, Copy)]
pub struct DsiHostPllConfig {
    /// Loop division factor
    pub(crate) ndiv: DsiPllNdiv,
    /// Input division factor
    pub(crate) idf: DsiPllInput,
    /// Output division factor
    pub(crate) odf: DsiPllOutput,
}

impl DsiHostPllConfig {
    /// Create a new DSI PLL configuration
    pub fn new(ndiv: DsiPllNdiv, idf: DsiPllInput, odf: DsiPllOutput) -> Self {
        #[cfg(dsihost_v1)]
        assert!(
            ndiv >= 10 && ndiv <= 125,
            "DSI PLL loop divisor must be 10 <= ndiv <= 125"
        );

        #[cfg(dsihost_u5)]
        assert!(
            ndiv >= 1 && ndiv <= 511,
            "DSI PLL loop divisor must be 1 <= ndiv <= 511"
        );

        Self { ndiv, idf, odf }
    }
}

/// Enable the DSIHOST PLL
pub fn configure_pll(hse: Option<Hertz>, config: DsiHostPllConfig) -> Hertz {
    let pll_vco = hse.expect("DSI requires configured HSE") * config.idf * config.ndiv * 2u32;

    #[cfg(feature = "defmt")]
    {
        debug!("DSI PLL VCO: {} MHz", pll_vco / Hertz::mhz(1));
    }

    assert!(
        pll_vco >= Hertz::mhz(1000) && pll_vco <= Hertz::mhz(2000),
        "DSI PLL VCO must be >= 1GHz and <= 2GHz"
    );

    #[cfg(dsihost_u5)]
    {
        assert!(config.idf <= 511, "DSI PLL input division factor must be <= 511");
        assert!(config.odf <= 511, "DSI PLL output division factor must be <= 511");
    }

    let pll_freq = pll_vco / 2u32 * config.odf;

    #[cfg(feature = "defmt")]
    {
        debug!("DSI PLL Output: {} MHz", pll_freq / Hertz::mhz(1));
    }

    assert!(
        pll_freq >= Hertz::khz(62_500) && pll_freq <= Hertz::mhz(1000),
        "DSI PLL output must be >= 62.5MHz and <= 1GHz"
    );

    unsafe { DSI_CONFIG = Some(config) };

    pll_freq
}
