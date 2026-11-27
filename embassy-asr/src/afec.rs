//! Analog front-end controller (AFEC).
//!
//! The PAC describes the digital AFEC registers at `0x4000_8200`. The vendor
//! SDK also accesses an analog register window starting at `0x4000_8000`,
//! where register index `n` is located at byte offset `n * 4`. That window is
//! not present in the current SVD, so its volatile accesses are kept in the
//! crate-private [`analog`] module below.
//!
//! The SDK publishes bit assignments for three raw status signals. It does not
//! publish bit assignments or write semantics for `CR` or `INT_SR`; therefore
//! this module exposes an opaque interrupt-status snapshot, but no control,
//! interrupt-enable, or interrupt-acknowledge operations.

use crate::{Peri, pac, peripherals};

/// Snapshot of the documented AFEC raw status signals.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
/// Only register indices used by the vendor RCC, ADC, DAC, power, and LCD
/// drivers are constructible. All read-modify-write operations preserve bits
/// outside the supplied mask and run in a critical section so interrupt-side
/// analog updates cannot be lost.
pub(crate) mod analog {
    use core::ptr::{read_volatile, write_volatile};

    use crate::pac;

    const BASE: usize = 0x4000_8000;

    /// A validated register in the AFEC analog window.
    ///
    /// The private field prevents other crate modules from constructing
    /// arbitrary analog-window addresses.
    #[derive(Clone, Copy)]
    pub(crate) struct Register<const INDEX: usize> {
        _private: (),
    }

    impl<const INDEX: usize> Register<INDEX> {
        const fn new() -> Self {
            Self { _private: () }
        }

        #[inline]
        const fn address(self) -> usize {
            BASE + INDEX * core::mem::size_of::<u32>()
        }

        /// Read the complete analog register.
        #[inline]
        pub(crate) fn read(self) -> u32 {
            critical_section::with(|_| {
                enable_clock_unlocked();

                // SAFETY: only the validated constants below can be
                // constructed, and each address follows TREMO_ANALOG_RD.
                unsafe { read_volatile(self.address() as *const u32) }
            })
        }

        /// Replace the selected bits and preserve every other bit.
        ///
        /// Bits in `value` outside `mask` are ignored.
        #[inline]
        pub(crate) fn modify(self, mask: u32, value: u32) {
            critical_section::with(|_| {
                enable_clock_unlocked();

                // SAFETY: only the validated constants below can be
                // constructed, and each address follows TREMO_ANALOG_RD/WR.
                unsafe {
                    let address = self.address() as *mut u32;
                    let old = read_volatile(address);
                    write_volatile(address, (old & !mask) | (value & mask));
                }
            });
        }

        /// Set the selected bits and preserve every other bit.
        #[inline]
        pub(crate) fn set_bits(self, mask: u32) {
            self.modify(mask, mask);
        }

        /// Clear the selected bits and preserve every other bit.
        #[inline]
        pub(crate) fn clear_bits(self, mask: u32) {
            self.modify(mask, 0);
        }
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

    // These are exactly the indices touched through TREMO_ANALOG_RD/WR in the
    // vendor files named below. Purpose descriptions summarize those accesses;
    // they do not assign undocumented register or field names.

    /// Vendor `tremo_rcc.c`: RCO32K and XO32K control.
    pub(crate) const REG_02: Register<0x02> = Register::new();

    /// Vendor `tremo_pwr.c`: XO32K low-power control.
    pub(crate) const REG_03: Register<0x03> = Register::new();

    /// Vendor `tremo_pwr.c`: low-power-run control.
    pub(crate) const REG_05: Register<0x05> = Register::new();

    /// Vendor `tremo_rcc.c`, `tremo_pwr.c`, and `tremo_lcd.c`: oscillator,
    /// low-power-run, and LCD analog control.
    pub(crate) const REG_06: Register<0x06> = Register::new();

    /// Vendor `tremo_lcd.c`: LCD COM-count control.
    pub(crate) const REG_09: Register<0x09> = Register::new();

    /// Vendor `tremo_lcd.c`: LCD COM analog control.
    pub(crate) const REG_0A: Register<0x0a> = Register::new();

    /// Vendor `tremo_lcd.c`: LCD drive, COM, and segment control.
    pub(crate) const REG_0B: Register<0x0b> = Register::new();

    /// Vendor `tremo_pwr.c`: STOP3 preparation.
    pub(crate) const REG_0C: Register<0x0c> = Register::new();

    /// Vendor `tremo_adc.c` and `tremo_dac.c`: ADC and DAC analog control.
    pub(crate) const REG_11: Register<0x11> = Register::new();

    /// Vendor `tremo_adc.c` and `tremo_dac.c`: ADC reference and DAC analog
    /// control.
    pub(crate) const REG_12: Register<0x12> = Register::new();

    /// Vendor `tremo_dac.c`: DAC analog output control.
    pub(crate) const REG_27: Register<0x27> = Register::new();

    /// Vendor `tremo_adc.c`: VBAT/3 measurement-path control.
    pub(crate) const REG_2C: Register<0x2c> = Register::new();
}
