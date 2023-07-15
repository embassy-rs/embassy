use core::marker::PhantomData;
use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};

use embassy_hal_common::{into_ref, PeripheralRef};
use pac::clocks::vals::*;

use crate::gpio::sealed::Pin;
use crate::gpio::AnyPin;
use crate::{pac, reset, Peripheral};

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
    rtc: AtomicU16::new(0),
};

#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeriClkSrc {
    Sys = ClkPeriCtrlAuxsrc::CLK_SYS as _,
    PllSys = ClkPeriCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    PllUsb = ClkPeriCtrlAuxsrc::CLKSRC_PLL_USB as _,
    Rosc = ClkPeriCtrlAuxsrc::ROSC_CLKSRC_PH as _,
    Xosc = ClkPeriCtrlAuxsrc::XOSC_CLKSRC as _,
    // Gpin0 = ClkPeriCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkPeriCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
}

#[non_exhaustive]
pub struct ClockConfig {
    pub rosc: Option<RoscConfig>,
    pub xosc: Option<XoscConfig>,
    pub ref_clk: RefClkConfig,
    pub sys_clk: SysClkConfig,
    pub peri_clk_src: Option<PeriClkSrc>,
    pub usb_clk: Option<UsbClkConfig>,
    pub adc_clk: Option<AdcClkConfig>,
    pub rtc_clk: Option<RtcClkConfig>,
    // gpin0: Option<(u32, Gpin<'static, AnyPin>)>,
    // gpin1: Option<(u32, Gpin<'static, AnyPin>)>,
}

impl ClockConfig {
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
                    post_div1: 6,
                    post_div2: 2,
                }),
                usb_pll: Some(PllConfig {
                    refdiv: 1,
                    fbdiv: 120,
                    post_div1: 6,
                    post_div2: 5,
                }),
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
    //         0 => self.gpin0 = Some((hz, gpin.map_into())),
    //         1 => self.gpin1 = Some((hz, gpin.map_into())),
    //         _ => unreachable!(),
    //     }
    //     // pin is now provisionally bound. if the config is applied it must be forgotten,
    //     // or Gpin::drop will deconfigure the clock input.
    // }
}

#[repr(u16)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoscRange {
    Low = pac::rosc::vals::FreqRange::LOW.0,
    Medium = pac::rosc::vals::FreqRange::MEDIUM.0,
    High = pac::rosc::vals::FreqRange::HIGH.0,
    TooHigh = pac::rosc::vals::FreqRange::TOOHIGH.0,
}

pub struct RoscConfig {
    /// Final frequency of the oscillator, after the divider has been applied.
    /// The oscillator has a nominal frequency of 6.5MHz at medium range with
    /// divider 16 and all drive strengths set to 0, other values should be
    /// measured in situ.
    pub hz: u32,
    pub range: RoscRange,
    pub drive_strength: [u8; 8],
    pub div: u16,
}

pub struct XoscConfig {
    pub hz: u32,
    pub sys_pll: Option<PllConfig>,
    pub usb_pll: Option<PllConfig>,
}

pub struct PllConfig {
    pub refdiv: u8,
    pub fbdiv: u16,
    pub post_div1: u8,
    pub post_div2: u8,
}

pub struct RefClkConfig {
    pub src: RefClkSrc,
    pub div: u8,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefClkSrc {
    // main sources
    Xosc,
    Rosc,
    // aux sources
    PllUsb,
    // Gpin0,
    // Gpin1,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SysClkSrc {
    // main sources
    Ref,
    // aux sources
    PllSys,
    PllUsb,
    Rosc,
    Xosc,
    // Gpin0,
    // Gpin1,
}

pub struct SysClkConfig {
    pub src: SysClkSrc,
    pub div_int: u32,
    pub div_frac: u8,
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsbClkSrc {
    PllUsb = ClkUsbCtrlAuxsrc::CLKSRC_PLL_USB as _,
    PllSys = ClkUsbCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    Rosc = ClkUsbCtrlAuxsrc::ROSC_CLKSRC_PH as _,
    Xosc = ClkUsbCtrlAuxsrc::XOSC_CLKSRC as _,
    // Gpin0 = ClkUsbCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkUsbCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
}

pub struct UsbClkConfig {
    pub src: UsbClkSrc,
    pub div: u8,
    pub phase: u8,
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdcClkSrc {
    PllUsb = ClkAdcCtrlAuxsrc::CLKSRC_PLL_USB as _,
    PllSys = ClkAdcCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    Rosc = ClkAdcCtrlAuxsrc::ROSC_CLKSRC_PH as _,
    Xosc = ClkAdcCtrlAuxsrc::XOSC_CLKSRC as _,
    // Gpin0 = ClkAdcCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkAdcCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
}

pub struct AdcClkConfig {
    pub src: AdcClkSrc,
    pub div: u8,
    pub phase: u8,
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RtcClkSrc {
    PllUsb = ClkRtcCtrlAuxsrc::CLKSRC_PLL_USB as _,
    PllSys = ClkRtcCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    Rosc = ClkRtcCtrlAuxsrc::ROSC_CLKSRC_PH as _,
    Xosc = ClkRtcCtrlAuxsrc::XOSC_CLKSRC as _,
    // Gpin0 = ClkRtcCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkRtcCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
}

pub struct RtcClkConfig {
    pub src: RtcClkSrc,
    pub div_int: u32,
    pub div_frac: u8,
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
    // TODO investigate if usb should be unreset here
    peris.set_usbctrl(false);
    peris.set_syscfg(false);
    peris.set_rtc(false);
    reset::reset(peris);

