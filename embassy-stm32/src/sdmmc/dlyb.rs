//! DLYB tuning helpers used by SDMMC for SDR50 / SDR104 sampling.
//! Lock-mode procedure per RM0486 §31.4.4.

use crate::sdmmc::Error;

const POLL_LIMIT: u32 = 100_000;

pub(crate) struct Dlyb {
    regs: crate::pac::dlybsd::Dlybsd,
}

impl Dlyb {
    pub(crate) fn new(regs: crate::pac::dlybsd::Dlybsd) -> Self {
        Self { regs }
    }

    pub(crate) fn enable_lock(&mut self) -> Result<(), Error> {
        self.regs.cfg().modify(|w| {
            w.set_sdmmc_dll_byp_en(false);
            w.set_sdmmc_dll_en(true);
        });
        for _ in 0..POLL_LIMIT {
            if self.regs.status().read().sdmmc_dll_lock() {
                return Ok(());
            }
        }
        Err(Error::Timeout)
    }

    pub(crate) fn disable(&mut self) {
        self.regs.cfg().modify(|w| w.set_sdmmc_dll_en(false));
    }

    pub(crate) fn set_tap(&mut self, tap: u8) -> Result<(), Error> {
        self.regs.cfg().modify(|w| w.set_sdmmc_rx_tap_sel(tap));
        for _ in 0..POLL_LIMIT {
            if self.regs.status().read().sdmmc_rx_tap_sel_ack() {
                return Ok(());
            }
        }
        Err(Error::Timeout)
    }
}
