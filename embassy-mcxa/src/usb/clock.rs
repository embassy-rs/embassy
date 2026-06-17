//! USB clock gating and USBPHY bring-up for the MCXA5xx USBHS controller.
//!
//! This mirrors the NXP MCUXpresso SDK `USB_DeviceClockInit` / `USB_EhciPhyInit`
//! sequence, reduced to what a full-speed device needs. Register layout matches
//! `PERI_USBPHY.h` from the SDK.

// Some bit constants and the deinit path document the hardware but are not all
// exercised by the current full-speed device bring-up.
#![allow(dead_code)]

use core::ptr::{read_volatile, write_volatile};

use super::registers::USBPHY_BASE;

mod phy_off {
    pub const PWD: usize = 0x00;
    pub const TX: usize = 0x10;
    pub const CTRL: usize = 0x30;
    pub const CTRL_SET: usize = 0x34;
    pub const CTRL_CLR: usize = 0x38;
    pub const PLL_SIC: usize = 0xA0;
    pub const PLL_SIC_SET: usize = 0xA4;
    pub const PLL_SIC_CLR: usize = 0xA8;
    pub const ANACTRL_SET: usize = 0x104;
    pub const TRIM_OVERRIDE_EN: usize = 0x130;
}

// USBPHY CTRL bits (standard MXS/i.MX USB PHY).
const CTRL_SFTRST: u32 = 1 << 31;
const CTRL_CLKGATE: u32 = 1 << 30;
const CTRL_ENUTMILEVEL2: u32 = 1 << 14;
const CTRL_ENUTMILEVEL3: u32 = 1 << 15;

// USBPHY PLL_SIC bits.
const PLL_EN_USB_CLKS: u32 = 0x40;
const PLL_POWER: u32 = 0x1000;
const PLL_BYPASS: u32 = 1 << 16;
const PLL_REG_ENABLE: u32 = 0x0020_0000;
const PLL_DIV_SEL_SHIFT: u32 = 22;
const PLL_LOCK: u32 = 0x8000_0000;
const ANACTRL_LVI_EN: u32 = 0x2;

#[inline(always)]
fn phy_r(off: usize) -> u32 {
    // SAFETY: valid USBPHY register offset.
    unsafe { read_volatile((USBPHY_BASE + off) as *const u32) }
}
#[inline(always)]
fn phy_w(off: usize, v: u32) {
    // SAFETY: valid USBPHY register offset.
    unsafe { write_volatile((USBPHY_BASE + off) as *mut u32, v) }
}

/// PHY trim parameters for the high-speed transmit drivers. Taken from the
/// FRDM-MCXA577 board support (`BOARD_USB_PHY_*`).
#[derive(Clone, Copy)]
pub struct PhyConfig {
    /// `D_CAL` trim value.
    pub d_cal: u32,
    /// `TXCAL45DP` trim value.
    pub txcal45dp: u32,
    /// `TXCAL45DM` trim value.
    pub txcal45dm: u32,
    /// PLL divider select for the reference clock (2 = 24 MHz reference).
    pub pll_div_sel: u32,
}

impl Default for PhyConfig {
    fn default() -> Self {
        // FRDM-MCXA577 board defaults; 24 MHz crystal reference.
        Self {
            d_cal: 0x04,
            txcal45dp: 0x07,
            txcal45dm: 0x07,
            pll_div_sel: 2,
        }
    }
}

