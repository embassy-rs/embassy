//! DSIHOST Phase Locked Loop (PLL)

use crate::time::Hertz;
use crate::time::Prescaler;

use super::DSIHOST_WAKER;
use super::DsiHost;
use super::Instance;
use core::future::poll_fn;
use core::task::Poll;

/// DSI PLL Input Divisor of HSE clock
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

impl Prescaler for DsiPllInput {
    fn num(&self) -> u32 {
        1
    }

    fn denom(&self) -> u32 {
        *self as _
    }
}

/// DSI PLL Output Divisor of the VCO frequency
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

/// DSI PLL Configuration
pub struct DsiHostPllConfig {
    /// Loop division factor
    ndiv: u8,
    /// Input division factor
    idf: DsiPllInput,
    /// Output division factor
    odf: DsiPllOutput,
}

impl DsiHostPllConfig {
    /// Create a new DSI PLL configuration
    pub fn new(ndiv: u8, idf: DsiPllInput, odf: DsiPllOutput) -> Self {
        assert!(
            ndiv >= 10 && ndiv <= 125,
            "DSI PLL loop divisor must be 10 <= ndiv <= 125"
        );
        Self { ndiv, idf, odf }
    }
}

impl<'d, T: Instance> DsiHost<'d, T> {
    /// Enable the DSIHOST PLL
    pub async fn enable_pll(&mut self, config: &DsiHostPllConfig) {
        let pll_vco = self.hse_freq * config.idf * config.ndiv * 2u32;

        #[cfg(feature = "defmt")]
        {
            debug!("DSI PLL VCO: {} MHz", pll_vco / Hertz::mhz(1));
        }

        assert!(
            pll_vco >= Hertz::mhz(1000) && pll_vco <= Hertz::mhz(2000),
            "DSI PLL VCO must be >= 1GHz and <= 2GHz"
        );

        let pll_freq = pll_vco / 2u32 * config.odf;

        #[cfg(feature = "defmt")]
        {
            debug!("DSI PLL Output: {} MHz", pll_freq / Hertz::mhz(1));
        }

        assert!(
            pll_freq >= Hertz::khz(62_500) && pll_freq <= Hertz::mhz(1000),
            "DSI PLL output must be >= 62.5MHz and <= 1GHz"
        );

        self.phy_freq = Some(pll_freq).into();

        // Set the PLL configuration
        T::regs().wrpcr().modify(|w| {
            w.set_ndiv(config.ndiv);
            w.set_idf(config.idf as u8);
            w.set_odf(config.odf as u8);
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
