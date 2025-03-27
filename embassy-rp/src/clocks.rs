//! Clock configuration for the RP2040

#[cfg(feature = "rp2040")]
use core::arch::asm;
use core::marker::PhantomData;
#[cfg(feature = "rp2040")]
use core::sync::atomic::AtomicU16;
use core::sync::atomic::{AtomicU32, Ordering};

use pac::clocks::vals::*;

use crate::gpio::{AnyPin, SealedPin};
#[cfg(feature = "rp2040")]
use crate::pac::common::{Reg, RW};
use crate::{pac, reset, Peri};

// NOTE: all gpin handling is commented out for future reference.
// gpin is not usually safe to use during the boot init() call, so it won't
// be very useful until we have runtime clock reconfiguration. once this
// happens we can resurrect the commented-out gpin bits.
struct Clocks {
    xosc: AtomicU32,
    sys: AtomicU32,
    reference: AtomicU32,
    pll_sys: AtomicU32,
    pll_usb: AtomicU32,
    usb: AtomicU32,
    adc: AtomicU32,
    // gpin0: AtomicU32,
    // gpin1: AtomicU32,
    rosc: AtomicU32,
    peri: AtomicU32,
    #[cfg(feature = "rp2040")]
    rtc: AtomicU16,
}

static CLOCKS: Clocks = Clocks {
    xosc: AtomicU32::new(0),
    sys: AtomicU32::new(0),
    reference: AtomicU32::new(0),
    pll_sys: AtomicU32::new(0),
    pll_usb: AtomicU32::new(0),
    usb: AtomicU32::new(0),
    adc: AtomicU32::new(0),
    // gpin0: AtomicU32::new(0),
    // gpin1: AtomicU32::new(0),
    rosc: AtomicU32::new(0),
    peri: AtomicU32::new(0),
    #[cfg(feature = "rp2040")]
    rtc: AtomicU16::new(0),
};

/// Peripheral clock sources.
#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeriClkSrc {
    /// SYS.
    Sys = ClkPeriCtrlAuxsrc::CLK_SYS as _,
    /// PLL SYS.
    PllSys = ClkPeriCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    /// PLL USB.
    PllUsb = ClkPeriCtrlAuxsrc::CLKSRC_PLL_USB as _,
    /// ROSC.
    Rosc = ClkPeriCtrlAuxsrc::ROSC_CLKSRC_PH as _,
    /// XOSC.
    Xosc = ClkPeriCtrlAuxsrc::XOSC_CLKSRC as _,
    // Gpin0 = ClkPeriCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkPeriCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
}

/// CLock configuration.
#[non_exhaustive]
pub struct ClockConfig {
    /// Ring oscillator configuration.
    pub rosc: Option<RoscConfig>,
    /// External oscillator configuration.
    pub xosc: Option<XoscConfig>,
    /// Reference clock configuration.
    pub ref_clk: RefClkConfig,
    /// System clock configuration.
    pub sys_clk: SysClkConfig,
    /// Peripheral clock source configuration.
    pub peri_clk_src: Option<PeriClkSrc>,
    /// USB clock configuration.
    pub usb_clk: Option<UsbClkConfig>,
    /// ADC clock configuration.
    pub adc_clk: Option<AdcClkConfig>,
    /// RTC clock configuration.
    #[cfg(feature = "rp2040")]
    pub rtc_clk: Option<RtcClkConfig>,
    // gpin0: Option<(u32, Gpin<'static, AnyPin>)>,
    // gpin1: Option<(u32, Gpin<'static, AnyPin>)>,
}

impl ClockConfig {
    /// Clock configuration derived from external crystal.
    pub fn crystal(crystal_hz: u32) -> Self {
        Self {
            rosc: Some(RoscConfig {
                hz: 6_500_000,
                range: RoscRange::Medium,
                drive_strength: [0; 8],
                div: 16,
            }),
            xosc: Some(XoscConfig {
                hz: crystal_hz,
                sys_pll: Some(PllConfig {
                    refdiv: 1,
                    fbdiv: 125,
                    #[cfg(feature = "rp2040")]
                    post_div1: 6,
                    #[cfg(feature = "_rp235x")]
                    post_div1: 5,
                    post_div2: 2,
                }),
                usb_pll: Some(PllConfig {
                    refdiv: 1,
                    fbdiv: 120,
                    post_div1: 6,
                    post_div2: 5,
                }),
                delay_multiplier: 128,
            }),
            ref_clk: RefClkConfig {
                src: RefClkSrc::Xosc,
                div: 1,
            },
            sys_clk: SysClkConfig {
                src: SysClkSrc::PllSys,
                div_int: 1,
                div_frac: 0,
            },
            peri_clk_src: Some(PeriClkSrc::Sys),
            // CLK USB = PLL USB (48MHz) / 1 = 48MHz
            usb_clk: Some(UsbClkConfig {
                src: UsbClkSrc::PllUsb,
                div: 1,
                phase: 0,
            }),
            // CLK ADC = PLL USB (48MHZ) / 1 = 48MHz
            adc_clk: Some(AdcClkConfig {
                src: AdcClkSrc::PllUsb,
                div: 1,
                phase: 0,
            }),
            // CLK RTC = PLL USB (48MHz) / 1024 = 46875Hz
            #[cfg(feature = "rp2040")]
            rtc_clk: Some(RtcClkConfig {
                src: RtcClkSrc::PllUsb,
                div_int: 1024,
                div_frac: 0,
                phase: 0,
            }),
            // gpin0: None,
            // gpin1: None,
        }
    }

