//! DSIHOST Phase Locked Loop (PLL)

use crate::rcc::get_freqs;
use crate::rcc::set_freqs;
use crate::time::Hertz;
#[cfg(dsihost_v1)]
use crate::time::Prescaler;

use super::DSIHOST_WAKER;
use super::DsiHost;
use super::Instance;
use core::future::poll_fn;
use core::task::Poll;

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
pub struct DsiHostPllConfig {
    /// Loop division factor
    ndiv: DsiPllNdiv,
    /// Input division factor
    idf: DsiPllInput,
    /// Output division factor
    odf: DsiPllOutput,
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

impl<'d, T: Instance> DsiHost<'d, T> {
    /// Enable the DSIHOST PLL
    pub async fn enable_pll(&mut self, config: &DsiHostPllConfig) {
        let mut clocks = unsafe { *get_freqs() };

        let pll_vco = clocks.hse.to_hertz().expect("DSI requires configured HSE") * config.idf * config.ndiv * 2u32;

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

        // Set the DSI PHY clock frequency in Clocks
        unsafe {
            clocks.dsi_phy = Some(pll_freq).into();
            set_freqs(clocks);
        };

        // Set the PLL configuration
        T::regs().wrpcr().modify(|w| {
            w.set_ndiv(config.ndiv);

            #[cfg(dsihost_v1)]
            {
                w.set_idf(config.idf as u8);
                w.set_odf(config.odf as u8);
            }

            #[cfg(dsihost_u5)]
            {
                w.set_idf(config.idf);
                w.set_odf(config.odf);
            }
        });

        Self::enable_wait_pll_lock().await;
    }

    /// Enable the PLL and wait for the lock interrupt
    async fn enable_wait_pll_lock() {
        poll_fn(|cx| {
            let status = T::regs().wisr().read();

            if status.plllif() || status.pllls() {
                T::regs().wifcr().modify(|w| w.set_cplllif(true));
                Poll::Ready(())
            } else {
                DSIHOST_WAKER.register(cx.waker());

                T::regs().wifcr().modify(|w| w.set_cplllif(true));
                T::regs().wifcr().modify(|w| w.set_cplluif(true));
                T::regs().wier().modify(|w| w.set_plllie(true));
                Self::enable_interrupts(true);

                // Set the PLL enable bit and wait for lock
                T::regs().wrpcr().modify(|w| w.set_pllen(true));

                Poll::Pending
            }
        })
        .await;
    }
}
