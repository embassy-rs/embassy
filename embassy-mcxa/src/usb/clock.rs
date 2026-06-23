//! USBPHY bring-up for the MCXA5xx USBHS controller.
//!
//! Clock source selection and MRCC gate/reset handling live in the clocks
//! subsystem. This module only performs the USBPHY register sequence from the
//! NXP MCUXpresso SDK `USB_EhciPhyInit` path.

/// PHY trim parameters for the high-speed transmit drivers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct PhyConfig {
    /// `D_CAL` trim value. Must fit in the 4-bit hardware field.
    pub d_cal: u8,
    /// `TXCAL45DP` trim value. Must fit in the 4-bit hardware field.
    pub txcal45dp: u8,
    /// `TXCAL45DM` trim value. Must fit in the 4-bit hardware field.
    pub txcal45dm: u8,
    /// PLL divider select for the reference clock (2 = 24 MHz reference).
    /// Must fit in the 3-bit hardware field.
    pub pll_div_sel: u8,
}

impl Default for PhyConfig {
    fn default() -> Self {
        Self {
            d_cal: 0,
            txcal45dp: 0,
            txcal45dm: 0,
            pll_div_sel: 2,
        }
    }
}

impl PhyConfig {
    /// FRDM-MCXA577 board calibration values, using the 24 MHz crystal reference.
    pub const fn frdm_mcxa577() -> Self {
        Self {
            d_cal: 0x04,
            txcal45dp: 0x07,
            txcal45dm: 0x07,
            pll_div_sel: 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PhyInitError {
    InvalidConfig,
    PllLock,
}

/// Bring up the USBPHY PLL and transmitter trims.
///
/// # Safety
///
/// Must be called after the USBHS/USBPHY MRCC gates and resets are configured,
/// with exclusive access to USBPHY. The SOSC / PHY reference clock must already
/// be running.
pub(crate) unsafe fn init_phy(cfg: &PhyConfig) -> Result<(), PhyInitError> {
    if cfg.d_cal > 0x0f || cfg.txcal45dp > 0x0f || cfg.txcal45dm > 0x0f || cfg.pll_div_sel > 0x07 {
        return Err(PhyInitError::InvalidConfig);
    }

    let phy = crate::pac::USB1_HS_PHY;

    phy.CTRL_CLR().write(|w| w.set_SFTRST(true));
    phy.ANACTRL_SET().write(|w| w.set_LVI_EN(true));
    phy.PLL_SIC_SET().write(|w| w.set_PLL_REG_ENABLE(true));

    // SDK waits at least 15 us for the PLL regulator to stabilize. This is
    // deliberately conservative in cycles because the CPU clock varies by app.
    cortex_m::asm::delay(15_000);

    phy.PLL_SIC_SET().write(|w| w.set_PLL_POWER(true));
    phy.PLL_SIC_CLR().write(|w| {
        w.set_PLL_DIV_SEL(0b111);
        w.set_PLL_BYPASS(true);
    });
    phy.PLL_SIC_SET().write(|w| {
        w.set_PLL_DIV_SEL(cfg.pll_div_sel);
        w.set_PLL_EN_USB_CLKS(true);
    });

    phy.CTRL_CLR().write(|w| w.set_CLKGATE(true));
    phy.PWD().write(|w| w.0 = 0);

    let mut locked = false;
    for _ in 0..4_000_000 {
        if phy.PLL_SIC().read().PLL_LOCK() == crate::pac::usbphy::PLL_LOCK::PLL_LOCKED {
            locked = true;
            break;
        }
        cortex_m::asm::nop();
    }
    if !locked {
        return Err(PhyInitError::PllLock);
    }

    phy.TRIM_OVERRIDE_EN_SET().write(|w| {
        w.set_DIV_SEL_OVERRIDE(true);
        w.set_TX_D_CAL_OVERRIDE(true);
        w.set_TX_CAL45DP_OVERRIDE(true);
        w.set_TX_CAL45DM_OVERRIDE(true);
    });
    phy.CTRL_SET().write(|w| {
        w.set_ENUTMILEVEL2(true);
        w.set_ENUTMILEVEL3(true);
    });
    phy.PWD().write(|w| w.0 = 0);

    phy.TX().modify(|w| {
        w.set_D_CAL(crate::pac::usbphy::D_CAL::from_bits(cfg.d_cal));
        w.set_TXCAL45DN(cfg.txcal45dm);
        w.set_TXCAL45DP(cfg.txcal45dp);
    });

    Ok(())
}
