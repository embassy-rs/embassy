use pac::clocks::vals::*;

use crate::{pac, reset};

const XOSC_MHZ: u32 = 12;

pub unsafe fn init() {
    // Now reset all the peripherals, except QSPI and XIP (we're using those
    // to execute from external flash!)

    // Reset everything except:
    // - QSPI (we're using it to run this code!)
    // - PLLs (it may be suicide if that's what's clocking us)
    let mut peris = reset::ALL_PERIPHERALS;
    peris.set_io_qspi(false);
    peris.set_pads_qspi(false);
    peris.set_pll_sys(false);
    peris.set_pll_usb(false);
    reset::reset(peris);

    let mut peris = reset::ALL_PERIPHERALS;
    peris.set_adc(false);
    peris.set_rtc(false);
    peris.set_spi0(false);
    peris.set_spi1(false);
    peris.set_uart0(false);
    peris.set_uart1(false);
    peris.set_usbctrl(false);
    reset::unreset_wait(peris);

    // xosc 12 mhz
    pac::WATCHDOG.tick().write(|w| {
        w.set_cycles(XOSC_MHZ as u16);
        w.set_enable(true);
    });

    let c = pac::CLOCKS;
    c.clk_sys_resus_ctrl()
        .write_value(pac::clocks::regs::ClkSysResusCtrl(0));

    // Enable XOSC
    const XOSC_MHZ: u32 = 12;
    pac::XOSC
        .ctrl()
        .write(|w| w.set_freq_range(pac::xosc::vals::CtrlFreqRange::_1_15MHZ));

    let startup_delay = (((XOSC_MHZ * 1_000_000) / 1000) + 128) / 256;
    pac::XOSC
        .startup()
        .write(|w| w.set_delay(startup_delay as u16));
    pac::XOSC.ctrl().write(|w| {
        w.set_freq_range(pac::xosc::vals::CtrlFreqRange::_1_15MHZ);
        w.set_enable(pac::xosc::vals::Enable::ENABLE);
    });
    while !pac::XOSC.status().read().stable() {}

    // Before we touch PLLs, switch sys and ref cleanly away from their aux sources.
    c.clk_sys_ctrl()
        .modify(|w| w.set_src(ClkSysCtrlSrc::CLK_REF));
    while c.clk_sys_selected().read() != 1 {}
    c.clk_ref_ctrl()
        .modify(|w| w.set_src(ClkRefCtrlSrc::ROSC_CLKSRC_PH));
    while c.clk_ref_selected().read() != 1 {}

    // Configure PLLs
    //                   REF     FBDIV VCO            POSTDIV
    // PLL SYS: 12 / 1 = 12MHz * 125 = 1500MHZ / 6 / 2 = 125MHz
    // PLL USB: 12 / 1 = 12MHz * 40  = 480 MHz / 5 / 2 =  48MHz

    let mut peris = reset::Peripherals(0);
    peris.set_pll_sys(true);
    peris.set_pll_usb(true);
    reset::reset(peris);
    reset::unreset_wait(peris);

    configure_pll(pac::PLL_SYS, 1, 1500_000_000, 6, 2);
    configure_pll(pac::PLL_USB, 1, 480_000_000, 5, 2);

    // CLK_REF = XOSC (12MHz) / 1 = 12MHz2Mhz
    c.clk_ref_ctrl().write(|w| {
        w.set_src(ClkRefCtrlSrc::XOSC_CLKSRC);
    });
    while c.clk_ref_selected().read() != 1 << ClkRefCtrlSrc::XOSC_CLKSRC.0 {}
    c.clk_ref_div().write(|w| w.set_int(1));

    // CLK SYS = PLL SYS (125MHz) / 1 = 125MHz
    c.clk_sys_ctrl().write(|w| {
        w.set_src(ClkSysCtrlSrc::CLK_REF);
    });
    while c.clk_sys_selected().read() != 1 << ClkSysCtrlSrc::CLK_REF.0 {}
    c.clk_sys_div().write(|w| w.set_int(1));
    c.clk_sys_ctrl().write(|w| {
        w.set_auxsrc(ClkSysCtrlAuxsrc::CLKSRC_PLL_SYS);
        w.set_src(ClkSysCtrlSrc::CLKSRC_CLK_SYS_AUX);
    });
    while c.clk_sys_selected().read() != 1 << ClkSysCtrlSrc::CLKSRC_CLK_SYS_AUX.0 {}

    // CLK USB = PLL USB (48MHz) / 1 = 48MHz
    c.clk_usb_div().write(|w| w.set_int(1));
    c.clk_usb_ctrl().write(|w| {
        w.set_enable(true);
        w.set_auxsrc(ClkUsbCtrlAuxsrc::CLKSRC_PLL_USB);
    });

    // CLK ADC = PLL USB (48MHZ) / 1 = 48MHz
    c.clk_adc_div().write(|w| w.set_int(1));
    c.clk_adc_ctrl().write(|w| {
        w.set_enable(true);
        w.set_auxsrc(ClkAdcCtrlAuxsrc::CLKSRC_PLL_USB);
    });

    // CLK RTC = PLL USB (48MHz) / 1024 = 46875Hz
    c.clk_rtc_ctrl().modify(|w| {
        w.set_enable(false);
    });
    c.clk_rtc_div().write(|w| w.set_int(1024));
    c.clk_rtc_ctrl().write(|w| {
        w.set_enable(true);
        w.set_auxsrc(ClkRtcCtrlAuxsrc::CLKSRC_PLL_USB);
    });

    // CLK PERI = clk_sys. Used as reference clock for Peripherals. No dividers so just select and enable
    // Normally choose clk_sys or clk_usb
    c.clk_peri_ctrl().write(|w| {
        w.set_enable(true);
        w.set_auxsrc(ClkPeriCtrlAuxsrc::CLK_SYS);
    });
}

pub(crate) fn clk_sys_freq() -> u32 {
    125_000_000
}

pub(crate) fn clk_peri_freq() -> u32 {
    125_000_000
}

pub(crate) fn clk_rtc_freq() -> u32 {
    46875
}

unsafe fn configure_pll(
    p: pac::pll::Pll,
    refdiv: u32,
    vco_freq: u32,
    post_div1: u8,
    post_div2: u8,
) {
    // Power off in case it's already running
    p.pwr().write(|w| {
        w.set_vcopd(true);
        w.set_postdivpd(true);
        w.set_dsmpd(true);
        w.set_pd(true);
    });
    p.fbdiv_int().write(|w| w.set_fbdiv_int(0));

    let ref_mhz = XOSC_MHZ / refdiv;
    p.cs().write(|w| w.set_refdiv(ref_mhz as _));

    let fbdiv = vco_freq / (ref_mhz * 1_000_000);
    assert!(fbdiv >= 16 && fbdiv <= 520);
    assert!((post_div1 >= 1 && post_div1 <= 7) && (post_div2 >= 1 && post_div2 <= 7));
    assert!(post_div2 <= post_div1);
    assert!(ref_mhz <= (vco_freq / 16));

    p.fbdiv_int().write(|w| w.set_fbdiv_int(fbdiv as _));

    p.pwr().modify(|w| {
        w.set_pd(false);
        w.set_vcopd(false);
    });

    while !p.cs().read().lock() {}

    p.prim().write(|w| {
        w.set_postdiv1(post_div1);
        w.set_postdiv2(post_div2);
    });

    p.pwr().modify(|w| w.set_postdivpd(false));
}
