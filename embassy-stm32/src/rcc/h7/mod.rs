use core::marker::PhantomData;

use embassy::util::Unborrow;
use embassy_extras::unborrow;

use crate::fmt::assert;
use crate::pac::peripherals;
use crate::pac::RCC;
use crate::time::Hertz;

mod pll;
use pll::pll_setup;
pub use pll::PllConfig;

const HSI: Hertz = Hertz(64_000_000);
const CSI: Hertz = Hertz(4_000_000);
const HSI48: Hertz = Hertz(48_000_000);
const LSI: Hertz = Hertz(32_000);

/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Default)]
pub struct Config {
    pub hse: Option<Hertz>,
    pub bypass_hse: bool,
    pub sys_ck: Option<Hertz>,
    pub per_ck: Option<Hertz>,
    rcc_hclk: Option<Hertz>,
    pub hclk: Option<Hertz>,
    pub pclk1: Option<Hertz>,
    pub pclk2: Option<Hertz>,
    pub pclk3: Option<Hertz>,
    pub pclk4: Option<Hertz>,
    pub pll1: PllConfig,
    pub pll2: PllConfig,
    pub pll3: PllConfig,
    pub vos: VoltageScale,
}

/// Voltage Scale
///
/// Represents the voltage range feeding the CPU core. The maximum core
/// clock frequency depends on this value.
#[derive(Copy, Clone, PartialEq)]
pub enum VoltageScale {
    /// VOS 0 range VCORE 1.26V - 1.40V
    Scale0,
    /// VOS 1 range VCORE 1.15V - 1.26V
    Scale1,
    /// VOS 2 range VCORE 1.05V - 1.15V
    Scale2,
    /// VOS 3 range VCORE 0.95V - 1.05V
    Scale3,
}

impl Default for VoltageScale {
    fn default() -> Self {
        Self::Scale1
    }
}

pub struct Rcc<'d> {
    inner: PhantomData<&'d ()>,
    config: Config,
}

impl<'d> Rcc<'d> {
    pub fn new(_rcc: impl Unborrow<Target = peripherals::RCC> + 'd, config: Config) -> Self {
        Self {
            inner: PhantomData,
            config,
        }
    }