    /// Clock configuration from internal oscillator.
    pub fn rosc() -> Self {
        Self {
            rosc: Some(RoscConfig {
                hz: 140_000_000,
                range: RoscRange::High,
                drive_strength: [0; 8],
                div: 1,
            }),
            xosc: None,
            ref_clk: RefClkConfig {
                src: RefClkSrc::Rosc,
                div: 1,
            },
            sys_clk: SysClkConfig {
                src: SysClkSrc::Rosc,
                div_int: 1,
                div_frac: 0,
            },
            peri_clk_src: Some(PeriClkSrc::Rosc),
            usb_clk: None,
            // CLK ADC = ROSC (140MHz) / 3 ≅ 48MHz
            adc_clk: Some(AdcClkConfig {
                src: AdcClkSrc::Rosc,
                div: 3,
                phase: 0,
            }),
            // CLK RTC = ROSC (140MHz) / 2986.667969 ≅ 46875Hz
            #[cfg(feature = "rp2040")]
            rtc_clk: Some(RtcClkConfig {
                src: RtcClkSrc::Rosc,
                div_int: 2986,
                div_frac: 171,
                phase: 0,
            }),
            // gpin0: None,
            // gpin1: None,
        }
    }

    // pub fn bind_gpin<P: GpinPin>(&mut self, gpin: Gpin<'static, P>, hz: u32) {
    //     match P::NR {
    //         0 => self.gpin0 = Some((hz, gpin.into())),
    //         1 => self.gpin1 = Some((hz, gpin.into())),
    //         _ => unreachable!(),
    //     }
    //     // pin is now provisionally bound. if the config is applied it must be forgotten,
    //     // or Gpin::drop will deconfigure the clock input.
    // }
}

/// ROSC freq range.
#[repr(u16)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoscRange {
    /// Low range.
    Low = pac::rosc::vals::FreqRange::LOW.0,
    /// Medium range (1.33x low)
    Medium = pac::rosc::vals::FreqRange::MEDIUM.0,
    /// High range (2x low)
    High = pac::rosc::vals::FreqRange::HIGH.0,
    /// Too high. Should not be used.
    TooHigh = pac::rosc::vals::FreqRange::TOOHIGH.0,
}

/// On-chip ring oscillator configuration.
pub struct RoscConfig {
    /// Final frequency of the oscillator, after the divider has been applied.
    /// The oscillator has a nominal frequency of 6.5MHz at medium range with
    /// divider 16 and all drive strengths set to 0, other values should be
    /// measured in situ.
    pub hz: u32,
    /// Oscillator range.
    pub range: RoscRange,
    /// Drive strength for oscillator.
    pub drive_strength: [u8; 8],
    /// Output divider.
    pub div: u16,
}

/// Crystal oscillator configuration.
pub struct XoscConfig {
    /// Final frequency of the oscillator.
    pub hz: u32,
    /// Configuring PLL for the system clock.
    pub sys_pll: Option<PllConfig>,
    /// Configuring PLL for the USB clock.
    pub usb_pll: Option<PllConfig>,
    /// Multiplier for the startup delay.
    pub delay_multiplier: u32,
}

/// PLL configuration.
pub struct PllConfig {
    /// Reference divisor.
    pub refdiv: u8,
    /// Feedback divisor.
    pub fbdiv: u16,
    /// Output divisor 1.
    pub post_div1: u8,
    /// Output divisor 2.
    pub post_div2: u8,
}

/// Reference clock config.
pub struct RefClkConfig {
    /// Reference clock source.
    pub src: RefClkSrc,
    /// Reference clock divider.
    pub div: u8,
}

/// Reference clock source.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefClkSrc {
    /// XOSC.
    Xosc,
    /// ROSC.
    Rosc,
    /// PLL USB.
    PllUsb,
    // Gpin0,
    // Gpin1,
}

/// SYS clock source.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SysClkSrc {
    /// REF.
    Ref,
    /// PLL SYS.
    PllSys,
    /// PLL USB.
    PllUsb,
    /// ROSC.
    Rosc,
    /// XOSC.
    Xosc,
    // Gpin0,
    // Gpin1,
}

/// SYS clock config.
pub struct SysClkConfig {
    /// SYS clock source.
    pub src: SysClkSrc,
    /// SYS clock divider.
    #[cfg(feature = "rp2040")]
    pub div_int: u32,
    /// SYS clock fraction.
    #[cfg(feature = "rp2040")]
    pub div_frac: u8,
    /// SYS clock divider.
    #[cfg(feature = "_rp235x")]
    pub div_int: u16,
    /// SYS clock fraction.
    #[cfg(feature = "_rp235x")]
    pub div_frac: u16,
}

/// USB clock source.
#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsbClkSrc {
    /// PLL USB.
    PllUsb = ClkUsbCtrlAuxsrc::CLKSRC_PLL_USB as _,
    /// PLL SYS.
    PllSys = ClkUsbCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    /// ROSC.
    Rosc = ClkUsbCtrlAuxsrc::ROSC_CLKSRC_PH as _,
    /// XOSC.
    Xosc = ClkUsbCtrlAuxsrc::XOSC_CLKSRC as _,
    // Gpin0 = ClkUsbCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkUsbCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
}

/// USB clock config.
pub struct UsbClkConfig {
    /// USB clock source.
    pub src: UsbClkSrc,
    /// USB clock divider.
    pub div: u8,
    /// USB clock phase.
    pub phase: u8,
}

/// ADC clock source.
#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdcClkSrc {
    /// PLL USB.
    PllUsb = ClkAdcCtrlAuxsrc::CLKSRC_PLL_USB as _,
    /// PLL SYS.
    PllSys = ClkAdcCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    /// ROSC.
    Rosc = ClkAdcCtrlAuxsrc::ROSC_CLKSRC_PH as _,
    /// XOSC.
    Xosc = ClkAdcCtrlAuxsrc::XOSC_CLKSRC as _,
    // Gpin0 = ClkAdcCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkAdcCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
}

/// ADC clock config.
pub struct AdcClkConfig {
    /// ADC clock source.
    pub src: AdcClkSrc,
    /// ADC clock divider.
    pub div: u8,
    /// ADC clock phase.
    pub phase: u8,
}