    // Disable resus that may be enabled from previous software
    let c = pac::CLOCKS;
    c.clk_sys_resus_ctrl()
        .write_value(pac::clocks::regs::ClkSysResusCtrl(0));

    // Before we touch PLLs, switch sys and ref cleanly away from their aux sources.
    c.clk_sys_ctrl().modify(|w| w.set_src(ClkSysCtrlSrc::CLK_REF));
    while c.clk_sys_selected().read() != 1 {}
    c.clk_ref_ctrl().modify(|w| w.set_src(ClkRefCtrlSrc::ROSC_CLKSRC_PH));
    while c.clk_ref_selected().read() != 1 {}

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
            start_xosc(config.hz);

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
    while c.clk_ref_selected().read() != 1 << ref_src as u32 {}
    c.clk_ref_div().write(|w| {
        w.set_int(config.ref_clk.div);
    });

    pac::WATCHDOG.tick().write(|w| {
        w.set_cycles((clk_ref_freq / 1_000_000) as u16);
        w.set_enable(true);
    });

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
        assert!(config.sys_clk.div_int <= 0x1000000);
        let div = config.sys_clk.div_int as u64 * 256 + config.sys_clk.div_frac as u64;
        (src, aux, ((freq as u64 * 256) / div) as u32)
    };
    assert!(clk_sys_freq != 0);
    CLOCKS.sys.store(clk_sys_freq, Ordering::Relaxed);
    if sys_src != ClkSysCtrlSrc::CLK_REF {
        c.clk_sys_ctrl().write(|w| w.set_src(ClkSysCtrlSrc::CLK_REF));
        while c.clk_sys_selected().read() != 1 << ClkSysCtrlSrc::CLK_REF as u32 {}
    }
    c.clk_sys_ctrl().write(|w| {
        w.set_auxsrc(sys_aux);
        w.set_src(sys_src);
    });
    while c.clk_sys_selected().read() != 1 << sys_src as u32 {}
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

pub fn rosc_freq() -> u32 {
    CLOCKS.rosc.load(Ordering::Relaxed)
}

pub fn xosc_freq() -> u32 {
    CLOCKS.xosc.load(Ordering::Relaxed)
}

// pub fn gpin0_freq() -> u32 {
//     CLOCKS.gpin0.load(Ordering::Relaxed)
// }
// pub fn gpin1_freq() -> u32 {
//     CLOCKS.gpin1.load(Ordering::Relaxed)
// }

pub fn pll_sys_freq() -> u32 {
    CLOCKS.pll_sys.load(Ordering::Relaxed)
}

pub fn pll_usb_freq() -> u32 {
    CLOCKS.pll_usb.load(Ordering::Relaxed)
}

pub fn clk_sys_freq() -> u32 {
    CLOCKS.sys.load(Ordering::Relaxed)
}

pub fn clk_ref_freq() -> u32 {
    CLOCKS.reference.load(Ordering::Relaxed)
}

pub fn clk_peri_freq() -> u32 {
    CLOCKS.peri.load(Ordering::Relaxed)
}

pub fn clk_usb_freq() -> u32 {
    CLOCKS.usb.load(Ordering::Relaxed)
}

pub fn clk_adc_freq() -> u32 {
    CLOCKS.adc.load(Ordering::Relaxed)
}

pub fn clk_rtc_freq() -> u16 {
    CLOCKS.rtc.load(Ordering::Relaxed)
}

fn start_xosc(crystal_hz: u32) {
    pac::XOSC
        .ctrl()
        .write(|w| w.set_freq_range(pac::xosc::vals::CtrlFreqRange::_1_15MHZ));

    let startup_delay = ((crystal_hz / 1000) + 128) / 256;
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
    assert!(vco_freq >= 750_000_000 && vco_freq <= 1800_000_000);

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

pub trait GpinPin: crate::gpio::Pin {
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

pub struct Gpin<'d, T: Pin> {
    gpin: PeripheralRef<'d, AnyPin>,
    _phantom: PhantomData<T>,
}

impl<'d, T: Pin> Gpin<'d, T> {
    pub fn new<P: GpinPin>(gpin: impl Peripheral<P = P> + 'd) -> Gpin<'d, P> {
        into_ref!(gpin);

        gpin.io().ctrl().write(|w| w.set_funcsel(0x08));

        Gpin {
            gpin: gpin.map_into(),
            _phantom: PhantomData,
        }
    }

    // fn map_into(self) -> Gpin<'d, AnyPin> {
    //     unsafe { core::mem::transmute(self) }
    // }
}

impl<'d, T: Pin> Drop for Gpin<'d, T> {
    fn drop(&mut self) {
        self.gpin
            .io()
            .ctrl()
            .write(|w| w.set_funcsel(pac::io::vals::Gpio0ctrlFuncsel::NULL as _));
    }
}

pub trait GpoutPin: crate::gpio::Pin {
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

#[repr(u8)]
pub enum GpoutSrc {
    PllSys = ClkGpoutCtrlAuxsrc::CLKSRC_PLL_SYS as _,
    // Gpin0 = ClkGpoutCtrlAuxsrc::CLKSRC_GPIN0 as _ ,
    // Gpin1 = ClkGpoutCtrlAuxsrc::CLKSRC_GPIN1 as _ ,
    PllUsb = ClkGpoutCtrlAuxsrc::CLKSRC_PLL_USB as _,
    Rosc = ClkGpoutCtrlAuxsrc::ROSC_CLKSRC as _,
    Xosc = ClkGpoutCtrlAuxsrc::XOSC_CLKSRC as _,
    Sys = ClkGpoutCtrlAuxsrc::CLK_SYS as _,
    Usb = ClkGpoutCtrlAuxsrc::CLK_USB as _,
    Adc = ClkGpoutCtrlAuxsrc::CLK_ADC as _,
    Rtc = ClkGpoutCtrlAuxsrc::CLK_RTC as _,
    Ref = ClkGpoutCtrlAuxsrc::CLK_REF as _,
}

pub struct Gpout<'d, T: GpoutPin> {
    gpout: PeripheralRef<'d, T>,
}

impl<'d, T: GpoutPin> Gpout<'d, T> {
    pub fn new(gpout: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(gpout);

        gpout.io().ctrl().write(|w| w.set_funcsel(0x08));

        Self { gpout }
    }

    pub fn set_div(&self, int: u32, frac: u8) {
        let c = pac::CLOCKS;
        c.clk_gpout_div(self.gpout.number()).write(|w| {
            w.set_int(int);
            w.set_frac(frac);
        });
    }

    pub fn set_src(&self, src: GpoutSrc) {
        let c = pac::CLOCKS;
        c.clk_gpout_ctrl(self.gpout.number()).modify(|w| {
            w.set_auxsrc(ClkGpoutCtrlAuxsrc::from_bits(src as _));
        });
    }

    pub fn enable(&self) {
        let c = pac::CLOCKS;
        c.clk_gpout_ctrl(self.gpout.number()).modify(|w| {
            w.set_enable(true);
        });
    }

    pub fn disable(&self) {
        let c = pac::CLOCKS;
        c.clk_gpout_ctrl(self.gpout.number()).modify(|w| {
            w.set_enable(false);
        });
    }

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
            ClkGpoutCtrlAuxsrc::CLK_RTC => clk_rtc_freq() as _,
            ClkGpoutCtrlAuxsrc::CLK_REF => clk_ref_freq(),
            _ => unreachable!(),
        };

        let div = c.clk_gpout_div(self.gpout.number()).read();
        let int = if div.int() == 0 { 65536 } else { div.int() } as u64;
        let frac = div.frac() as u64;

        ((base as u64 * 256) / (int * 256 + frac)) as u32
    }
}

impl<'d, T: GpoutPin> Drop for Gpout<'d, T> {
    fn drop(&mut self) {
        self.disable();
        self.gpout
            .io()
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
