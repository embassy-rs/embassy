//! Analog front-end controller (AFEC).
//!
//! The PAC describes both the digital AFEC registers at `0x4000_8200` and the
//! analog control window starting at `0x4000_8000`, where register index `n`
//! is located at byte offset `n * 4`. This module exposes the digital block
//! publicly and keeps access to the shared analog window crate-private.
//!
//! The SDK publishes bit assignments for three raw status signals. It does not
//! publish bit assignments or write semantics for `CR` or `INT_SR`; therefore
//! this module exposes an opaque interrupt-status snapshot, but no control,
//! interrupt-enable, or interrupt-acknowledge operations.

use crate::{Peri, pac, peripherals};

/// Snapshot of the documented AFEC raw status signals.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawStatus {
    rco24m_ready: bool,
    pll_unlocked: bool,
    rco4m_ready: bool,
}

impl RawStatus {
    /// Whether the SDK's `RCO24M_READY` signal is asserted.
    ///
    /// The vendor RCC driver uses this signal while enabling or disabling the
    /// oscillator exposed there as RCO48M.
    #[inline]
    pub const fn rco24m_ready(self) -> bool {
        self.rco24m_ready
    }

    /// Whether the PLL-unlock signal is asserted.
    #[inline]
    pub const fn pll_unlocked(self) -> bool {
        self.pll_unlocked
    }

    /// Whether the RCO4M-ready signal is asserted.
    #[inline]
    pub const fn rco4m_ready(self) -> bool {
        self.rco4m_ready
    }
}

/// Opaque snapshot of the AFEC interrupt status register.
///
/// The vendor SDK identifies `INT_SR` as a read/write interrupt-status
/// register, but publishes neither bit assignments nor write semantics for it.
/// Consequently, the value can be inspected, but this driver does not provide
/// field accessors or a way to write it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterruptStatus(u32);

impl InterruptStatus {
    /// Return the unmodified `INT_SR` register value.
    #[inline]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Whether no AFEC interrupt-status bit is asserted.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// ASR6601 analog front-end controller.
///
/// Creating the driver enables the AFEC peripheral clock. The clock remains
/// enabled when the driver is dropped because the analog register window is
/// shared by the RCC, ADC, DAC, power, and LCD blocks.
pub struct Afec<'d> {
    _peri: Peri<'d, peripherals::AFEC>,
}

impl<'d> Afec<'d> {
    /// Create an AFEC driver.
    ///
    /// This deliberately does not reset AFEC: the analog window controls
    /// oscillators that may already be supplying system clocks.
    pub fn new(peri: Peri<'d, peripherals::AFEC>) -> Self {
        analog::enable_clock();
        Self { _peri: peri }
    }

    /// Read the documented raw AFEC status signals.
    #[inline]
    pub fn raw_status(&self) -> RawStatus {
        let status = Self::regs().raw_sr().read();
        RawStatus {
            rco24m_ready: status.rco24m_ready().bit_is_set(),
            pll_unlocked: status.pll_unlock().bit_is_set(),
            rco4m_ready: status.rco4m_ready().bit_is_set(),
        }
    }

    /// Read the opaque AFEC interrupt status register.
    #[inline]
    pub fn interrupt_status(&self) -> InterruptStatus {
        InterruptStatus(Self::regs().int_sr().read().bits())
    }

    #[inline]
    fn regs() -> pac::Afec {
        // The lifetime-owned AFEC token guarantees exclusive public driver
        // ownership. `steal` is used only to obtain the PAC register handle.
        unsafe { pac::Afec::steal() }
    }
}

/// Safe crate-internal access to the vendor-defined AFEC analog window.
///
/// Analog settings are shared by several otherwise independent peripherals.
/// [`with`] enables the AFEC clock and serializes the register access so
/// interrupt-side analog updates cannot be lost.
pub(crate) mod analog {
    use crate::pac;

    /// Run `f` with the AFEC clock enabled and the analog window accessible.
    ///
    /// The callback runs inside a critical section so multi-register updates
    /// are atomic with respect to interrupt handlers using the same window.
    pub(crate) fn with<R>(f: impl FnOnce(&pac::Afec) -> R) -> R {
        critical_section::with(|_| {
            enable_clock_unlocked();
            // SAFETY: the AFEC clock is enabled and the handle is not retained.
            let afec = unsafe { pac::Afec::steal() };
            f(&afec)
        })
    }

    /// Enable the AFEC clock without requiring ownership of its digital block.
    ///
    /// Analog settings are shared by several otherwise independent
    /// peripherals, so their drivers use this function before analog access.
    pub(crate) fn enable_clock() {
        critical_section::with(|_| enable_clock_unlocked());
    }

    #[inline]
    fn enable_clock_unlocked() {
        // SAFETY: this handle is used for one critical-section-protected,
        // idempotent clock-bit update and is not retained.
        let rcc = unsafe { pac::Rcc::steal() };
        rcc.cgr0().modify(|_, w| w.afec_clk_en().set_bit());
    }
}