/// RTC clock source.
#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(feature = "rp2040")]
pub enum RtcClkSrc {
    /// PLL USB.
    PllUsb = ClkRtcCtrlAuxsrc::CLKSRC_PLL_USB as _,
    /// PLL SYS.
    PllSys = ClkRtcCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    /// ROSC.
    Rosc = ClkRtcCtrlAuxsrc::ROSC_CLKSRC_PH as _,
    /// XOSC.
    Xosc = ClkRtcCtrlAuxsrc::XOSC_CLKSRC as _,
    // Gpin0 = ClkRtcCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkRtcCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
}

/// RTC clock config.
#[cfg(feature = "rp2040")]
pub struct RtcClkConfig {
    /// RTC clock source.
    pub src: RtcClkSrc,
    /// RTC clock divider.
    pub div_int: u32,
    /// RTC clock divider fraction.
    pub div_frac: u8,
    /// RTC clock phase.
    pub phase: u8,
}

/// safety: must be called exactly once at bootup
pub(crate) unsafe fn init(config: ClockConfig) {
    // Reset everything except:
    // - QSPI (we're using it to run this code!)
    // - PLLs (it may be suicide if that's what's clocking us)
    // - USB, SYSCFG (breaks usb-to-swd on core1)
    // - RTC (else there would be no more time...)
    let mut peris = reset::ALL_PERIPHERALS;
    peris.set_io_qspi(false);
    // peris.set_io_bank0(false); // might be suicide if we're clocked from gpin
    peris.set_pads_qspi(false);
    peris.set_pll_sys(false);
    peris.set_pll_usb(false);
    peris.set_usbctrl(false);
    peris.set_syscfg(false);
    //peris.set_rtc(false);
    reset::reset(peris);

    // Disable resus that may be enabled from previous software
    let c = pac::CLOCKS;
    c.clk_sys_resus_ctrl()
        .write_value(pac::clocks::regs::ClkSysResusCtrl(0));

    // Before we touch PLLs, switch sys and ref cleanly away from their aux sources.
    c.clk_sys_ctrl().modify(|w| w.set_src(ClkSysCtrlSrc::CLK_REF));
    #[cfg(feature = "rp2040")]
    while c.clk_sys_selected().read() != 1 {}
    #[cfg(feature = "_rp235x")]
    while c.clk_sys_selected().read() != pac::clocks::regs::ClkSysSelected(1) {}
    c.clk_ref_ctrl().modify(|w| w.set_src(ClkRefCtrlSrc::ROSC_CLKSRC_PH));
    #[cfg(feature = "rp2040")]
    while c.clk_ref_selected().read() != 1 {}
    #[cfg(feature = "_rp235x")]
    while c.clk_ref_selected().read() != pac::clocks::regs::ClkRefSelected(1) {}

    // Reset the PLLs
    let mut peris = reset::Peripherals(0);
    peris.set_pll_sys(true);
    peris.set_pll_usb(true);
    reset::reset(peris);
    reset::unreset_wait(peris);

    // let gpin0_freq = config.gpin0.map_or(0, |p| {
    //     core::mem::forget(p.1);
    //     p.0
    // });
    // CLOCKS.gpin0.store(gpin0_freq, Ordering::Relaxed);
    // let gpin1_freq = config.gpin1.map_or(0, |p| {
    //     core::mem::forget(p.1);
    //     p.0
    // });
    // CLOCKS.gpin1.store(gpin1_freq, Ordering::Relaxed);

    let rosc_freq = match config.rosc {
        Some(config) => configure_rosc(config),
        None => 0,
    };
    CLOCKS.rosc.store(rosc_freq, Ordering::Relaxed);

    let (xosc_freq, pll_sys_freq, pll_usb_freq) = match config.xosc {
        Some(config) => {
            // start XOSC
            // datasheet mentions support for clock inputs into XIN, but doesn't go into
            // how this is achieved. pico-sdk doesn't support this at all.
            start_xosc(config.hz, config.delay_multiplier);

            let pll_sys_freq = match config.sys_pll {
                Some(sys_pll_config) => configure_pll(pac::PLL_SYS, config.hz, sys_pll_config),
                None => 0,
            };
            let pll_usb_freq = match config.usb_pll {
                Some(usb_pll_config) => configure_pll(pac::PLL_USB, config.hz, usb_pll_config),
                None => 0,
            };

            (config.hz, pll_sys_freq, pll_usb_freq)
        }
        None => (0, 0, 0),
    };
    CLOCKS.xosc.store(xosc_freq, Ordering::Relaxed);
    CLOCKS.pll_sys.store(pll_sys_freq, Ordering::Relaxed);
    CLOCKS.pll_usb.store(pll_usb_freq, Ordering::Relaxed);

    let (ref_src, ref_aux, clk_ref_freq) = {
        use {ClkRefCtrlAuxsrc as Aux, ClkRefCtrlSrc as Src};
        let div = config.ref_clk.div as u32;
        assert!(div >= 1 && div <= 4);
        match config.ref_clk.src {
            RefClkSrc::Xosc => (Src::XOSC_CLKSRC, Aux::CLKSRC_PLL_USB, xosc_freq / div),
            RefClkSrc::Rosc => (Src::ROSC_CLKSRC_PH, Aux::CLKSRC_PLL_USB, rosc_freq / div),
            RefClkSrc::PllUsb => (Src::CLKSRC_CLK_REF_AUX, Aux::CLKSRC_PLL_USB, pll_usb_freq / div),
            // RefClkSrc::Gpin0 => (Src::CLKSRC_CLK_REF_AUX, Aux::CLKSRC_GPIN0, gpin0_freq / div),
            // RefClkSrc::Gpin1 => (Src::CLKSRC_CLK_REF_AUX, Aux::CLKSRC_GPIN1, gpin1_freq / div),
        }
    };
    assert!(clk_ref_freq != 0);
    CLOCKS.reference.store(clk_ref_freq, Ordering::Relaxed);
    c.clk_ref_ctrl().write(|w| {
        w.set_src(ref_src);
        w.set_auxsrc(ref_aux);
    });
    #[cfg(feature = "rp2040")]
    while c.clk_ref_selected().read() != (1 << ref_src as u32) {}
    #[cfg(feature = "_rp235x")]
    while c.clk_ref_selected().read() != pac::clocks::regs::ClkRefSelected(1 << ref_src as u32) {}
    c.clk_ref_div().write(|w| {
        w.set_int(config.ref_clk.div);
    });

    // Configure tick generation on the 2040.
    #[cfg(feature = "rp2040")]
    pac::WATCHDOG.tick().write(|w| {
        w.set_cycles((clk_ref_freq / 1_000_000) as u16);
        w.set_enable(true);
    });
    // Configure tick generator on the 2350
    #[cfg(feature = "_rp235x")]
    {
        let cycle_count = clk_ref_freq / 1_000_000;

        pac::TICKS.timer0_cycles().write(|w| w.0 = cycle_count);
        pac::TICKS.timer0_ctrl().write(|w| w.set_enable(true));

        pac::TICKS.watchdog_cycles().write(|w| w.0 = cycle_count);
        pac::TICKS.watchdog_ctrl().write(|w| w.set_enable(true));
    }

    let (sys_src, sys_aux, clk_sys_freq) = {
        use {ClkSysCtrlAuxsrc as Aux, ClkSysCtrlSrc as Src};
        let (src, aux, freq) = match config.sys_clk.src {
            SysClkSrc::Ref => (Src::CLK_REF, Aux::CLKSRC_PLL_SYS, clk_ref_freq),
            SysClkSrc::PllSys => (Src::CLKSRC_CLK_SYS_AUX, Aux::CLKSRC_PLL_SYS, pll_sys_freq),
            SysClkSrc::PllUsb => (Src::CLKSRC_CLK_SYS_AUX, Aux::CLKSRC_PLL_USB, pll_usb_freq),
            SysClkSrc::Rosc => (Src::CLKSRC_CLK_SYS_AUX, Aux::ROSC_CLKSRC, rosc_freq),
            SysClkSrc::Xosc => (Src::CLKSRC_CLK_SYS_AUX, Aux::XOSC_CLKSRC, xosc_freq),
            // SysClkSrc::Gpin0 => (Src::CLKSRC_CLK_SYS_AUX, Aux::CLKSRC_GPIN0, gpin0_freq),
            // SysClkSrc::Gpin1 => (Src::CLKSRC_CLK_SYS_AUX, Aux::CLKSRC_GPIN1, gpin1_freq),
        };
        let div = config.sys_clk.div_int as u64 * 256 + config.sys_clk.div_frac as u64;
        (src, aux, ((freq as u64 * 256) / div) as u32)
    };
    assert!(clk_sys_freq != 0);
    CLOCKS.sys.store(clk_sys_freq, Ordering::Relaxed);
    if sys_src != ClkSysCtrlSrc::CLK_REF {
        c.clk_sys_ctrl().write(|w| w.set_src(ClkSysCtrlSrc::CLK_REF));
        #[cfg(feature = "rp2040")]
        while c.clk_sys_selected().read() != (1 << ClkSysCtrlSrc::CLK_REF as u32) {}
        #[cfg(feature = "_rp235x")]
        while c.clk_sys_selected().read() != pac::clocks::regs::ClkSysSelected(1 << ClkSysCtrlSrc::CLK_REF as u32) {}
    }
    c.clk_sys_ctrl().write(|w| {
        w.set_auxsrc(sys_aux);
        w.set_src(sys_src);
    });

    #[cfg(feature = "rp2040")]
    while c.clk_sys_selected().read() != (1 << sys_src as u32) {}
    #[cfg(feature = "_rp235x")]
    while c.clk_sys_selected().read() != pac::clocks::regs::ClkSysSelected(1 << sys_src as u32) {}

    c.clk_sys_div().write(|w| {
        w.set_int(config.sys_clk.div_int);
        w.set_frac(config.sys_clk.div_frac);
    });

    let mut peris = reset::ALL_PERIPHERALS;

    if let Some(src) = config.peri_clk_src {
        c.clk_peri_ctrl().write(|w| {
            w.set_enable(true);
            w.set_auxsrc(ClkPeriCtrlAuxsrc::from_bits(src as _));
        });
        let peri_freq = match src {
            PeriClkSrc::Sys => clk_sys_freq,
            PeriClkSrc::PllSys => pll_sys_freq,
            PeriClkSrc::PllUsb => pll_usb_freq,
            PeriClkSrc::Rosc => rosc_freq,
            PeriClkSrc::Xosc => xosc_freq,
            // PeriClkSrc::Gpin0 => gpin0_freq,
            // PeriClkSrc::Gpin1 => gpin1_freq,
        };
        assert!(peri_freq != 0);
        CLOCKS.peri.store(peri_freq, Ordering::Relaxed);
    } else {
        peris.set_spi0(false);
        peris.set_spi1(false);
        peris.set_uart0(false);
        peris.set_uart1(false);
        CLOCKS.peri.store(0, Ordering::Relaxed);
    }

    if let Some(conf) = config.usb_clk {
        c.clk_usb_div().write(|w| w.set_int(conf.div));
        c.clk_usb_ctrl().write(|w| {
            w.set_phase(conf.phase);
            w.set_enable(true);
            w.set_auxsrc(ClkUsbCtrlAuxsrc::from_bits(conf.src as _));
        });
        let usb_freq = match conf.src {
            UsbClkSrc::PllUsb => pll_usb_freq,
            UsbClkSrc::PllSys => pll_sys_freq,
            UsbClkSrc::Rosc => rosc_freq,
            UsbClkSrc::Xosc => xosc_freq,
            // UsbClkSrc::Gpin0 => gpin0_freq,
            // UsbClkSrc::Gpin1 => gpin1_freq,
        };
        assert!(usb_freq != 0);
        assert!(conf.div >= 1 && conf.div <= 4);
        CLOCKS.usb.store(usb_freq / conf.div as u32, Ordering::Relaxed);
    } else {
        peris.set_usbctrl(false);
        CLOCKS.usb.store(0, Ordering::Relaxed);
    }

    if let Some(conf) = config.adc_clk {
        c.clk_adc_div().write(|w| w.set_int(conf.div));
        c.clk_adc_ctrl().write(|w| {
            w.set_phase(conf.phase);
            w.set_enable(true);
            w.set_auxsrc(ClkAdcCtrlAuxsrc::from_bits(conf.src as _));
        });
        let adc_in_freq = match conf.src {
            AdcClkSrc::PllUsb => pll_usb_freq,
            AdcClkSrc::PllSys => pll_sys_freq,
            AdcClkSrc::Rosc => rosc_freq,
            AdcClkSrc::Xosc => xosc_freq,
            // AdcClkSrc::Gpin0 => gpin0_freq,
            // AdcClkSrc::Gpin1 => gpin1_freq,
        };
        assert!(adc_in_freq != 0);
        assert!(conf.div >= 1 && conf.div <= 4);
        CLOCKS.adc.store(adc_in_freq / conf.div as u32, Ordering::Relaxed);
    } else {
        peris.set_adc(false);
        CLOCKS.adc.store(0, Ordering::Relaxed);
    }

    // rp2040 specific clocks
    #[cfg(feature = "rp2040")]
    if let Some(conf) = config.rtc_clk {
        c.clk_rtc_div().write(|w| {
            w.set_int(conf.div_int);
            w.set_frac(conf.div_frac);
        });
        c.clk_rtc_ctrl().write(|w| {
            w.set_phase(conf.phase);
            w.set_enable(true);
            w.set_auxsrc(ClkRtcCtrlAuxsrc::from_bits(conf.src as _));
        });
        let rtc_in_freq = match conf.src {
            RtcClkSrc::PllUsb => pll_usb_freq,
            RtcClkSrc::PllSys => pll_sys_freq,
            RtcClkSrc::Rosc => rosc_freq,
            RtcClkSrc::Xosc => xosc_freq,
            // RtcClkSrc::Gpin0 => gpin0_freq,
            // RtcClkSrc::Gpin1 => gpin1_freq,
        };
        assert!(rtc_in_freq != 0);
        assert!(config.sys_clk.div_int <= 0x1000000);
        CLOCKS.rtc.store(
            ((rtc_in_freq as u64 * 256) / (conf.div_int as u64 * 256 + conf.div_frac as u64)) as u16,
            Ordering::Relaxed,
        );
    } else {
        peris.set_rtc(false);
        CLOCKS.rtc.store(0, Ordering::Relaxed);
    }

    // rp235x specific clocks
    #[cfg(feature = "_rp235x")]
    {
        // TODO hstx clock
        peris.set_hstx(false);
    }

    // Peripheral clocks should now all be running
    reset::unreset_wait(peris);
}

