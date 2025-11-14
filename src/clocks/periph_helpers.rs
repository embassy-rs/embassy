use super::{ClockError, Clocks, Div8, PoweredClock};
use crate::pac;

pub trait SPConfHelper {
    fn post_enable_config(&self, clocks: &Clocks) -> Result<u32, ClockError>;
}

// config types

#[derive(Debug, Clone, Copy)]
pub enum LpuartClockSel {
    /// FRO12M/FRO_LF/SIRC clock source, passed through divider
    /// "fro_lf_div"
    FroLfDiv,
    /// FRO180M/FRO_HF/FIRC clock source, passed through divider
    /// "fro_hf_div"
    FroHfDiv,
    /// SOSC/XTAL/EXTAL clock source
    ClkIn,
    /// FRO16K/clk_16k source
    Clk16K,
    /// clk_1m/FRO_LF divided by 12
    Clk1M,
    /// Output of PLL1, passed through clock divider,
    /// "pll1_clk_div", maybe "pll1_lf_div"?
    Pll1ClkDiv,
    /// Disabled
    None,
}

pub enum LpuartInstance {
    Lpuart0,
    Lpuart1,
    Lpuart2,
    Lpuart3,
    Lpuart4,
    Lpuart5,
}

pub struct LpuartConfig {
    /// Power state required for this peripheral
    pub power: PoweredClock,
    /// Clock source
    pub source: LpuartClockSel,
    /// Clock divisor
    pub div: Div8,
    /// Which instance is this?
    // NOTE: should not be user settable
    pub(crate) instance: LpuartInstance,
}

// impls

impl SPConfHelper for LpuartConfig {
    fn post_enable_config(&self, clocks: &Clocks) -> Result<u32, ClockError> {
        // check that source is suitable
        let mrcc0 = unsafe { pac::Mrcc0::steal() };
        use mcxa_pac::mrcc0::mrcc_lpuart0_clksel::Mux;

        let (clkdiv, clksel) = match self.instance {
            LpuartInstance::Lpuart0 => (mrcc0.mrcc_lpuart0_clkdiv(), mrcc0.mrcc_lpuart0_clksel()),
            LpuartInstance::Lpuart1 => (mrcc0.mrcc_lpuart1_clkdiv(), mrcc0.mrcc_lpuart1_clksel()),
            LpuartInstance::Lpuart2 => (mrcc0.mrcc_lpuart2_clkdiv(), mrcc0.mrcc_lpuart2_clksel()),
            LpuartInstance::Lpuart3 => (mrcc0.mrcc_lpuart3_clkdiv(), mrcc0.mrcc_lpuart3_clksel()),
            LpuartInstance::Lpuart4 => (mrcc0.mrcc_lpuart4_clkdiv(), mrcc0.mrcc_lpuart4_clksel()),
            LpuartInstance::Lpuart5 => (mrcc0.mrcc_lpuart5_clkdiv(), mrcc0.mrcc_lpuart5_clksel()),
        };

        let (freq, variant) = match self.source {
            LpuartClockSel::FroLfDiv => {
                let freq = clocks.ensure_fro_lf_div_active(&self.power)?;
                (freq, Mux::ClkrootFunc0)
            }
            LpuartClockSel::FroHfDiv => {
                let freq = clocks.ensure_fro_hf_div_active(&self.power)?;
                (freq, Mux::ClkrootFunc2)
            }
            LpuartClockSel::ClkIn => {
                let freq = clocks.ensure_clk_in_active(&self.power)?;
                (freq, Mux::ClkrootFunc3)
            }
            LpuartClockSel::Clk16K => {
                let freq = clocks.ensure_clk_16k_active(&self.power)?;
                (freq, Mux::ClkrootFunc4)
            }
            LpuartClockSel::Clk1M => {
                let freq = clocks.ensure_clk_1m_active(&self.power)?;
                (freq, Mux::ClkrootFunc5)
            }
            LpuartClockSel::Pll1ClkDiv => {
                let freq = clocks.ensure_pll1_clk_div_active(&self.power)?;
                (freq, Mux::ClkrootFunc6)
            }
            LpuartClockSel::None => unsafe {
                // no ClkrootFunc7, just write manually for now
                clksel.write(|w| w.bits(0b111));
                clkdiv.modify(|_r, w| {
                    w.reset().on();
                    w.halt().on();
                    w
                });
                return Ok(0);
            },
        };

        // set clksel
        clksel.modify(|_r, w| w.mux().variant(variant));

        // Set up clkdiv
        clkdiv.modify(|_r, w| {
            w.halt().on();
            w.reset().on();
            w
        });
        clkdiv.modify(|_r, w| {
            w.halt().off();
            w.reset().off();
            unsafe { w.div().bits(self.div.into_bits()) };
            w
        });

        while clkdiv.read().unstab().is_on() {}

        Ok(freq / self.div.into_divisor())
    }
}
