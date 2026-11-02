//! One-channel SRAM ECC error injection.
//!
//! Injection affects controller-wide reads, not a selected address or stored bits.
//! Dropping this driver is defensive cleanup, never a bound on an unsafe armed
//! interval: destructors can be skipped (including by `mem::forget`).

use crate::clocks::periph_helpers::NoConfig;
use crate::clocks::{ClockError, WakeGuard, default_register_value, enable_and_reset, enable_ecc_ram_a_clock};
#[cfg(feature = "_mcxa5xx")]
use crate::pac::eim::Eim as Regs;
#[cfg(not(feature = "_mcxa5xx"))]
use crate::pac::eim0::Eim0 as Regs;
use crate::{Peri, pac, peripherals};

struct Info {
    regs: Regs,
}

static INFO: Info = Info { regs: pac::EIM0 };

/// A validated data or checkbit identity.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Bit(u8);

impl Bit {
    /// Data bit 0 through 31; returns `None` for an out-of-range index.
    pub const fn data(index: u8) -> Option<Self> {
        if index < 32 { Some(Self(index)) } else { None }
    }

    /// Checkbit 0 through 6; returns `None` for an out-of-range index.
    pub const fn checkbit(index: u8) -> Option<Self> {
        if index < 7 { Some(Self(32 + index)) } else { None }
    }
}

/// Invalid two-bit descriptor.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum MaskError {
    /// The two identities name the same bit.
    DuplicateBit,
}

/// Exactly one or two distinct inverted bits.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Fault {
    data_mask: u32,
    checkbit_mask: u8,
}

impl Fault {
    /// A single-bit fault.
    pub const fn single(bit: Bit) -> Self {
        if bit.0 < 32 {
            Self {
                data_mask: 1u32 << bit.0,
                checkbit_mask: 0,
            }
        } else {
            Self {
                data_mask: 0,
                checkbit_mask: 1u8 << (bit.0 - 32),
            }
        }
    }

    /// A two-bit fault; duplicate identities are rejected.
    pub const fn double(first: Bit, second: Bit) -> Result<Self, MaskError> {
        if first.0 == second.0 {
            Err(MaskError::DuplicateBit)
        } else {
            let first = Self::single(first);
            let second = Self::single(second);
            Ok(Self {
                data_mask: first.data_mask | second.data_mask,
                checkbit_mask: first.checkbit_mask | second.checkbit_mask,
            })
        }
    }

    /// Logical data mask.
    pub const fn data_mask(self) -> u32 {
        self.data_mask
    }

    /// Logical seven-bit checkbit mask (not shifted to the register position).
    pub const fn checkbit_mask(self) -> u8 {
        self.checkbit_mask
    }
}

/// Initialization failure.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Clock prerequisite failed.
    ClockSetup(ClockError),
}

/// Exclusive owner of EIM channel zero.
pub struct Eim<'d> {
    info: &'static Info,
    _peri: Peri<'d, peripherals::EIM0>,
    _rama: WakeGuard,
    _wg: Option<WakeGuard>,
}

impl<'d> Eim<'d> {
    /// Resets EIM, leaving both enables off and both descriptors zero.  This
    /// discards any previously configured injection descriptor without changing
    /// the SRAM ECC policy. Retains the RAM A clock and a wake vote.
    pub fn new(peri: Peri<'d, peripherals::EIM0>) -> Result<Self, SetupError> {
        let rama = enable_ecc_ram_a_clock().map_err(SetupError::ClockSetup)?;
        // SAFETY: the peripheral token proves exclusive ownership before use.
        let parts = unsafe { enable_and_reset::<peripherals::EIM0>(&NoConfig).map_err(SetupError::ClockSetup)? };
        let mut this = Self {
            info: &INFO,
            _peri: peri,
            _rama: rama,
            _wg: parts.wake_guard,
        };
        this.disable();
        this.info.regs.eichd0_word0().write(|w| w.set_chkbit_mask(0));
        this.info.regs.eichd0_word1().write(|w| w.set_b0_3data_mask(0));
        Ok(this)
    }

