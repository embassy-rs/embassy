use crate::pac;
use core::sync::atomic::{AtomicU32, Ordering};

pub(crate) struct Clocks {
    /// PLL, SYSOSC, or HFXT
    pub m_clk: AtomicU32,
    /// LFOSC or LFXT
    pub rtc_clk: AtomicU32,
    /// ULPCLK
    pub ulp_clk: AtomicU32,
    /// MFCLK
    pub mf_clk: AtomicU32,
    /// SYSOSC
    pub sys_osc: AtomicU32,
    // HFCLK
    // pub hf_clk: AtomicU32,
}

pub(crate) static CLOCKS: Clocks = Clocks {
    m_clk: AtomicU32::new(0),
    rtc_clk: AtomicU32::new(0),
    ulp_clk: AtomicU32::new(0),
    mf_clk: AtomicU32::new(0),
    sys_osc: AtomicU32::new(0),
    // hf_clk: AtomicU32::new(0),
};

#[cfg(mspm0g)]
#[derive(Clone)]
pub struct PllConfig {
    pdiv: u8,
    qdiv: u8,
    rdiv2x: u8,
}

#[derive(Clone)]
pub enum SysOscSpeed {
    HighSpeed,
    // UserTrim,
    LowSpeed,
}

impl SysOscSpeed {
    pub fn frequency(&self) -> u32 {
        match self {
            #[cfg(any(mspm0g, mspm0l, mspm0h))]
            SysOscSpeed::HighSpeed => 32_000_000,
            // NOTE: TRM is not clear about limit also specifying 32MHz possible
            #[cfg(any(mspm0c))]
            SysOscSpeed::HighSpeed => 24_000_000,
            // SysOscSpeed::UserTrim => unimplemented!(),
            SysOscSpeed::LowSpeed => 4_000_000,
        }
    }
}

#[derive(Clone)]
pub struct SysOscConfig {
    fcl_enabled: bool,
    speed: SysOscSpeed,
}

#[derive(Clone)]
pub enum MClkSource {
    #[cfg(mspm0g)]
    PLL,
    SYSOSC,
    LFCLK,
    HFCLK,
}

#[derive(Clone)]
pub struct ClockConfig {
    #[cfg(mspm0g)]
    pub pll_config: Option<PllConfig>,

    pub sysosc_config: SysOscConfig,

    pub mclk_source: MClkSource,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            #[cfg(mspm0g)]
            pll_config: Some(PllConfig {
                pdiv: 0,
                qdiv: 1,
                rdiv2x: 0,
            }),
            sysosc_config: SysOscConfig {
                fcl_enabled: true,
                speed: SysOscSpeed::HighSpeed,
            },
            #[cfg(mspm0g)]
            mclk_source: MClkSource::PLL,
            #[cfg(not(mspm0g))]
            mclk_source: MClkSource::SYSOSC,
        }
    }
}

// TODO: create extensive ClockConfig
pub(crate) unsafe fn init(config: ClockConfig) {
    // TODO: Further clock configuration

    CLOCKS.m_clk.store(
        match config.mclk_source {
            MClkSource::PLL => 80_000_000, // TODO: calculate
            MClkSource::SYSOSC => config.sysosc_config.speed.frequency(),
            MClkSource::LFCLK => unimplemented!("LFCLK is not supported yet"),
            MClkSource::HFCLK => unimplemented!("HFCLK is not supported yet"),
        },
        Ordering::Relaxed,
    );

    CLOCKS.rtc_clk.store(32_000, Ordering::Relaxed);
    CLOCKS.mf_clk.store(4_000_000, Ordering::Relaxed);
    CLOCKS
        .sys_osc
        .store(config.sysosc_config.speed.frequency(), Ordering::Relaxed);

    // TODO: this can deviate in case HF/PLL is not running at max freq
    #[cfg(mspm0g)]
    CLOCKS.ulp_clk.store(40_000_000, Ordering::Relaxed);

    pac::SYSCTL.sysoscfclctl().write(|w| {
        w.set_setusefcl(config.sysosc_config.fcl_enabled);
        w.set_setuseexres(config.sysosc_config.fcl_enabled);
    });
    pac::SYSCTL.sysosccfg().write(|w| {
        w.set_freq(match config.sysosc_config.speed {
            SysOscSpeed::HighSpeed => mspm0_metapac::sysctl::vals::SysosccfgFreq::SYSOSCBASE,
            SysOscSpeed::LowSpeed => mspm0_metapac::sysctl::vals::SysosccfgFreq::SYSOSC4M,
        });
    });

    pac::SYSCTL.mclkcfg().modify(|w| {
        // Enable MFCLK
        match config.mclk_source {
            MClkSource::SYSOSC => {
                w.set_usemftick(true);
                w.set_mdiv(0);
            }
            #[cfg(mspm0g)]
            MClkSource::PLL => {
                w.set_usehsclk(true);
            }
            MClkSource::HFCLK => {
                w.set_usehsclk(false);
            }
            MClkSource::LFCLK => {
                w.set_uselfclk(true);
            }
        }
    });

    // Enable MFCLK for peripheral use
    //
    // TODO: Optional?
    pac::SYSCTL.genclken().modify(|w| {
        w.set_mfpclken(true);
    });

    pac::SYSCTL.hsclkcfg().write(|w| match config.mclk_source {
        #[cfg(mspm0g)]
        MClkSource::PLL => w.set_hsclksel(mspm0_metapac::sysctl::vals::Hsclksel::SYSPLL),
        MClkSource::HFCLK => w.set_hsclksel(mspm0_metapac::sysctl::vals::Hsclksel::HFCLKCLK),
        _ => {}
    });

    #[cfg(mspm0g)]
    if let Some(pll_config) = config.pll_config {
        pac::SYSCTL.hsclkcfg().write(|w| {
            // For now PLL is assumed
            w.set_hsclksel(mspm0_metapac::sysctl::vals::Hsclksel::SYSPLL);
        });

        // let divs = crate::common::hillclimb([0u32, 1u32, 0u32], |divs| {
        //     if divs[0] > 3 || !(1..127).contains(&divs[1]) || divs[2] > 15 {
        //         return i32::MAX;
        //     }

        //     let pdiv = 1 << divs[0];
        //     let qdiv = divs[1] + 1;
        //     let rdiv = divs[2] + 1;
        //     target_freq as i32 - (((2 * common::get_mclk_frequency() * qdiv) / pdiv) / rdiv) as i32
        // });

        pac::SYSCTL.syspllcfg0().write(|w| {
            w.set_rdivclk2x(mspm0_metapac::sysctl::vals::Rdivclk2x::from_bits(pll_config.rdiv2x));
        });
        pac::SYSCTL.syspllcfg1().write(|w| {
            w.set_pdiv(mspm0_metapac::sysctl::vals::Pdiv::from_bits(pll_config.pdiv));
            w.set_qdiv(mspm0_metapac::sysctl::vals::Qdiv(pll_config.qdiv));
        });

        pac::SYSCTL.syspllcfg0().write(|w| {
            w.set_syspllref(mspm0_metapac::sysctl::vals::Syspllref::SYSOSC);
            w.set_enableclk2x(true);
            w.set_enableclk1(true);
            w.set_mclk2xvco(true);
        });

        pac::SYSCTL.hsclken().write(|w| {
            w.set_syspllen(true);
        });
    }
}
