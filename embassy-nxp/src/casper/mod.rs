//! Cryptographic Accelerator and Signal Processing Engine (CASPER)
//!
//! This module provides hardware acceleration for asymmetric cryptography (RSA).

use crate::pac;
use embassy_hal_internal::{impl_peripheral, Peri};

/// Custom peripheral marker struct for CASPER hardware block
pub struct CASPER;

impl_peripheral!(CASPER);

pub struct CasperDriver<'d> {
    _peri: Peri<'d, CASPER>,
}

impl<'d> CasperDriver<'d> {
    /// Create a new driver instance, enable clocks and apply hardware reset
    pub fn new(peri: impl Into<Peri<'d, CASPER>>) -> Self {
        let peri = peri.into();

        // Get access to SYSCON PAC instance
        let syscon = pac::SYSCON;

        // 1. Enable clock for CASPER in AHBCLKCTRL2 register (bit 24 LPC55S6xLPC55S2xLPC552x User manual)
        syscon.ahbclkctrl2().modify(|w| w.set_casper(true));

        // 2. Reset the CASPER block in PRESETCTRL2 register (bit 24 in LPC55S6xLPC55S2xLPC552x User manual)
        syscon.presetctrl2().modify(|w| w.set_casper_rst(pac::syscon::vals::CasperRst::ASSERTED));   // Activate reset
        syscon.presetctrl2().modify(|w| w.set_casper_rst(pac::syscon::vals::CasperRst::RELEASED));  // Release reset

        // Test reading the status register to verify the block is responding
        let _status = pac::CASPER.status().read();

        Self { _peri: peri }
    }
}