    /// Disables both enables before writing either descriptor; remains disabled.
    pub fn configure(&mut self, fault: Fault) {
        self.disable();
        self.info
            .regs
            .eichd0_word0()
            .write(|w| w.set_chkbit_mask(fault.checkbit_mask()));
        self.info
            .regs
            .eichd0_word1()
            .write(|w| w.set_b0_3data_mask(fault.data_mask()));
    }

    /// Configures the complete descriptor and enables channel then global
    /// injection.
    ///
    /// # Safety
    ///
    /// All affected reads by CPU, DMA and debugger must be controlled until an
    /// explicit `disable`. Code, vectors, stack and unrelated accesses must not
    /// use affected ECC memory. Two-bit faults can cause uncorrectable
    /// exceptions.  The caller must handle faults, NMI and debugger halt; Drop
    /// is not a safety guarantee and forgetting this owner does not disable
    /// injection.
    pub unsafe fn enable(&mut self, fault: Fault) {
        self.configure(fault);
        cortex_m::asm::dsb();
        self.info.regs.eichen().write(|w| w.set_eich0en(true));
        self.info.regs.eimcr().write(|w| w.set_geien(true));
        cortex_m::asm::dsb();
    }

    /// Disables global injection, then the channel.
    pub fn disable(&mut self) {
        self.info.regs.eimcr().write(|w| w.set_geien(false));
        cortex_m::asm::dsb();
        self.info.regs.eichen().write(|w| w.set_eich0en(false));
        cortex_m::asm::dsb();
    }

    /// True only when both global and channel enables are set.
    pub fn is_enabled(&self) -> bool {
        self.info.regs.eimcr().read().geien() && self.info.regs.eichen().read().eich0en()
    }

    /// Programs a single-bit fault and injects exactly one CPU word load.
    /// Replaces any previous descriptor, including a two-bit descriptor.  Both
    /// enables are off on normal return. Ordinary interrupts retain their
    /// previous mask state after disarming. Only normal return is bounded;
    /// fault or halt can prevent disarming, and Drop cannot guarantee cleanup.
    ///
    /// # Safety
    ///
    /// `target` must be valid for a word read, word aligned, initialized with a
    /// full-word write, and in ECC-enabled SRAM. Instruction fetch must be
    /// outside affected ECC memory. Quiesce DMA, debugger reads and other
    /// masters. No conflicting NMI, fault or halt may occur during the armed
    /// window. ECC policy is not changed.
    pub unsafe fn inject_read(&mut self, target: *const u32, bit: Bit) -> u32 {
        critical_section::with(|_| {
            // Never reactivate a previously configured double-bit descriptor.
            self.configure(Fault::single(bit));
            let channel = self.info.regs.eichen();
            let global = self.info.regs.eimcr();
            let mut channel_on = default_register_value(channel);
            channel_on.set_eich0en(true);
            let mut global_on = default_register_value(global);
            global_on.set_geien(true);
            let value: u32;
            // All pointers and values are in registers before arming. `out`, not
            // `lateout`, prevents the load result clobbering a disarm operand.
            unsafe {
                core::arch::asm!(
                    "dsb sy",
                    "str {channel_on}, [{channel}]",
                    "str {global_on}, [{global}]",
                    "dsb sy",
                    "ldr {value}, [{target}]",
                    "dsb sy",
                    "str {off}, [{global}]",
                    "dsb sy",
                    "str {off}, [{channel}]",
                    "dsb sy",
                    channel = in(reg) channel.as_ptr(),
                    global = in(reg) global.as_ptr(),
                    channel_on = in(reg) channel_on.0,
                    global_on = in(reg) global_on.0,
                    off = in(reg) 0u32,
                    target = in(reg) target,
                    value = out(reg) value,
                    options(nostack),
                );
            }
            value
        })
    }
}

impl Drop for Eim<'_> {
    fn drop(&mut self) {
        self.disable();
    }
}
