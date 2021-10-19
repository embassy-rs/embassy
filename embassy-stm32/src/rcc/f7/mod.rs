mod max;

use crate::pac::{FLASH, PWR, RCC};
use crate::peripherals;
use crate::rcc::{get_freqs, set_freqs, Clocks};
use crate::time::Hertz;
use core::marker::PhantomData;
use embassy::util::Unborrow;

const HSI: u32 = 16_000_000;

#[non_exhaustive]
#[derive(Default)]
pub struct Config {
    pub hse: Option<Hertz>,
    pub bypass_hse: bool,
    pub hclk: Option<Hertz>,
    pub sys_ck: Option<Hertz>,
    pub pclk1: Option<Hertz>,
    pub pclk2: Option<Hertz>,

    pub pll48: bool,
}

/// RCC peripheral
pub struct Rcc<'d> {
    config: Config,
    phantom: PhantomData<&'d mut peripherals::RCC>,
}

impl<'d> Rcc<'d> {
    pub fn new(_rcc: impl Unborrow<Target = peripherals::RCC> + 'd, config: Config) -> Self {
        if let Some(hse) = config.hse {
            if config.bypass_hse {
                assert!((max::HSE_BYPASS_MIN..=max::HSE_BYPASS_MAX).contains(&hse.0));
            } else {
                assert!((max::HSE_OSC_MIN..=max::HSE_OSC_MAX).contains(&hse.0));
            }
        }
        Self {
            config,
            phantom: PhantomData,
        }
    }

    fn freeze(mut self) -> Clocks {
        use super::sealed::RccPeripheral;
        use crate::pac::pwr::vals::Vos;
        use crate::pac::rcc::vals::{Hpre, Hsebyp, Ppre, Sw};

        let base_clock = self.config.hse.map(|hse| hse.0).unwrap_or(HSI);
        let sysclk = self.config.sys_ck.map(|sys| sys.0).unwrap_or(base_clock);
        let sysclk_on_pll = sysclk != base_clock;

        assert!((max::SYSCLK_MIN..=max::SYSCLK_MAX).contains(&sysclk));

        let plls = self.setup_pll(
            base_clock,
            self.config.hse.is_some(),
            if sysclk_on_pll { Some(sysclk) } else { None },
            self.config.pll48,
        );

        if self.config.pll48 {
            assert!(
                // USB specification allows +-0.25%
                plls.pll48clk
                    .map(|freq| (max::PLL_48_CLK as i32 - freq as i32).abs()
                        <= max::PLL_48_TOLERANCE as i32)
                    .unwrap_or(false)
            );
        }

        let sysclk = if sysclk_on_pll {
            unwrap!(plls.pllsysclk)
        } else {
            sysclk
        };

        // AHB prescaler
        let hclk = self.config.hclk.map(|h| h.0).unwrap_or(sysclk);
        let (hpre_bits, hpre_div) = match (sysclk + hclk - 1) / hclk {
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

        // Calculate real AHB clock
        let hclk = sysclk / hpre_div;

        assert!(hclk < max::HCLK_MAX);

        let pclk1 = self
            .config
            .pclk1
            .map(|p| p.0)
            .unwrap_or_else(|| core::cmp::min(max::PCLK1_MAX, hclk));

        let (ppre1_bits, ppre1) = match (hclk + pclk1 - 1) / pclk1 {
            0 => unreachable!(),
            1 => (0b000, 1),
            2 => (0b100, 2),
            3..=5 => (0b101, 4),
            6..=11 => (0b110, 8),
            _ => (0b111, 16),
        };
        let timer_mul1 = if ppre1 == 1 { 1 } else { 2 };

        // Calculate real APB1 clock
        let pclk1 = hclk / ppre1;
        assert!((max::PCLK1_MIN..=max::PCLK1_MAX).contains(&pclk1));

        let pclk2 = self
            .config
            .pclk2
            .map(|p| p.0)
            .unwrap_or_else(|| core::cmp::min(max::PCLK2_MAX, hclk));
        let (ppre2_bits, ppre2) = match (hclk + pclk2 - 1) / pclk2 {
            0 => unreachable!(),
            1 => (0b000, 1),
            2 => (0b100, 2),
            3..=5 => (0b101, 4),
            6..=11 => (0b110, 8),
            _ => (0b111, 16),
        };
        let timer_mul2 = if ppre2 == 1 { 1 } else { 2 };

        // Calculate real APB2 clock
        let pclk2 = hclk / ppre2;
        assert!((max::PCLK2_MIN..=max::PCLK2_MAX).contains(&pclk2));

        Self::flash_setup(sysclk);

        if self.config.hse.is_some() {
            // NOTE(unsafe) We own the peripheral block
            unsafe {
                RCC.cr().modify(|w| {
                    w.set_hsebyp(Hsebyp(self.config.bypass_hse as u8));
                    w.set_hseon(true);
                });
                while !RCC.cr().read().hserdy() {}
            }
        }

        if plls.use_pll {
            unsafe {
                RCC.cr().modify(|w| w.set_pllon(false));

                // enable PWR and setup VOSScale

                RCC.apb1enr().modify(|w| w.set_pwren(true));

                let vos_scale = if sysclk <= 144_000_000 {
                    3
                } else if sysclk <= 168_000_000 {
                    2
                } else {
                    1
                };
                PWR.cr1().modify(|w| {
                    w.set_vos(match vos_scale {
                        3 => Vos::SCALE3,
                        2 => Vos::SCALE2,
                        1 => Vos::SCALE1,
                        _ => panic!("Invalid VOS Scale."),
                    })
                });

                RCC.cr().modify(|w| w.set_pllon(true));

                while !RCC.cr().read().pllrdy() {}

                if hclk > max::HCLK_OVERDRIVE_FREQUENCY {
                    peripherals::PWR::enable();

                    PWR.cr1().modify(|w| w.set_oden(true));
                    while !PWR.csr1().read().odrdy() {}

                    PWR.cr1().modify(|w| w.set_odswen(true));
                    while !PWR.csr1().read().odswrdy() {}
                }

                while !RCC.cr().read().pllrdy() {}
            }
        }

        unsafe {
            RCC.cfgr().modify(|w| {
                w.set_ppre2(Ppre(ppre2_bits));
                w.set_ppre1(Ppre(ppre1_bits));
                w.set_hpre(hpre_bits);
            });

            // Wait for the new prescalers to kick in
            // "The clocks are divided with the new prescaler factor from 1 to 16 AHB cycles after write"
            cortex_m::asm::delay(16);

            RCC.cfgr().modify(|w| {
                w.set_sw(if sysclk_on_pll {
                    Sw::PLL
                } else if self.config.hse.is_some() {
                    Sw::HSE
                } else {
                    Sw::HSI
                })
            });
        }

        Clocks {
            sys: Hertz(sysclk),
            apb1: Hertz(pclk1),
            apb2: Hertz(pclk2),

            apb1_tim: Hertz(pclk1 * timer_mul1),
            apb2_tim: Hertz(pclk2 * timer_mul2),

            ahb1: Hertz(hclk),
            ahb2: Hertz(hclk),
            ahb3: Hertz(hclk),
        }
    }

    // Safety: RCC init must have been called
    pub fn clocks(&self) -> &'static Clocks {
        unsafe { get_freqs() }
    }

