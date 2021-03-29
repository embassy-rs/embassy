use crate::{pac, pll, resets};

#[non_exhaustive]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

/// safety: must only call once.
pub unsafe fn configure(_config: Config) {
    // Now reset all the peripherals, except QSPI and XIP (we're using those
    // to execute from external flash!)

    let resets = resets::Resets::new();

    // Reset everything except:
    // - QSPI (we're using it to run this code!)
    // - PLLs (it may be suicide if that's what's clocking us)
    let mut peris = resets::ALL_PERIPHERALS;
    peris.set_io_qspi(false);
    peris.set_pads_qspi(false);
    peris.set_pll_sys(false);
    peris.set_pll_usb(false);
    resets.reset(peris);

    let mut peris = resets::ALL_PERIPHERALS;
    peris.set_adc(false);
    peris.set_rtc(false);
    peris.set_spi0(false);
    peris.set_spi1(false);
    peris.set_uart0(false);
    peris.set_uart1(false);
    peris.set_usbctrl(false);
    resets.unreset_wait(peris);

    // xosc 12 mhz
    pac::WATCHDOG.tick().write(|w| {
        w.set_cycles(XOSC_MHZ as u16);
        w.set_enable(true);
    });

    pac::CLOCKS
        .clk_sys_resus_ctrl()
        .write_value(pac::clocks::regs::ClkSysResusCtrl(0));

    // Enable XOSC
    // TODO extract to HAL module
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
        w.set_enable(pac::xosc::vals::CtrlEnable::ENABLE);
    });
    while !pac::XOSC.status().read().stable() {}

    // Before we touch PLLs, switch sys and ref cleanly away from their aux sources.
    pac::CLOCKS
        .clk_sys_ctrl()
        .modify(|w| w.set_src(pac::clocks::vals::ClkSysCtrlSrc::CLK_REF));
    while pac::CLOCKS.clk_sys_selected().read() != 1 {}
    pac::CLOCKS
        .clk_ref_ctrl()
        .modify(|w| w.set_src(pac::clocks::vals::ClkRefCtrlSrc::ROSC_CLKSRC_PH));
    while pac::CLOCKS.clk_ref_selected().read() != 1 {}

    let mut peris = resets::Peripherals(0);
    peris.set_pll_sys(true);
    peris.set_pll_usb(true);
    resets.reset(peris);
    resets.unreset_wait(peris);

    pll::PLL::new(pll::PllSys).configure(1, 1500_000_000, 6, 2);
    pll::PLL::new(pll::PllUsb).configure(1, 480_000_000, 5, 2);

    // Activate peripheral clock and take external oscillator as input
    pac::CLOCKS.clk_peri_ctrl().write(|w| {
        w.set_enable(true);
        w.set_auxsrc(pac::clocks::vals::ClkPeriCtrlAuxsrc::XOSC_CLKSRC);
    });
}
