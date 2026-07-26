//! RAM configuration (RAMCFG)
//!
//! RAMCFG lets you mass-erase an SRAM bank, tune its wait-state count, and (SRAM2 only) detect
//! parity errors and write-protect individual 1 KB pages.
//!
//! Currently only implemented for WBA, which has two RAMCFG-managed banks: SRAM1 (erase and
//! wait-state only) and SRAM2 (also parity-error detection and write protection). A third bank,
//! SRAM6 — feeding the 2.4 GHz radio's TX/RX and sequencer RAMs — exists in hardware (wait-state
//! configuration only) but isn't currently exposed by the PAC.
//!
//! U5 and N6 also have RAMCFG, but with a meaningfully different register layout (more banks,
//! and ECC rather than parity error detection on some of them); that's not covered here yet.

use core::marker::PhantomData;

use embassy_hal_internal::Peri;

use crate::pac;
use crate::pac::ramcfg::vals::Id;
use crate::peripherals::RAMCFG;

fn regs() -> pac::ramcfg::Ramcfg {
    pac::RAMCFG
}

const ERASE_KEY1: u8 = 0xCA;
const ERASE_KEY2: u8 = 0x53;

/// RAMCFG driver.
pub struct RamCfg<'d> {
    _peri: Peri<'d, RAMCFG>,
}

impl<'d> RamCfg<'d> {
    /// Create a new RAMCFG driver.
    pub fn new(_peri: Peri<'d, RAMCFG>) -> Self {
        crate::rcc::enable_and_reset::<RAMCFG>();
        Self { _peri }
    }

    /// SRAM1 bank: mass erase and wait-state configuration only.
    pub fn sram1(&mut self) -> Sram1<'_> {
        Sram1(PhantomData)
    }

    /// SRAM2 bank: mass erase, wait-state configuration, parity-error detection and per-page
    /// write protection.
    pub fn sram2(&mut self) -> Sram2<'_> {
        Sram2(PhantomData)
    }
}

/// SRAM1 bank handle. See [`RamCfg::sram1()`].
pub struct Sram1<'a>(PhantomData<&'a mut ()>);

impl Sram1<'_> {
    /// Launch a mass erase (fills the bank with zeros) and block until it completes.
    pub fn erase(&mut self) {
        let r = regs();
        r.m1erkeyr().write(|w| w.set_erasekey(ERASE_KEY1));
        r.m1erkeyr().write(|w| w.set_erasekey(ERASE_KEY2));
        r.m1cr().modify(|w| w.set_sramer(true));
        while r.m1isr().read().srambusy() {}
    }

    /// Set the wait-state count (0-7) for accesses to this bank.
    pub fn set_wait_states(&mut self, wait_states: u8) {
        assert!(wait_states <= 7);
        regs().m1cr().modify(|w| w.set_wsc(wait_states));
    }
}

/// SRAM2 bank handle. See [`RamCfg::sram2()`].
pub struct Sram2<'a>(PhantomData<&'a mut ()>);

impl Sram2<'_> {
    /// Launch a mass erase (fills the bank with zeros, including write-protected pages) and
    /// block until it completes.
    pub fn erase(&mut self) {
        let r = regs();
        r.m2erkeyr().write(|w| w.set_erasekey(ERASE_KEY1));
        r.m2erkeyr().write(|w| w.set_erasekey(ERASE_KEY2));
        r.m2cr().modify(|w| w.set_sramer(true));
        while r.m2isr().read().srambusy() {}
    }

    /// Set the wait-state count (0-7) for accesses to this bank.
    pub fn set_wait_states(&mut self, wait_states: u8) {
        assert!(wait_states <= 7);
        regs().m2cr().modify(|w| w.set_wsc(wait_states));
    }

    /// Enable write protection for `count` consecutive 1 KB pages starting at `start_page`.
    ///
    /// There is no way to disable write protection once enabled, other than a peripheral or
    /// system reset. Any write to a protected page causes a `HardFault`.
    ///
    /// Panics if `start_page + count` exceeds 64. Chips with a smaller SRAM2 (and thus fewer
    /// pages) simply ignore page numbers past their actual page count.
    pub fn enable_write_protection(&mut self, start_page: u8, count: u8) {
        let end = start_page as u32 + count as u32;
        assert!(end <= 64, "RAMCFG SRAM2 write-protect page out of range");

        let r = regs();
        let mut wpr1 = r.m2wpr1().read();
        let mut wpr2 = r.m2wpr2().read();
        for page in start_page..(start_page + count) {
            if page < 32 {
                wpr1.set_pwp(page as usize, true);
            } else {
                wpr2.set_pwp((page - 32) as usize, true);
            }
        }
        r.m2wpr1().write_value(wpr1);
        r.m2wpr2().write_value(wpr2);
    }

    /// Enable or disable latching of the failing address on a parity error (`M2PEAR`). Without
    /// this, parity errors are still detected and flagged, but the address isn't recorded.
    pub fn set_latch_parity_error_address(&mut self, enable: bool) {
        regs().m2cr().modify(|w| w.set_ale(enable));
    }

    /// Enable or disable parity-error interrupt sources: `error` for a regular interrupt, `nmi`
    /// to instead (or additionally) redirect it to the NMI.
    pub fn set_interrupts(&mut self, error: bool, nmi: bool) {
        regs().m2ier().modify(|w| {
            w.set_peie(error);
            w.set_penmi(nmi);
        });
    }

    /// Returns `true` if a parity error has been detected.
    pub fn parity_error(&self) -> bool {
        regs().m2isr().read().ped()
    }

    /// Clear the parity-error flag. Once cleared, a new failing address can be latched (see
    /// [`Self::set_latch_parity_error_address()`]).
    pub fn clear_parity_error(&mut self) {
        regs().m2icr().write(|w| w.set_cped(true));
    }

    /// Get the address (and metadata) of the latched parity error.
    ///
    /// Only meaningful once [`Self::parity_error()`] is `true` and address latching is enabled.
    pub fn parity_error_address(&self) -> ParityErrorAddress {
        let pear = regs().m2pear().read();
        ParityErrorAddress {
            offset: pear.pea(),
            bus_master: pear.id(),
            byte: pear.byte(),
        }
    }
}

/// Parity error location, read from `M2PEAR`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ParityErrorAddress {
    /// Word-aligned SRAM2 offset of the failing access.
    pub offset: u16,
    /// AHB bus master that performed the failing access.
    pub bus_master: Id,
    /// Bitmask of which byte(s) within the word failed parity.
    pub byte: u8,
}