fn configure_rosc(config: RoscConfig) -> u32 {
    let p = pac::ROSC;

    p.freqa().write(|w| {
        w.set_passwd(pac::rosc::vals::Passwd::PASS);
        w.set_ds0(config.drive_strength[0]);
        w.set_ds1(config.drive_strength[1]);
        w.set_ds2(config.drive_strength[2]);
        w.set_ds3(config.drive_strength[3]);
    });

    p.freqb().write(|w| {
        w.set_passwd(pac::rosc::vals::Passwd::PASS);
        w.set_ds4(config.drive_strength[4]);
        w.set_ds5(config.drive_strength[5]);
        w.set_ds6(config.drive_strength[6]);
        w.set_ds7(config.drive_strength[7]);
    });

    p.div().write(|w| {
        w.set_div(pac::rosc::vals::Div(config.div + pac::rosc::vals::Div::PASS.0));
    });

    p.ctrl().write(|w| {
        w.set_enable(pac::rosc::vals::Enable::ENABLE);
        w.set_freq_range(pac::rosc::vals::FreqRange(config.range as u16));
    });

    config.hz
}

/// ROSC clock frequency.
pub fn rosc_freq() -> u32 {
    CLOCKS.rosc.load(Ordering::Relaxed)
}

/// XOSC clock frequency.
pub fn xosc_freq() -> u32 {
    CLOCKS.xosc.load(Ordering::Relaxed)
}

