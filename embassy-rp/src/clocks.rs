use embassy_hal_common::{into_ref, PeripheralRef};
use pac::clocks::vals::*;

use crate::{pac, reset, Peripheral};

// TODO fix terrible use of global here
static mut XIN_HZ: u32 = 0;

#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeriClkSrc {
    Sys = ClkPeriCtrlAuxsrc::CLK_SYS.0,
    PllSys = ClkPeriCtrlAuxsrc::CLKSRC_PLL_SYS.0,
    PllUsb = ClkPeriCtrlAuxsrc::CLKSRC_PLL_USB.0,
    Rosc = ClkPeriCtrlAuxsrc::ROSC_CLKSRC_PH.0,
    Xosc = ClkPeriCtrlAuxsrc::XOSC_CLKSRC.0,
    Gpin0 = ClkPeriCtrlAuxsrc::CLKSRC_GPIN0.0,
    Gpin1 = ClkPeriCtrlAuxsrc::CLKSRC_GPIN1.0,
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
}

impl ClockConfig {
    pub fn crystal(crystal_hz: u32) -> Self {
        Self {
            rosc: Some(RoscConfig {
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
            usb_clk: Some(UsbClkConfig {
                src: UsbClkSrc::PllUsb,
                div: 1,
                phase: 0,
            }),
            adc_clk: Some(AdcClkConfig {
                src: AdcClkSrc::PllUsb,
                div: 1,
                phase: 0,
            }),
            rtc_clk: Some(RtcClkConfig {
                src: RtcClkSrc::PllUsb,
                div_int: 1024,
                div_frac: 0,
                phase: 0,
            }),
        }
    }

    pub fn rosc() -> Self {
        Self {
            rosc: Some(RoscConfig {
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
            adc_clk: Some(AdcClkConfig {
                src: AdcClkSrc::Rosc,
                div: 1,
                phase: 0,
            }),
            rtc_clk: Some(RtcClkConfig {
                src: RtcClkSrc::Rosc,
                div_int: 1024,
                div_frac: 0,
                phase: 0,
            }),
        }
    }
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
    pub refdiv: u32,
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
    Gpin0,
    Gpin1,
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
    Gpin0,
    Gpin1,
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
    PllUsb = ClkUsbCtrlAuxsrc::CLKSRC_PLL_USB.0,
    PllSys = ClkUsbCtrlAuxsrc::CLKSRC_PLL_SYS.0,
    Rosc = ClkUsbCtrlAuxsrc::ROSC_CLKSRC_PH.0,
    Xosc = ClkUsbCtrlAuxsrc::XOSC_CLKSRC.0,
    Gpin0 = ClkUsbCtrlAuxsrc::CLKSRC_GPIN0.0,
    Gpin1 = ClkUsbCtrlAuxsrc::CLKSRC_GPIN1.0,
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
    PllUsb = ClkAdcCtrlAuxsrc::CLKSRC_PLL_USB.0,
    PllSys = ClkAdcCtrlAuxsrc::CLKSRC_PLL_SYS.0,
    Rosc = ClkAdcCtrlAuxsrc::ROSC_CLKSRC_PH.0,
    Xosc = ClkAdcCtrlAuxsrc::XOSC_CLKSRC.0,
    Gpin0 = ClkAdcCtrlAuxsrc::CLKSRC_GPIN0.0,
    Gpin1 = ClkAdcCtrlAuxsrc::CLKSRC_GPIN1.0,
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
    PllUsb = ClkRtcCtrlAuxsrc::CLKSRC_PLL_USB.0,
    PllSys = ClkRtcCtrlAuxsrc::CLKSRC_PLL_SYS.0,
    Rosc = ClkRtcCtrlAuxsrc::ROSC_CLKSRC_PH.0,
    Xosc = ClkRtcCtrlAuxsrc::XOSC_CLKSRC.0,
    Gpin0 = ClkRtcCtrlAuxsrc::CLKSRC_GPIN0.0,
    Gpin1 = ClkRtcCtrlAuxsrc::CLKSRC_GPIN1.0,
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
    let mut peris = reset::ALL_PERIPHERALS;
    peris.set_io_qspi(false);
    peris.set_pads_qspi(false);
    peris.set_pll_sys(false);
    peris.set_pll_usb(false);
    // TODO investigate if usb should be unreset here
    peris.set_usbctrl(false);
    peris.set_syscfg(false);
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

    if let Some(config) = config.rosc {
        configure_rosc(config);
    }

    if let Some(config) = config.xosc {
        XIN_HZ = config.hz;

        pac::WATCHDOG.tick().write(|w| {
            w.set_cycles((config.hz / 1_000_000) as u16);
            w.set_enable(true);
        });

        // start XOSC
        // datasheet mentions support for clock inputs into XIN, but doesn't go into
        // how this is achieved. pico-sdk doesn't support this at all.
        start_xosc(config.hz);

        if let Some(sys_pll_config) = config.sys_pll {
            configure_pll(pac::PLL_SYS, config.hz, sys_pll_config);
        }
        if let Some(usb_pll_config) = config.usb_pll {
            configure_pll(pac::PLL_USB, config.hz, usb_pll_config);
        }
    }

    let (ref_src, ref_aux) = {
        use {ClkRefCtrlAuxsrc as Aux, ClkRefCtrlSrc as Src};
        match config.ref_clk.src {
            RefClkSrc::Xosc => (Src::XOSC_CLKSRC, Aux::CLKSRC_PLL_USB),
            RefClkSrc::Rosc => (Src::ROSC_CLKSRC_PH, Aux::CLKSRC_PLL_USB),
            RefClkSrc::PllUsb => (Src::CLKSRC_CLK_REF_AUX, Aux::CLKSRC_PLL_USB),
            RefClkSrc::Gpin0 => (Src::CLKSRC_CLK_REF_AUX, Aux::CLKSRC_GPIN0),
            RefClkSrc::Gpin1 => (Src::CLKSRC_CLK_REF_AUX, Aux::CLKSRC_GPIN1),
        }
    };
    c.clk_ref_ctrl().write(|w| {
        w.set_src(ref_src);
        w.set_auxsrc(ref_aux);
    });
    while c.clk_ref_selected().read() != 1 << ref_src.0 {}
    c.clk_ref_div().write(|w| {
        w.set_int(config.ref_clk.div);
    });

    pac::WATCHDOG.tick().write(|w| {
        w.set_cycles((clk_ref_freq() / 1_000_000) as u16);
        w.set_enable(true);
    });

    let (sys_src, sys_aux) = {
        use {ClkSysCtrlAuxsrc as Aux, ClkSysCtrlSrc as Src};
        match config.sys_clk.src {
            SysClkSrc::Ref => (Src::CLK_REF, Aux::CLKSRC_PLL_SYS),
            SysClkSrc::PllSys => (Src::CLKSRC_CLK_SYS_AUX, Aux::CLKSRC_PLL_SYS),
            SysClkSrc::PllUsb => (Src::CLKSRC_CLK_SYS_AUX, Aux::CLKSRC_PLL_USB),
            SysClkSrc::Rosc => (Src::CLKSRC_CLK_SYS_AUX, Aux::ROSC_CLKSRC),
            SysClkSrc::Xosc => (Src::CLKSRC_CLK_SYS_AUX, Aux::XOSC_CLKSRC),
            SysClkSrc::Gpin0 => (Src::CLKSRC_CLK_SYS_AUX, Aux::CLKSRC_GPIN0),
            SysClkSrc::Gpin1 => (Src::CLKSRC_CLK_SYS_AUX, Aux::CLKSRC_GPIN1),
        }
    };
    if sys_src != ClkSysCtrlSrc::CLK_REF {
        c.clk_sys_ctrl().write(|w| w.set_src(ClkSysCtrlSrc::CLK_REF));
        while c.clk_sys_selected().read() != 1 << ClkSysCtrlSrc::CLK_REF.0 {}
    }
    c.clk_sys_ctrl().write(|w| {
        w.set_auxsrc(sys_aux);
        w.set_src(sys_src);
    });
    while c.clk_sys_selected().read() != 1 << sys_src.0 {}
    c.clk_sys_div().write(|w| {
        w.set_int(config.sys_clk.div_int);
        w.set_frac(config.sys_clk.div_frac);
    });

    let mut peris = reset::ALL_PERIPHERALS;

    if let Some(src) = config.peri_clk_src {
        c.clk_peri_ctrl().write(|w| {
            w.set_enable(true);
            w.set_auxsrc(ClkPeriCtrlAuxsrc(src as _));
        });
    } else {
        peris.set_spi0(false);
        peris.set_spi1(false);
        peris.set_uart0(false);
        peris.set_uart1(false);
    }

    if let Some(conf) = config.usb_clk {
        // CLK USB = PLL USB (48MHz) / 1 = 48MHz
        c.clk_usb_div().write(|w| w.set_int(conf.div));
        c.clk_usb_ctrl().write(|w| {
            w.set_phase(conf.phase);
            w.set_enable(true);
            w.set_auxsrc(ClkUsbCtrlAuxsrc(conf.src as _));
        });
    } else {
        peris.set_usbctrl(false);
    }

    if let Some(conf) = config.adc_clk {
        // CLK ADC = PLL USB (48MHZ) / 1 = 48MHz
        c.clk_adc_div().write(|w| w.set_int(conf.div));
        c.clk_adc_ctrl().write(|w| {
            w.set_phase(conf.phase);
            w.set_enable(true);
            w.set_auxsrc(ClkAdcCtrlAuxsrc(conf.src as _));
        });
    } else {
        peris.set_adc(false);
    }

    if let Some(conf) = config.rtc_clk {
        // CLK RTC = PLL USB (48MHz) / 1024 = 46875Hz
        c.clk_rtc_ctrl().modify(|w| {
            w.set_enable(false);
        });
        c.clk_rtc_div().write(|w| {
            w.set_int(conf.div_int);
            w.set_frac(conf.div_frac);
        });
        c.clk_rtc_ctrl().write(|w| {
            w.set_phase(conf.phase);
            w.set_enable(true);
            w.set_auxsrc(ClkRtcCtrlAuxsrc(conf.src as _));
        });
    } else {
        peris.set_rtc(false);
    }

    // Peripheral clocks should now all be running
    reset::unreset_wait(peris);
}

unsafe fn configure_rosc(config: RoscConfig) {
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
}

pub fn estimate_rosc_freq() -> u32 {
    let p = pac::ROSC;

    let base = match unsafe { p.ctrl().read().freq_range() } {
        pac::rosc::vals::FreqRange::LOW => 84_000_000,
        pac::rosc::vals::FreqRange::MEDIUM => 104_000_000,
        pac::rosc::vals::FreqRange::HIGH => 140_000_000,
        pac::rosc::vals::FreqRange::TOOHIGH => 208_000_000,
        _ => unreachable!(),
    };
    let mut div = unsafe { p.div().read().0 - pac::rosc::vals::Div::PASS.0 as u32 };
    if div == 0 {
        div = 32
    }

    base / div
}

pub fn xosc_freq() -> u32 {
    unsafe { XIN_HZ }
}

pub fn gpin0_freq() -> u32 {
    todo!()
}
pub fn gpin1_freq() -> u32 {
    todo!()
}

pub fn pll_sys_freq() -> u32 {
    let p = pac::PLL_SYS;

    let input_freq = xosc_freq();
    let cs = unsafe { p.cs().read() };

    let refdiv = cs.refdiv() as u32;
    let fbdiv = unsafe { p.fbdiv_int().read().fbdiv_int() } as u32;
    let (postdiv1, postdiv2) = unsafe {
        let prim = p.prim().read();
        (prim.postdiv1() as u32, prim.postdiv2() as u32)
    };

    (((input_freq / refdiv) * fbdiv) / postdiv1) / postdiv2
}

pub fn pll_usb_freq() -> u32 {
    let p = pac::PLL_USB;

    let input_freq = xosc_freq();
    let cs = unsafe { p.cs().read() };

    let refdiv = cs.refdiv() as u32;
    let fbdiv = unsafe { p.fbdiv_int().read().fbdiv_int() } as u32;
    let (postdiv1, postdiv2) = unsafe {
        let prim = p.prim().read();
        (prim.postdiv1() as u32, prim.postdiv2() as u32)
    };

    (((input_freq / refdiv) * fbdiv) / postdiv1) / postdiv2
}

pub fn clk_sys_freq() -> u32 {
    let c = pac::CLOCKS;
    let ctrl = unsafe { c.clk_sys_ctrl().read() };

    let base = match ctrl.src() {
        ClkSysCtrlSrc::CLK_REF => clk_ref_freq(),
        ClkSysCtrlSrc::CLKSRC_CLK_SYS_AUX => match ctrl.auxsrc() {
            ClkSysCtrlAuxsrc::CLKSRC_PLL_SYS => pll_sys_freq(),
            ClkSysCtrlAuxsrc::CLKSRC_PLL_USB => pll_usb_freq(),
            ClkSysCtrlAuxsrc::ROSC_CLKSRC => estimate_rosc_freq(),
            ClkSysCtrlAuxsrc::XOSC_CLKSRC => xosc_freq(),
            ClkSysCtrlAuxsrc::CLKSRC_GPIN0 => gpin0_freq(),
            ClkSysCtrlAuxsrc::CLKSRC_GPIN1 => gpin1_freq(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    let div = unsafe { c.clk_sys_div().read() };
    let int = if div.int() == 0 { 65536 } else { div.int() };
    // TODO handle fractional clock div
    let _frac = div.frac();

    base / int
}

pub fn clk_ref_freq() -> u32 {
    let c = pac::CLOCKS;
    let ctrl = unsafe { c.clk_ref_ctrl().read() };

    let base = match ctrl.src() {
        ClkRefCtrlSrc::ROSC_CLKSRC_PH => estimate_rosc_freq(),
        ClkRefCtrlSrc::XOSC_CLKSRC => xosc_freq(),
        ClkRefCtrlSrc::CLKSRC_CLK_REF_AUX => match ctrl.auxsrc() {
            ClkRefCtrlAuxsrc::CLKSRC_PLL_USB => pll_usb_freq(),
            ClkRefCtrlAuxsrc::CLKSRC_GPIN0 => gpin0_freq(),
            ClkRefCtrlAuxsrc::CLKSRC_GPIN1 => gpin1_freq(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    let div = unsafe { c.clk_ref_div().read() };
    let int = if div.int() == 0 { 4 } else { div.int() as u32 };

    base / int
}

pub fn clk_peri_freq() -> u32 {
    let c = pac::CLOCKS;
    let src = unsafe { c.clk_peri_ctrl().read().auxsrc() };

    match src {
        ClkPeriCtrlAuxsrc::CLK_SYS => clk_sys_freq(),
        ClkPeriCtrlAuxsrc::CLKSRC_PLL_SYS => pll_sys_freq(),
        ClkPeriCtrlAuxsrc::CLKSRC_PLL_USB => pll_usb_freq(),
        ClkPeriCtrlAuxsrc::ROSC_CLKSRC_PH => estimate_rosc_freq(),
        ClkPeriCtrlAuxsrc::XOSC_CLKSRC => xosc_freq(),
        ClkPeriCtrlAuxsrc::CLKSRC_GPIN0 => gpin0_freq(),
        ClkPeriCtrlAuxsrc::CLKSRC_GPIN1 => gpin1_freq(),
        _ => unreachable!(),
    }
}

pub fn clk_usb_freq() -> u32 {
    let c = pac::CLOCKS;
    let ctrl = unsafe { c.clk_usb_ctrl().read() };

    let base = match ctrl.auxsrc() {
        ClkUsbCtrlAuxsrc::CLKSRC_PLL_SYS => pll_sys_freq(),
        ClkUsbCtrlAuxsrc::CLKSRC_PLL_USB => pll_usb_freq(),
        ClkUsbCtrlAuxsrc::ROSC_CLKSRC_PH => estimate_rosc_freq(),
        ClkUsbCtrlAuxsrc::XOSC_CLKSRC => xosc_freq(),
        ClkUsbCtrlAuxsrc::CLKSRC_GPIN0 => gpin0_freq(),
        ClkUsbCtrlAuxsrc::CLKSRC_GPIN1 => gpin1_freq(),
        _ => unreachable!(),
    };

    let div = unsafe { c.clk_ref_div().read() };
    let int = if div.int() == 0 { 4 } else { div.int() as u32 };

    base / int
}

pub fn clk_adc_freq() -> u32 {
    let c = pac::CLOCKS;
    let ctrl = unsafe { c.clk_adc_ctrl().read() };

    let base = match ctrl.auxsrc() {
        ClkAdcCtrlAuxsrc::CLKSRC_PLL_SYS => pll_sys_freq(),
        ClkAdcCtrlAuxsrc::CLKSRC_PLL_USB => pll_usb_freq(),
        ClkAdcCtrlAuxsrc::ROSC_CLKSRC_PH => estimate_rosc_freq(),
        ClkAdcCtrlAuxsrc::XOSC_CLKSRC => xosc_freq(),
        ClkAdcCtrlAuxsrc::CLKSRC_GPIN0 => gpin0_freq(),
        ClkAdcCtrlAuxsrc::CLKSRC_GPIN1 => gpin1_freq(),
        _ => unreachable!(),
    };

    let div = unsafe { c.clk_adc_div().read() };
    let int = if div.int() == 0 { 4 } else { div.int() as u32 };

    base / int
}

pub fn clk_rtc_freq() -> u32 {
    let c = pac::CLOCKS;
    let src = unsafe { c.clk_rtc_ctrl().read().auxsrc() };

    let base = match src {
        ClkRtcCtrlAuxsrc::CLKSRC_PLL_USB => pll_usb_freq(),
        ClkRtcCtrlAuxsrc::CLKSRC_PLL_SYS => pll_sys_freq(),
        ClkRtcCtrlAuxsrc::ROSC_CLKSRC_PH => estimate_rosc_freq(),
        ClkRtcCtrlAuxsrc::XOSC_CLKSRC => xosc_freq(),
        ClkRtcCtrlAuxsrc::CLKSRC_GPIN0 => gpin0_freq(),
        ClkRtcCtrlAuxsrc::CLKSRC_GPIN1 => gpin1_freq(),
        _ => unreachable!(),
    };

    let div = unsafe { c.clk_rtc_div().read() };
    let int = if div.int() == 0 { 65536 } else { div.int() };
    // TODO handle fractional clock div
    let _frac = div.frac();

    base / int
}

unsafe fn start_xosc(crystal_hz: u32) {
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
unsafe fn configure_pll(p: pac::pll::Pll, input_freq: u32, config: PllConfig) {
    let ref_freq = input_freq / config.refdiv;

    assert!(config.fbdiv >= 16 && config.fbdiv <= 320);
    assert!(config.post_div1 >= 1 && config.post_div1 <= 7);
    assert!(config.post_div2 >= 1 && config.post_div2 <= 7);
    assert!(config.post_div2 <= config.post_div1);
    assert!(ref_freq >= 5_000_000 && ref_freq <= 800_000_000);

    // Load VCO-related dividers before starting VCO
    p.cs().write(|w| w.set_refdiv(config.refdiv as _));
    p.fbdiv_int().write(|w| w.set_fbdiv_int(config.fbdiv));

    // Turn on PLL
    p.pwr().modify(|w| {
        w.set_pd(false);
        w.set_vcopd(false);
        w.set_postdivpd(true);
    });

    // Wait for PLL to lock
    while !p.cs().read().lock() {}

    // Wait for PLL to lock
    p.prim().write(|w| {
        w.set_postdiv1(config.post_div1);
        w.set_postdiv2(config.post_div2);
    });

    // Turn on post divider
    p.pwr().modify(|w| w.set_postdivpd(false));
}

pub trait GpinPin: crate::gpio::Pin {
    fn number(&self) -> usize;
}

macro_rules! impl_gpinpin {
    ($name:ident, $pin_num:expr, $gpin_num:expr) => {
        impl GpinPin for crate::peripherals::$name {
            fn number(&self) -> usize {
                $gpin_num
            }
        }
    };
}

impl_gpinpin!(PIN_20, 20, 0);
impl_gpinpin!(PIN_22, 22, 1);

pub struct Gpin<'d, T: GpinPin> {
    gpin: PeripheralRef<'d, T>,
}

impl<'d, T: GpinPin> Gpin<'d, T> {
    pub fn new(gpin: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(gpin);

        unsafe {
            gpin.io().ctrl().write(|w| w.set_funcsel(0x08));
        }

        Self { gpin }
    }
}

impl<'d, T: GpinPin> Drop for Gpin<'d, T> {
    fn drop(&mut self) {
        unsafe {
            self.gpin
                .io()
                .ctrl()
                .write(|w| w.set_funcsel(pac::io::vals::Gpio0ctrlFuncsel::NULL.0));
        }
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
    PllSys = ClkGpoutCtrlAuxsrc::CLKSRC_PLL_SYS.0,
    Gpin0 = ClkGpoutCtrlAuxsrc::CLKSRC_GPIN0.0,
    Gpin1 = ClkGpoutCtrlAuxsrc::CLKSRC_GPIN1.0,
    PllUsb = ClkGpoutCtrlAuxsrc::CLKSRC_PLL_USB.0,
    Rosc = ClkGpoutCtrlAuxsrc::ROSC_CLKSRC.0,
    Xosc = ClkGpoutCtrlAuxsrc::XOSC_CLKSRC.0,
    Sys = ClkGpoutCtrlAuxsrc::CLK_SYS.0,
    Usb = ClkGpoutCtrlAuxsrc::CLK_USB.0,
    Adc = ClkGpoutCtrlAuxsrc::CLK_ADC.0,
    Rtc = ClkGpoutCtrlAuxsrc::CLK_RTC.0,
    Ref = ClkGpoutCtrlAuxsrc::CLK_REF.0,
}

pub struct Gpout<'d, T: GpoutPin> {
    gpout: PeripheralRef<'d, T>,
}

impl<'d, T: GpoutPin> Gpout<'d, T> {
    pub fn new(gpout: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(gpout);

        unsafe {
            gpout.io().ctrl().write(|w| w.set_funcsel(0x08));
        }

        Self { gpout }
    }

    pub fn set_div(&self, int: u32, frac: u8) {
        unsafe {
            let c = pac::CLOCKS;
            c.clk_gpout_div(self.gpout.number()).write(|w| {
                w.set_int(int);
                w.set_frac(frac);
            });
        }
    }

    pub fn set_src(&self, src: GpoutSrc) {
        unsafe {
            let c = pac::CLOCKS;
            c.clk_gpout_ctrl(self.gpout.number()).modify(|w| {
                w.set_auxsrc(ClkGpoutCtrlAuxsrc(src as _));
            });
        }
    }

    pub fn enable(&self) {
        unsafe {
            let c = pac::CLOCKS;
            c.clk_gpout_ctrl(self.gpout.number()).modify(|w| {
                w.set_enable(true);
            });
        }
    }

    pub fn disable(&self) {
        unsafe {
            let c = pac::CLOCKS;
            c.clk_gpout_ctrl(self.gpout.number()).modify(|w| {
                w.set_enable(false);
            });
        }
    }

    pub fn get_freq(&self) -> u32 {
        let c = pac::CLOCKS;
        let src = unsafe { c.clk_gpout_ctrl(self.gpout.number()).read().auxsrc() };

        let base = match src {
            ClkGpoutCtrlAuxsrc::CLKSRC_PLL_SYS => pll_sys_freq(),
            ClkGpoutCtrlAuxsrc::CLKSRC_GPIN0 => gpin0_freq(),
            ClkGpoutCtrlAuxsrc::CLKSRC_GPIN1 => gpin1_freq(),
            ClkGpoutCtrlAuxsrc::CLKSRC_PLL_USB => pll_usb_freq(),
            ClkGpoutCtrlAuxsrc::ROSC_CLKSRC => estimate_rosc_freq(),
            ClkGpoutCtrlAuxsrc::XOSC_CLKSRC => xosc_freq(),
            ClkGpoutCtrlAuxsrc::CLK_SYS => clk_sys_freq(),
            ClkGpoutCtrlAuxsrc::CLK_USB => clk_usb_freq(),
            ClkGpoutCtrlAuxsrc::CLK_ADC => clk_adc_freq(),
            ClkGpoutCtrlAuxsrc::CLK_RTC => clk_rtc_freq(),
            ClkGpoutCtrlAuxsrc::CLK_REF => clk_ref_freq(),
            _ => unreachable!(),
        };

        let div = unsafe { c.clk_gpout_div(self.gpout.number()).read() };
        let int = if div.int() == 0 { 65536 } else { div.int() };
        // TODO handle fractional clock div
        let _frac = div.frac();

        base / int
    }
}

impl<'d, T: GpoutPin> Drop for Gpout<'d, T> {
    fn drop(&mut self) {
        self.disable();
        unsafe {
            self.gpout
                .io()
                .ctrl()
                .write(|w| w.set_funcsel(pac::io::vals::Gpio0ctrlFuncsel::NULL.0));
        }
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
            acc |= unsafe { random_reg.read().randombit() as u8 };
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