/// Enable the USB controller and PHY clock gates in MRCC, select clock sources,
/// and bring up the USBPHY for the EHCI device controller.
///
/// # Safety
/// Must be called once during USB initialization with exclusive access to the
/// MRCC and USBPHY. The SOSC / PHY reference clock must already be running.
pub(crate) unsafe fn init_clocks_and_phy(cfg: &PhyConfig) {
    use crate::pac::mrcc::{ClkdivHalt, ClkdivReset, ClkdivUnstab, PhyClkselMux, UsbClkselMux};
    use crate::pac::{MRCC0, SPC0};

    // Match the FRDM-MCXA577 SDK USB init delay setting before enabling the
    // USB clocks. Voltage policy remains owned by the clock driver.
    SPC0.active_vdelay().write(|w| w.set_active_vdelay(0x0500));

    MRCC0
        .mrcc_usb1_clksel()
        .write(|w| w.set_mux(UsbClkselMux::I2ClkUsbhs0PhyClkXtal));
    MRCC0
        .mrcc_usb1_phy_clksel()
        .write(|w| w.set_mux(PhyClkselMux::I2ClkrootSosc));

    // Raw divider 0 is divide-by-1, matching `CLOCK_SetClockDiv(..., 1)`.
    let phy_clkdiv = MRCC0.mrcc_usb1_phy_clkdiv();
    phy_clkdiv.modify(|w| {
        w.set_div(0);
        w.set_halt(ClkdivHalt::Off);
        w.set_reset(ClkdivReset::Off);
    });
    phy_clkdiv.modify(|w| {
        w.set_halt(ClkdivHalt::On);
        w.set_reset(ClkdivReset::On);
    });
    while phy_clkdiv.read().unstab() == ClkdivUnstab::Off {}

    // 1. Ungate the USB1 controller and PHY clocks.
    MRCC0.mrcc_glb_cc2().modify(|w| {
        w.set_usb1(true);
        w.set_usb1_phy(true);
    });

    // 2. Release the USB1 + PHY resets (reset bit set = released).
    MRCC0.mrcc_glb_rst2().modify(|w| {
        w.set_usb1(true);
        w.set_usb1_phy(true);
    });

    // 3. Bring up the USBPHY PLL, matching `CLOCK_EnableUsbhsPhyPllClock`.
    phy_w(phy_off::CTRL_CLR, CTRL_SFTRST);
    phy_w(phy_off::ANACTRL_SET, ANACTRL_LVI_EN);
    phy_w(phy_off::PLL_SIC_SET, PLL_REG_ENABLE);
    // SDK waits at least 15 us for the PLL regulator to stabilize. This is
    // deliberately conservative in cycles because the CPU clock varies by app.
    cortex_m::asm::delay(15_000);
    phy_w(phy_off::PLL_SIC_SET, PLL_POWER);

    let sic = phy_r(phy_off::PLL_SIC);
    let sic = (sic & !(0b111 << PLL_DIV_SEL_SHIFT)) | ((cfg.pll_div_sel & 0b111) << PLL_DIV_SEL_SHIFT);
    phy_w(phy_off::PLL_SIC, sic);
    phy_w(phy_off::PLL_SIC_CLR, PLL_BYPASS);
    phy_w(phy_off::PLL_SIC_SET, PLL_EN_USB_CLKS);

    phy_w(phy_off::CTRL_CLR, CTRL_CLKGATE);
    phy_w(phy_off::PWD, 0);

    let mut locked = false;
    for _ in 0..4_000_000 {
        if phy_r(phy_off::PLL_SIC) & PLL_LOCK != 0 {
            locked = true;
            break;
        }
        cortex_m::asm::nop();
    }
    assert!(locked, "USB PHY PLL did not lock");

    // 4. Finish normal EHCI PHY init, matching `USB_EhciPhyInit`.
    phy_w(phy_off::TRIM_OVERRIDE_EN, 0x001F);
    phy_w(phy_off::CTRL_SET, CTRL_ENUTMILEVEL2 | CTRL_ENUTMILEVEL3);
    phy_w(phy_off::PWD, 0);

    // 5. Apply the transmit calibration trims.
    let tx = phy_r(phy_off::TX);
    // D_CAL occupies bits 3:0, TXCAL45DP bits 19:16, TXCAL45DM bits 11:8.
    let tx = tx & !(0x000F | 0x0F00 | 0x000F_0000);
    let tx = tx | (cfg.d_cal & 0xF) | ((cfg.txcal45dm & 0xF) << 8) | ((cfg.txcal45dp & 0xF) << 16);
    phy_w(phy_off::TX, tx);
}

/// Power down the USBPHY and gate the USB clocks.
///
/// # Safety
/// Exclusive access to MRCC/USBPHY required.
pub(crate) unsafe fn deinit_clocks_and_phy() {
    use crate::pac::MRCC0;

    // Power down PHY (all bits set powers everything down).
    phy_w(phy_off::PWD, 0xFFFF_FFFF);
    // Power down PLL.
    phy_w(phy_off::PLL_SIC_CLR, PLL_EN_USB_CLKS | PLL_POWER | PLL_REG_ENABLE);
    // Gate the PHY clock.
    phy_w(phy_off::CTRL_SET, CTRL_CLKGATE);

    MRCC0.mrcc_glb_cc2().modify(|w| {
        w.set_usb1(false);
        w.set_usb1_phy(false);
    });
}