// pub fn gpin0_freq() -> u32 {
//     CLOCKS.gpin0.load(Ordering::Relaxed)
// }
// pub fn gpin1_freq() -> u32 {
//     CLOCKS.gpin1.load(Ordering::Relaxed)
// }

/// PLL SYS clock frequency.
pub fn pll_sys_freq() -> u32 {
    CLOCKS.pll_sys.load(Ordering::Relaxed)
}

/// PLL USB clock frequency.
pub fn pll_usb_freq() -> u32 {
    CLOCKS.pll_usb.load(Ordering::Relaxed)
}

/// SYS clock frequency.
pub fn clk_sys_freq() -> u32 {
    CLOCKS.sys.load(Ordering::Relaxed)
}

/// REF clock frequency.
pub fn clk_ref_freq() -> u32 {
    CLOCKS.reference.load(Ordering::Relaxed)
}

/// Peripheral clock frequency.
pub fn clk_peri_freq() -> u32 {
    CLOCKS.peri.load(Ordering::Relaxed)
}

/// USB clock frequency.
pub fn clk_usb_freq() -> u32 {
    CLOCKS.usb.load(Ordering::Relaxed)
}

/// ADC clock frequency.
pub fn clk_adc_freq() -> u32 {
    CLOCKS.adc.load(Ordering::Relaxed)
}