    fn setup_pll(
        &mut self,
        pllsrcclk: u32,
        use_hse: bool,
        pllsysclk: Option<u32>,
        pll48clk: bool,
    ) -> PllResults {
        use crate::pac::rcc::vals::{Pllp, Pllsrc};

        let sysclk = pllsysclk.unwrap_or(pllsrcclk);
        if pllsysclk.is_none() && !pll48clk {
            // NOTE(unsafe) We have a mutable borrow to the owner of the RegBlock
            unsafe {
                RCC.pllcfgr()
                    .modify(|w| w.set_pllsrc(Pllsrc(use_hse as u8)));
            }

            return PllResults {
                use_pll: false,
                pllsysclk: None,
                pll48clk: None,
            };
        }
        // Input divisor from PLL source clock, must result to frequency in
        // the range from 1 to 2 MHz
        let pllm_min = (pllsrcclk + 1_999_999) / 2_000_000;
        let pllm_max = pllsrcclk / 1_000_000;

        // Sysclk output divisor must be one of 2, 4, 6 or 8
        let sysclk_div = core::cmp::min(8, (432_000_000 / sysclk) & !1);

        let target_freq = if pll48clk {
            48_000_000
        } else {
            sysclk * sysclk_div
        };

        // Find the lowest pllm value that minimize the difference between
        // target frequency and the real vco_out frequency.
        let pllm = unwrap!((pllm_min..=pllm_max).min_by_key(|pllm| {
            let vco_in = pllsrcclk / pllm;
            let plln = target_freq / vco_in;
            target_freq - vco_in * plln
        }));

        let vco_in = pllsrcclk / pllm;
        assert!((1_000_000..=2_000_000).contains(&vco_in));

        // Main scaler, must result in >= 100MHz (>= 192MHz for F401)
        // and <= 432MHz, min 50, max 432
        let plln = if pll48clk {
            // try the different valid pllq according to the valid
            // main scaller values, and take the best
            let pllq = unwrap!((4..=9).min_by_key(|pllq| {
                let plln = 48_000_000 * pllq / vco_in;
                let pll48_diff = 48_000_000 - vco_in * plln / pllq;
                let sysclk_diff = (sysclk as i32 - (vco_in * plln / sysclk_div) as i32).abs();
                (pll48_diff, sysclk_diff)
            }));
            48_000_000 * pllq / vco_in
        } else {
            sysclk * sysclk_div / vco_in
        };

        let pllp = (sysclk_div / 2) - 1;

        let pllq = (vco_in * plln + 47_999_999) / 48_000_000;
        let real_pll48clk = vco_in * plln / pllq;

        unsafe {
            RCC.pllcfgr().modify(|w| {
                w.set_pllm(pllm as u8);
                w.set_plln(plln as u16);
                w.set_pllp(Pllp(pllp as u8));
                w.set_pllq(pllq as u8);
                w.set_pllsrc(Pllsrc(use_hse as u8));
            });
        }

        let real_pllsysclk = vco_in * plln / sysclk_div;

        PllResults {
            use_pll: true,
            pllsysclk: Some(real_pllsysclk),
            pll48clk: if pll48clk { Some(real_pll48clk) } else { None },
        }
    }

    fn flash_setup(sysclk: u32) {
        use crate::pac::flash::vals::Latency;

        // Be conservative with voltage ranges
        const FLASH_LATENCY_STEP: u32 = 30_000_000;

        critical_section::with(|_| unsafe {
            FLASH
                .acr()
                .modify(|w| w.set_latency(Latency(((sysclk - 1) / FLASH_LATENCY_STEP) as u8)));
        });
    }
}

pub unsafe fn init(config: Config) {
    let r = <peripherals::RCC as embassy::util::Steal>::steal();
    let clocks = Rcc::new(r, config).freeze();
    set_freqs(clocks);
}

struct PllResults {
    use_pll: bool,
    pllsysclk: Option<u32>,
    pll48clk: Option<u32>,
}
