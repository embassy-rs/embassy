use core::marker::PhantomData;

use embassy::util::Unborrow;

use crate::fmt::{assert, panic};
use crate::pac::peripherals;
use crate::pac::rcc::vals::Timpre;
use crate::pac::{DBGMCU, RCC, SYSCFG};
use crate::pwr::{Power, VoltageScale};
use crate::time::Hertz;

mod pll;
use pll::pll_setup;
pub use pll::PllConfig;

const HSI: Hertz = Hertz(64_000_000);
const CSI: Hertz = Hertz(4_000_000);
const HSI48: Hertz = Hertz(48_000_000);
const LSI: Hertz = Hertz(32_000);

/// Core clock frequencies
#[derive(Clone, Copy)]
pub struct CoreClocks {
    pub hclk: Hertz,
    pub pclk1: Hertz,
    pub pclk2: Hertz,
    pub pclk3: Hertz,
    pub pclk4: Hertz,
    pub ppre1: u8,
    pub ppre2: u8,
    pub ppre3: u8,
    pub ppre4: u8,
    pub csi_ck: Option<Hertz>,
    pub hsi_ck: Option<Hertz>,
    pub hsi48_ck: Option<Hertz>,
    pub lsi_ck: Option<Hertz>,
    pub per_ck: Option<Hertz>,
    pub hse_ck: Option<Hertz>,
    pub pll1_p_ck: Option<Hertz>,
    pub pll1_q_ck: Option<Hertz>,
    pub pll1_r_ck: Option<Hertz>,
    pub pll2_p_ck: Option<Hertz>,
    pub pll2_q_ck: Option<Hertz>,
    pub pll2_r_ck: Option<Hertz>,
    pub pll3_p_ck: Option<Hertz>,
    pub pll3_q_ck: Option<Hertz>,
    pub pll3_r_ck: Option<Hertz>,
    pub timx_ker_ck: Option<Hertz>,
    pub timy_ker_ck: Option<Hertz>,
    pub sys_ck: Hertz,
    pub c_ck: Hertz,
}

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
    pub fn freeze(mut self, pwr: &Power) -> CoreClocks {
        use crate::pac::rcc::vals::{
            Apb4enrSyscfgen, Ckpersel, D1ppre, D2ppre1, D3ppre, Hpre, Hsebyp, Hsidiv, Hsion, Lsion,
            Pllsrc, Sw,
        };

        let srcclk = self.config.hse.unwrap_or(HSI); // Available clocks
        let (sys_ck, sys_use_pll1_p) = self.sys_ck_setup(srcclk);

        // Configure traceclk from PLL if needed
        self.traceclk_setup(sys_use_pll1_p);

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

        // Refer to part datasheet "General operating conditions"
        // table for (rev V). We do not assert checks for earlier
        // revisions which may have lower limits.
        let (sys_d1cpre_ck_max, rcc_hclk_max, pclk_max) = match pwr.vos {
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
            0 => panic!(),
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
                                 // Timer prescaler selection
        let timpre = Timpre::DEFAULTX2;

        let requested_pclk1 = self
            .config
            .pclk1
            .map(|v| v.0)
            .unwrap_or_else(|| pclk_max.min(rcc_hclk / 2));
        let (rcc_pclk1, ppre1_bits, ppre1, rcc_timerx_ker_ck) =
            Self::ppre_calculate(requested_pclk1, rcc_hclk, pclk_max, Some(timpre));

        let requested_pclk2 = self
            .config
            .pclk2
            .map(|v| v.0)
            .unwrap_or_else(|| pclk_max.min(rcc_hclk / 2));
        let (rcc_pclk2, ppre2_bits, ppre2, rcc_timery_ker_ck) =
            Self::ppre_calculate(requested_pclk2, rcc_hclk, pclk_max, Some(timpre));

        let requested_pclk3 = self
            .config
            .pclk3
            .map(|v| v.0)
            .unwrap_or_else(|| pclk_max.min(rcc_hclk / 2));
        let (rcc_pclk3, ppre3_bits, ppre3, _) =
            Self::ppre_calculate(requested_pclk3, rcc_hclk, pclk_max, None);

        let requested_pclk4 = self
            .config
            .pclk4
            .map(|v| v.0)
            .unwrap_or_else(|| pclk_max.min(rcc_hclk / 2));
        let (rcc_pclk4, ppre4_bits, ppre4, _) =
            Self::ppre_calculate(requested_pclk4, rcc_hclk, pclk_max, None);

        Self::flash_setup(rcc_aclk, pwr.vos);

        // Start switching clocks -------------------
        // NOTE(unsafe) We have the RCC singleton
        unsafe {
            // Ensure CSI is on and stable
            RCC.cr().modify(|w| w.set_csion(Hsion::ON));
            while !RCC.cr().read().csirdy() {}

            // Ensure HSI48 is on and stable
            RCC.cr().modify(|w| w.set_hsi48on(Hsion::ON));
            while RCC.cr().read().hsi48on() == Hsion::OFF {}

            // XXX: support MCO ?

            let hse_ck = match self.config.hse {
                Some(hse) => {
                    // Ensure HSE is on and stable
                    RCC.cr().modify(|w| {
                        w.set_hseon(Hsion::ON);
                        w.set_hsebyp(if self.config.bypass_hse {
                            Hsebyp::BYPASSED
                        } else {
                            Hsebyp::NOTBYPASSED
                        });
                    });
                    while !RCC.cr().read().hserdy() {}
                    Some(hse)
                }
                None => None,
            };

            let pllsrc = if self.config.hse.is_some() {
                Pllsrc::HSE
            } else {
                Pllsrc::HSI
            };
            RCC.pllckselr().modify(|w| w.set_pllsrc(pllsrc));

            if pll1_p_ck.is_some() {
                RCC.cr().modify(|w| w.set_pll1on(Hsion::ON));
                while !RCC.cr().read().pll1rdy() {}
            }

            if pll2_p_ck.is_some() {
                RCC.cr().modify(|w| w.set_pll2on(Hsion::ON));
                while !RCC.cr().read().pll2rdy() {}
            }

            if pll3_p_ck.is_some() {
                RCC.cr().modify(|w| w.set_pll3on(Hsion::ON));
                while !RCC.cr().read().pll3rdy() {}
            }

            // Core Prescaler / AHB Prescaler / APB3 Prescaler
            RCC.d1cfgr().modify(|w| {
                w.set_d1cpre(Hpre(d1cpre_bits));
                w.set_d1ppre(D1ppre(ppre3_bits));
                w.set_hpre(hpre_bits)
            });
            // Ensure core prescaler value is valid before future lower
            // core voltage
            while RCC.d1cfgr().read().d1cpre().0 != d1cpre_bits {}

            // APB1 / APB2 Prescaler
            RCC.d2cfgr().modify(|w| {
                w.set_d2ppre1(D2ppre1(ppre1_bits));
                w.set_d2ppre2(D2ppre1(ppre2_bits));
            });

            // APB4 Prescaler
            RCC.d3cfgr().modify(|w| w.set_d3ppre(D3ppre(ppre4_bits)));

            // Peripheral Clock (per_ck)
            RCC.d1ccipr().modify(|w| w.set_ckpersel(ckpersel));

            // Set timer clocks prescaler setting
            RCC.cfgr().modify(|w| w.set_timpre(timpre));

            // Select system clock source
            let sw = match (sys_use_pll1_p, self.config.hse.is_some()) {
                (true, _) => Sw::PLL1,
                (false, true) => Sw::HSE,
                _ => Sw::HSI,
            };
            RCC.cfgr().modify(|w| w.set_sw(sw));
            while RCC.cfgr().read().sws() != sw.0 {}

            // IO compensation cell - Requires CSI clock and SYSCFG
            assert!(RCC.cr().read().csirdy());
            RCC.apb4enr()
                .modify(|w| w.set_syscfgen(Apb4enrSyscfgen::ENABLED));

            // Enable the compensation cell, using back-bias voltage code
            // provide by the cell.
            critical_section::with(|_| {
                SYSCFG.cccsr().modify(|w| {
                    w.set_en(true);
                    w.set_cs(false);
                    w.set_hslv(false);
                })
            });
            while !SYSCFG.cccsr().read().ready() {}

            CoreClocks {
                hclk: Hertz(rcc_hclk),
                pclk1: Hertz(rcc_pclk1),
                pclk2: Hertz(rcc_pclk2),
                pclk3: Hertz(rcc_pclk3),
                pclk4: Hertz(rcc_pclk4),
                ppre1,
                ppre2,
                ppre3,
                ppre4,
                csi_ck: Some(CSI),
                hsi_ck: Some(HSI),
                hsi48_ck: Some(HSI48),
                lsi_ck: Some(LSI),
                per_ck: Some(per_ck),
                hse_ck,
                pll1_p_ck: pll1_p_ck.map(Hertz),
                pll1_q_ck: pll1_q_ck.map(Hertz),
                pll1_r_ck: pll1_r_ck.map(Hertz),
                pll2_p_ck: pll2_p_ck.map(Hertz),
                pll2_q_ck: pll2_q_ck.map(Hertz),
                pll2_r_ck: pll2_r_ck.map(Hertz),
                pll3_p_ck: pll3_p_ck.map(Hertz),
                pll3_q_ck: pll3_q_ck.map(Hertz),
                pll3_r_ck: pll3_r_ck.map(Hertz),
                timx_ker_ck: rcc_timerx_ker_ck.map(Hertz),
                timy_ker_ck: rcc_timery_ker_ck.map(Hertz),
                sys_ck,
                c_ck: Hertz(sys_d1cpre_ck),
            }
        }
    }

    /// Enables debugging during WFI/WFE
    ///
    /// Set `enable_dma1` to true if you do not have at least one bus master (other than the CPU)
    /// enable during WFI/WFE
    pub fn enable_debug_wfe(&mut self, _dbg: &mut peripherals::DBGMCU, enable_dma1: bool) {
        use crate::pac::rcc::vals::Ahb1enrDma1en;

        // NOTE(unsafe) We have exclusive access to the RCC and DBGMCU
        unsafe {
            if enable_dma1 {
                RCC.ahb1enr()
                    .modify(|w| w.set_dma1en(Ahb1enrDma1en::ENABLED));
            }

            DBGMCU.cr().modify(|w| {
                w.set_dbgsleep_d1(true);
                w.set_dbgstby_d1(true);
                w.set_dbgstop_d1(true);
            });
        }
    }

    /// Setup traceclk
    /// Returns a pll1_r_ck
    fn traceclk_setup(&mut self, sys_use_pll1_p: bool) {
        let pll1_r_ck = match (sys_use_pll1_p, self.config.pll1.r_ck) {
            // pll1_p_ck selected as system clock but pll1_r_ck not
            // set. The traceclk mux is synchronous with the system
            // clock mux, but has pll1_r_ck as an input. In order to
            // keep traceclk running, we force a pll1_r_ck.
            (true, None) => Some(Hertz(self.config.pll1.p_ck.unwrap().0 / 2)),

            // Either pll1 not selected as system clock, free choice
            // of pll1_r_ck. Or pll1 is selected, assume user has set
            // a suitable pll1_r_ck frequency.
            _ => self.config.pll1.r_ck,
        };
        self.config.pll1.r_ck = pll1_r_ck;
    }

    /// Divider calculator for pclk 1 - 4
    ///
    /// Returns real pclk, bits, ppre and the timer kernel clock
    fn ppre_calculate(
        requested_pclk: u32,
        hclk: u32,
        max_pclk: u32,
        tim_pre: Option<Timpre>,
    ) -> (u32, u8, u8, Option<u32>) {
        let (bits, ppre) = match (hclk + requested_pclk - 1) / requested_pclk {
            0 => panic!(),
            1 => (0b000, 1),
            2 => (0b100, 2),
            3..=5 => (0b101, 4),
            6..=11 => (0b110, 8),
            _ => (0b111, 16),
        };
        let real_pclk = hclk / u32::from(ppre);
        assert!(real_pclk <= max_pclk);

        let tim_ker_clk = if let Some(tim_pre) = tim_pre {
            let clk = match (bits, tim_pre) {
                (0b101, Timpre::DEFAULTX2) => hclk / 2,
                (0b110, Timpre::DEFAULTX4) => hclk / 2,
                (0b110, Timpre::DEFAULTX2) => hclk / 4,
                (0b111, Timpre::DEFAULTX4) => hclk / 4,
                (0b111, Timpre::DEFAULTX2) => hclk / 8,
                _ => hclk,
            };
            Some(clk)
        } else {
            None
        };
        (real_pclk, bits, ppre, tim_ker_clk)
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

    fn flash_setup(rcc_aclk: u32, vos: VoltageScale) {
        use crate::pac::FLASH;

        // ACLK in MHz, round down and subtract 1 from integers. eg.
        // 61_999_999 -> 61MHz
        // 62_000_000 -> 61MHz
        // 62_000_001 -> 62MHz
        let rcc_aclk_mhz = (rcc_aclk - 1) / 1_000_000;

        // See RM0433 Rev 7 Table 17. FLASH recommended number of wait
        // states and programming delay
        let (wait_states, progr_delay) = match vos {
            // VOS 0 range VCORE 1.26V - 1.40V
            VoltageScale::Scale0 => match rcc_aclk_mhz {
                0..=69 => (0, 0),
                70..=139 => (1, 1),
                140..=184 => (2, 1),
                185..=209 => (2, 2),
                210..=224 => (3, 2),
                225..=239 => (4, 2),
                _ => (7, 3),
            },
            // VOS 1 range VCORE 1.15V - 1.26V
            VoltageScale::Scale1 => match rcc_aclk_mhz {
                0..=69 => (0, 0),
                70..=139 => (1, 1),
                140..=184 => (2, 1),
                185..=209 => (2, 2),
                210..=224 => (3, 2),
                _ => (7, 3),
            },
            // VOS 2 range VCORE 1.05V - 1.15V
            VoltageScale::Scale2 => match rcc_aclk_mhz {
                0..=54 => (0, 0),
                55..=109 => (1, 1),
                110..=164 => (2, 1),
                165..=224 => (3, 2),
                _ => (7, 3),
            },
            // VOS 3 range VCORE 0.95V - 1.05V
            VoltageScale::Scale3 => match rcc_aclk_mhz {
                0..=44 => (0, 0),
                45..=89 => (1, 1),
                90..=134 => (2, 1),
                135..=179 => (3, 2),
                180..=224 => (4, 2),
                _ => (7, 3),
            },
        };

        // NOTE(unsafe) Atomic write
        unsafe {
            FLASH.acr().write(|w| {
                w.set_wrhighfreq(progr_delay);
                w.set_latency(wait_states)
            });
            while FLASH.acr().read().latency() != wait_states {}
        }
    }
}

// TODO
pub unsafe fn init(_config: Config) {}