    // TODO: FLASH and PWR
    /// Freeze the core clocks, returning a Core Clocks Distribution
    /// and Reset (CCDR) structure. The actual frequency of the clocks
    /// configured is returned in the `clocks` member of the CCDR
    /// structure.
    ///
    /// Note that `freeze` will never result in a clock _faster_ than
    /// that specified. It may result in a clock that is a factor of [1,
    /// 2) slower.
    ///
    /// `syscfg` is required to enable the I/O compensation cell.
    ///
    /// # Panics
    ///
    /// If a clock specification cannot be achieved within the
    /// hardware specification then this function will panic. This
    /// function may also panic if a clock specification can be
    /// achieved, but the mechanism for doing so is not yet
    /// implemented here.
    pub fn freeze(mut self) {
        use crate::pac::rcc::vals::{Ckpersel, Hpre, Hsidiv, Hsion, Lsion, Timpre};

        let srcclk = self.config.hse.unwrap_or(HSI); // Available clocks
        let (sys_ck, sys_use_pll1_p) = self.sys_ck_setup(srcclk);

        // NOTE(unsafe) We have exclusive access to the RCC
        let (pll1_p_ck, pll1_q_ck, pll1_r_ck) =
            unsafe { pll_setup(srcclk.0, &self.config.pll1, 0) };
        let (pll2_p_ck, pll2_q_ck, pll2_r_ck) =
            unsafe { pll_setup(srcclk.0, &self.config.pll2, 1) };
        let (pll3_p_ck, pll3_q_ck, pll3_r_ck) =
            unsafe { pll_setup(srcclk.0, &self.config.pll3, 2) };

        let sys_ck = if sys_use_pll1_p {
            Hertz(pll1_p_ck.unwrap()) // Must have been set by sys_ck_setup
        } else {
            sys_ck
        };

        // NOTE(unsafe) We own the regblock
        unsafe {
            // This routine does not support HSIDIV != 1. To
            // do so it would need to ensure all PLLxON bits are clear
            // before changing the value of HSIDIV
            let cr = RCC.cr().read();
            assert!(cr.hsion() == Hsion::ON);
            assert!(cr.hsidiv() == Hsidiv::DIV1);

            RCC.csr().modify(|w| w.set_lsion(Lsion::ON));
            while !RCC.csr().read().lsirdy() {}
        }

        // per_ck from HSI by default
        let (per_ck, ckpersel) = match (self.config.per_ck == self.config.hse, self.config.per_ck) {
            (true, Some(hse)) => (hse, Ckpersel::HSE), // HSE
            (_, Some(CSI)) => (CSI, Ckpersel::CSI),    // CSI
            _ => (HSI, Ckpersel::HSI),                 // HSI
        };

        // D1 Core Prescaler
        // Set to 1
        let d1cpre_bits = 0;
        let d1cpre_div = 1;
        let sys_d1cpre_ck = sys_ck.0 / d1cpre_div;

        // Timer prescaler selection
        let timpre = Timpre::DEFAULTX2;

        // Refer to part datasheet "General operating conditions"
        // table for (rev V). We do not assert checks for earlier
        // revisions which may have lower limits.
        let (sys_d1cpre_ck_max, rcc_hclk_max, pclk_max) = match self.config.vos {
            VoltageScale::Scale0 => (480_000_000, 240_000_000, 120_000_000),
            VoltageScale::Scale1 => (400_000_000, 200_000_000, 100_000_000),
            VoltageScale::Scale2 => (300_000_000, 150_000_000, 75_000_000),
            _ => (200_000_000, 100_000_000, 50_000_000),
        };
        assert!(sys_d1cpre_ck <= sys_d1cpre_ck_max);

        let rcc_hclk = self
            .config
            .rcc_hclk
            .map(|v| v.0)
            .unwrap_or(sys_d1cpre_ck / 2);
        assert!(rcc_hclk <= rcc_hclk_max);

        // Estimate divisor
        let (hpre_bits, hpre_div) = match (sys_d1cpre_ck + rcc_hclk - 1) / rcc_hclk {
            0 => unreachable!(),
            1 => (Hpre::DIV1, 1),
            2 => (Hpre::DIV2, 2),
            3..=5 => (Hpre::DIV4, 4),
            6..=11 => (Hpre::DIV8, 8),
            12..=39 => (Hpre::DIV16, 16),
            40..=95 => (Hpre::DIV64, 64),
            96..=191 => (Hpre::DIV128, 128),
            192..=383 => (Hpre::DIV256, 256),
            _ => (Hpre::DIV512, 512),
        };
        // Calculate real AXI and AHB clock
        let rcc_hclk = sys_d1cpre_ck / hpre_div;
        assert!(rcc_hclk <= rcc_hclk_max);
        let rcc_aclk = rcc_hclk; // AXI clock is always equal to AHB clock on H7

        todo!()
    }

    /// Setup sys_ck
    /// Returns sys_ck frequency, and a pll1_p_ck
    fn sys_ck_setup(&mut self, srcclk: Hertz) -> (Hertz, bool) {
        // Compare available with wanted clocks
        let sys_ck = self.config.sys_ck.unwrap_or(srcclk);

        if sys_ck != srcclk {
            // The requested system clock is not the immediately available
            // HSE/HSI clock. Perhaps there are other ways of obtaining
            // the requested system clock (such as `HSIDIV`) but we will
            // ignore those for now.
            //
            // Therefore we must use pll1_p_ck
            let pll1_p_ck = match self.config.pll1.p_ck {
                Some(p_ck) => {
                    assert!(p_ck == sys_ck,
                            "Error: Cannot set pll1_p_ck independently as it must be used to generate sys_ck");
                    Some(p_ck)
                }
                None => Some(sys_ck),
            };
            self.config.pll1.p_ck = pll1_p_ck;

            (sys_ck, true)
        } else {
            // sys_ck is derived directly from a source clock
            // (HSE/HSI). pll1_p_ck can be as requested
            (sys_ck, false)
        }
    }
}