/// RTC clock frequency.
#[cfg(feature = "rp2040")]
pub fn clk_rtc_freq() -> u16 {
    CLOCKS.rtc.load(Ordering::Relaxed)
}

fn start_xosc(crystal_hz: u32, delay_multiplier: u32) {
    let startup_delay = (((crystal_hz / 1000) * delay_multiplier) + 128) / 256;
    pac::XOSC.startup().write(|w| w.set_delay(startup_delay as u16));
    pac::XOSC.ctrl().write(|w| {
        w.set_freq_range(pac::xosc::vals::CtrlFreqRange::_1_15MHZ);
        w.set_enable(pac::xosc::vals::Enable::ENABLE);
    });
    while !pac::XOSC.status().read().stable() {}
}

#[inline(always)]
fn configure_pll(p: pac::pll::Pll, input_freq: u32, config: PllConfig) -> u32 {
    let ref_freq = input_freq / config.refdiv as u32;
    assert!(config.fbdiv >= 16 && config.fbdiv <= 320);
    assert!(config.post_div1 >= 1 && config.post_div1 <= 7);
    assert!(config.post_div2 >= 1 && config.post_div2 <= 7);
    assert!(config.refdiv >= 1 && config.refdiv <= 63);
    assert!(ref_freq >= 5_000_000 && ref_freq <= 800_000_000);
    let vco_freq = ref_freq.saturating_mul(config.fbdiv as u32);
    assert!(vco_freq >= 750_000_000 && vco_freq <= 1_800_000_000);

    // Load VCO-related dividers before starting VCO
    p.cs().write(|w| w.set_refdiv(config.refdiv as _));
    p.fbdiv_int().write(|w| w.set_fbdiv_int(config.fbdiv));

    // Turn on PLL
    let pwr = p.pwr().write(|w| {
        w.set_dsmpd(true); // "nothing is achieved by setting this low"
        w.set_pd(false);
        w.set_vcopd(false);
        w.set_postdivpd(true);
        *w
    });

    // Wait for PLL to lock
    while !p.cs().read().lock() {}

    // Set post-dividers
    p.prim().write(|w| {
        w.set_postdiv1(config.post_div1);
        w.set_postdiv2(config.post_div2);
    });

    // Turn on post divider
    p.pwr().write(|w| {
        *w = pwr;
        w.set_postdivpd(false);
    });

    vco_freq / ((config.post_div1 * config.post_div2) as u32)
}

/// General purpose input clock pin.
pub trait GpinPin: crate::gpio::Pin {
    /// Pin number.
    const NR: usize;
}

macro_rules! impl_gpinpin {
    ($name:ident, $pin_num:expr, $gpin_num:expr) => {
        impl GpinPin for crate::peripherals::$name {
            const NR: usize = $gpin_num;
        }
    };
}

impl_gpinpin!(PIN_20, 20, 0);
impl_gpinpin!(PIN_22, 22, 1);

/// General purpose clock input driver.
pub struct Gpin<'d, T: GpinPin> {
    gpin: Peri<'d, AnyPin>,
    _phantom: PhantomData<T>,
}

impl<'d, T: GpinPin> Gpin<'d, T> {
    /// Create new gpin driver.
    pub fn new(gpin: Peri<'d, T>) -> Self {
        #[cfg(feature = "rp2040")]
        gpin.gpio().ctrl().write(|w| w.set_funcsel(0x08));

        // On RP2350 GPIN changed from F8 toF9
        #[cfg(feature = "_rp235x")]
        gpin.gpio().ctrl().write(|w| w.set_funcsel(0x09));

        #[cfg(feature = "_rp235x")]
        gpin.pad_ctrl().write(|w| {
            w.set_iso(false);
        });

        Gpin {
            gpin: gpin.into(),
            _phantom: PhantomData,
        }
    }
}

impl<'d, T: GpinPin> Drop for Gpin<'d, T> {
    fn drop(&mut self) {
        self.gpin.pad_ctrl().write(|_| {});
        self.gpin
            .gpio()
            .ctrl()
            .write(|w| w.set_funcsel(pac::io::vals::Gpio0ctrlFuncsel::NULL as _));
    }
}

/// General purpose clock output pin.
pub trait GpoutPin: crate::gpio::Pin {
    /// Pin number.
    fn number(&self) -> usize;
}

macro_rules! impl_gpoutpin {
    ($name:ident, $gpout_num:expr) => {
        impl GpoutPin for crate::peripherals::$name {
            fn number(&self) -> usize {
                $gpout_num
            }
        }
    };
}

impl_gpoutpin!(PIN_21, 0);
impl_gpoutpin!(PIN_23, 1);
impl_gpoutpin!(PIN_24, 2);
impl_gpoutpin!(PIN_25, 3);

/// Gpout clock source.
#[repr(u8)]
pub enum GpoutSrc {
    /// Sys PLL.
    PllSys = ClkGpoutCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    // Gpin0 = ClkGpoutCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkGpoutCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
    /// USB PLL.
    PllUsb = ClkGpoutCtrlAuxsrc::CLKSRC_PLL_USB as _,
    /// ROSC.
    Rosc = ClkGpoutCtrlAuxsrc::ROSC_CLKSRC as _,
    /// XOSC.
    Xosc = ClkGpoutCtrlAuxsrc::XOSC_CLKSRC as _,
    /// SYS.
    Sys = ClkGpoutCtrlAuxsrc::CLK_SYS as _,
    /// USB.
    Usb = ClkGpoutCtrlAuxsrc::CLK_USB as _,
    /// ADC.
    Adc = ClkGpoutCtrlAuxsrc::CLK_ADC as _,
    /// RTC.
    #[cfg(feature = "rp2040")]
    Rtc = ClkGpoutCtrlAuxsrc::CLK_RTC as _,
    /// REF.
    Ref = ClkGpoutCtrlAuxsrc::CLK_REF as _,
}

/// General purpose clock output driver.
pub struct Gpout<'d, T: GpoutPin> {
    gpout: Peri<'d, T>,
}

impl<'d, T: GpoutPin> Gpout<'d, T> {
    /// Create new general purpose clock output.
    pub fn new(gpout: Peri<'d, T>) -> Self {
        #[cfg(feature = "rp2040")]
        gpout.gpio().ctrl().write(|w| w.set_funcsel(0x08));

        // On RP2350 GPOUT changed from F8 toF9
        #[cfg(feature = "_rp235x")]
        gpout.gpio().ctrl().write(|w| w.set_funcsel(0x09));

        #[cfg(feature = "_rp235x")]
        gpout.pad_ctrl().write(|w| {
            w.set_iso(false);
        });

        Self { gpout }
    }

    /// Set clock divider.
    #[cfg(feature = "rp2040")]
    pub fn set_div(&self, int: u32, frac: u8) {
        let c = pac::CLOCKS;
        c.clk_gpout_div(self.gpout.number()).write(|w| {
            w.set_int(int);
            w.set_frac(frac);
        });
    }

    /// Set clock divider.
    #[cfg(feature = "_rp235x")]
    pub fn set_div(&self, int: u16, frac: u16) {
        let c = pac::CLOCKS;
        c.clk_gpout_div(self.gpout.number()).write(|w| {
            w.set_int(int);
            w.set_frac(frac);
        });
    }

    /// Set clock source.
    pub fn set_src(&self, src: GpoutSrc) {
        let c = pac::CLOCKS;
        c.clk_gpout_ctrl(self.gpout.number()).modify(|w| {
            w.set_auxsrc(ClkGpoutCtrlAuxsrc::from_bits(src as _));
        });
    }

    /// Enable clock.
    pub fn enable(&self) {
        let c = pac::CLOCKS;
        c.clk_gpout_ctrl(self.gpout.number()).modify(|w| {
            w.set_enable(true);
        });
    }

    /// Disable clock.
    pub fn disable(&self) {
        let c = pac::CLOCKS;
        c.clk_gpout_ctrl(self.gpout.number()).modify(|w| {
            w.set_enable(false);
        });
    }

    /// Clock frequency.
    pub fn get_freq(&self) -> u32 {
        let c = pac::CLOCKS;
        let src = c.clk_gpout_ctrl(self.gpout.number()).read().auxsrc();

        let base = match src {
            ClkGpoutCtrlAuxsrc::CLKSRC_PLL_SYS => pll_sys_freq(),
            // ClkGpoutCtrlAuxsrc::CLKSRC_GPIN0 => gpin0_freq(),
            // ClkGpoutCtrlAuxsrc::CLKSRC_GPIN1 => gpin1_freq(),
            ClkGpoutCtrlAuxsrc::CLKSRC_PLL_USB => pll_usb_freq(),
            ClkGpoutCtrlAuxsrc::ROSC_CLKSRC => rosc_freq(),
            ClkGpoutCtrlAuxsrc::XOSC_CLKSRC => xosc_freq(),
            ClkGpoutCtrlAuxsrc::CLK_SYS => clk_sys_freq(),
            ClkGpoutCtrlAuxsrc::CLK_USB => clk_usb_freq(),
            ClkGpoutCtrlAuxsrc::CLK_ADC => clk_adc_freq(),
            //ClkGpoutCtrlAuxsrc::CLK_RTC => clk_rtc_freq() as _,
            ClkGpoutCtrlAuxsrc::CLK_REF => clk_ref_freq(),
            _ => unreachable!(),
        };

        let div = c.clk_gpout_div(self.gpout.number()).read();
        let int = if div.int() == 0 { 0xFFFF } else { div.int() } as u64;
        let frac = div.frac() as u64;

        ((base as u64 * 256) / (int * 256 + frac)) as u32
    }
}

impl<'d, T: GpoutPin> Drop for Gpout<'d, T> {
    fn drop(&mut self) {
        self.disable();
        self.gpout.pad_ctrl().write(|_| {});
        self.gpout
            .gpio()
            .ctrl()
            .write(|w| w.set_funcsel(pac::io::vals::Gpio0ctrlFuncsel::NULL as _));
    }
}

/// Random number generator based on the ROSC RANDOMBIT register.
///
/// This will not produce random values if the ROSC is stopped or run at some
/// harmonic of the bus frequency. With default clock settings these are not
/// issues.
pub struct RoscRng;

impl RoscRng {
    fn next_u8() -> u8 {
        let random_reg = pac::ROSC.randombit();
        let mut acc = 0;
        for _ in 0..u8::BITS {
            acc <<= 1;
            acc |= random_reg.read().randombit() as u8;
        }
        acc
    }
}

impl rand_core::RngCore for RoscRng {
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        Ok(self.fill_bytes(dest))
    }

    fn next_u32(&mut self) -> u32 {
        rand_core::impls::next_u32_via_fill(self)
    }

    fn next_u64(&mut self) -> u64 {
        rand_core::impls::next_u64_via_fill(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill_with(Self::next_u8)
    }
}
/// Enter the `DORMANT` sleep state. This will stop *all* internal clocks
/// and can only be exited through resets, dormant-wake GPIO interrupts,
/// and RTC interrupts. If RTC is clocked from an internal clock source
/// it will be stopped and not function as a wakeup source.
#[cfg(all(target_arch = "arm", feature = "rp2040"))]
pub fn dormant_sleep() {
    struct Set<T: Copy, F: Fn()>(Reg<T, RW>, T, F);

    impl<T: Copy, F: Fn()> Drop for Set<T, F> {
        fn drop(&mut self) {
            self.0.write_value(self.1);
            self.2();
        }
    }

    fn set_with_post_restore<T: Copy, After: Fn(), F: FnOnce(&mut T) -> After>(
        reg: Reg<T, RW>,
        f: F,
    ) -> Set<T, impl Fn()> {
        reg.modify(|w| {
            let old = *w;
            let after = f(w);
            Set(reg, old, after)
        })
    }

    fn set<T: Copy, F: FnOnce(&mut T)>(reg: Reg<T, RW>, f: F) -> Set<T, impl Fn()> {
        set_with_post_restore(reg, |r| {
            f(r);
            || ()
        })
    }

    // disable all clocks that are not vital in preparation for disabling clock sources.
    // we'll keep gpout and rtc clocks untouched, gpout because we don't care about them
    // and rtc because it's a possible wakeup source. if clk_rtc is not configured for
    // gpin we'll never wake from rtc, but that's what the user asked for then.
    let _stop_adc = set(pac::CLOCKS.clk_adc_ctrl(), |w| w.set_enable(false));
    let _stop_usb = set(pac::CLOCKS.clk_usb_ctrl(), |w| w.set_enable(false));
    let _stop_peri = set(pac::CLOCKS.clk_peri_ctrl(), |w| w.set_enable(false));
    // set up rosc. we could ask the user to tell us which clock source to wake from like
    // the C SDK does, but that seems rather unfriendly. we *may* disturb rtc by changing
    // rosc configuration if it's currently the rtc clock source, so we'll configure rosc
    // to the slowest frequency to minimize that impact.
    let _configure_rosc = (
        set(pac::ROSC.ctrl(), |w| {
            w.set_enable(pac::rosc::vals::Enable::ENABLE);
            w.set_freq_range(pac::rosc::vals::FreqRange::LOW);
        }),
        // div=32
        set(pac::ROSC.div(), |w| w.set_div(pac::rosc::vals::Div(0xaa0))),
    );
    while !pac::ROSC.status().read().stable() {}
    // switch over to rosc as the system clock source. this will change clock sources for
    // watchdog and timer clocks, but timers won't be a concern and the watchdog won't
    // speed up by enough to worry about (unless it's clocked from gpin, which we don't
    // support anyway).
    let _switch_clk_ref = set(pac::CLOCKS.clk_ref_ctrl(), |w| {
        w.set_src(pac::clocks::vals::ClkRefCtrlSrc::ROSC_CLKSRC_PH);
    });
    let _switch_clk_sys = set(pac::CLOCKS.clk_sys_ctrl(), |w| {
        w.set_src(pac::clocks::vals::ClkSysCtrlSrc::CLK_REF);
    });
    // oscillator dormancy does not power down plls, we have to do that ourselves. we'll
    // restore them to their prior glory when woken though since the system may be clocked
    // from either (and usb/adc will probably need the USB PLL anyway)
    let _stop_pll_sys = set_with_post_restore(pac::PLL_SYS.pwr(), |w| {
        let wake = !w.pd() && !w.vcopd();
        w.set_pd(true);
        w.set_vcopd(true);
        move || while wake && !pac::PLL_SYS.cs().read().lock() {}
    });
    let _stop_pll_usb = set_with_post_restore(pac::PLL_USB.pwr(), |w| {
        let wake = !w.pd() && !w.vcopd();
        w.set_pd(true);
        w.set_vcopd(true);
        move || while wake && !pac::PLL_USB.cs().read().lock() {}
    });
    // dormancy only stops the oscillator we're telling to go dormant, the other remains
    // running. nothing can use xosc at this point any more. not doing this costs an 200µA.
    let _stop_xosc = set_with_post_restore(pac::XOSC.ctrl(), |w| {
        let wake = w.enable() == pac::xosc::vals::Enable::ENABLE;
        if wake {
            w.set_enable(pac::xosc::vals::Enable::DISABLE);
        }
        move || while wake && !pac::XOSC.status().read().stable() {}
    });
    let _power_down_xip_cache = set(pac::XIP_CTRL.ctrl(), |w| w.set_power_down(true));

    // only power down memory if we're running from XIP (or ROM? how?).
    // powering down memory otherwise would require a lot of exacting checks that
    // are better done by the user in a local copy of this function.
    // powering down memories saves ~100µA, so it's well worth doing.
    unsafe {
        let is_in_flash = {
            // we can't rely on the address of this function as rust sees it since linker
            // magic or even boot2 may place it into ram.
            let pc: usize;
            asm!(
                "mov {pc}, pc",
                pc = out (reg) pc
            );
            pc < 0x20000000
        };
        if is_in_flash {
            // we will be powering down memories, so we must be *absolutely*
            // certain that we're running entirely from XIP and registers until
            // memories are powered back up again. accessing memory that's powered
            // down may corrupt memory contents (see section 2.11.4 of the manual).
            // additionally a 20ns wait time is needed after powering up memories
            // again. rosc is likely to run at only a few MHz at most, so the
            // inter-instruction delay alone will be enough to satisfy this bound.
            asm!(
                "ldr {old_mem}, [{mempowerdown}]",
                "str {power_down_mems}, [{mempowerdown}]",
                "str {coma}, [{dormant}]",
                "str {old_mem}, [{mempowerdown}]",
                old_mem = out (reg) _,
                mempowerdown = in (reg) pac::SYSCFG.mempowerdown().as_ptr(),
                power_down_mems = in (reg) 0b11111111,
                dormant = in (reg) pac::ROSC.dormant().as_ptr(),
                coma = in (reg) 0x636f6d61,
            );
        } else {
            pac::ROSC.dormant().write_value(rp_pac::rosc::regs::Dormant(0x636f6d61));
        }
    }
}
