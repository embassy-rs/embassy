//! Reset and Clock Control (RCC)

#![macro_use]
#![allow(missing_docs)] // TODO

use core::mem::MaybeUninit;

mod bd;
pub use bd::*;

#[cfg(any(stm32f0, stm32f1, stm32f3))]
pub const LSI_FREQ: crate::time::Hertz = crate::time::Hertz(40_000);
#[cfg(not(any(stm32f0, stm32f1, stm32f3)))]
pub const LSI_FREQ: crate::time::Hertz = crate::time::Hertz(32_000);

mod reset;
pub use reset::*;

#[cfg(all(not(stm32g5), any(mco, mco1, mco2)))]
mod mco;
use critical_section::CriticalSection;
#[cfg(all(not(stm32g5), any(mco, mco1, mco2)))]
pub use mco::*;

#[cfg(dsihost)]
pub(crate) mod dsi;

#[cfg(all(crs, not(stm32c5), not(stm32g5)))]
mod hsi48;
#[cfg(all(crs, not(stm32c5), not(stm32g5)))]
pub use hsi48::*;

#[cfg_attr(any(stm32f0, stm32f1, stm32f3), path = "f013.rs")]
#[cfg_attr(any(stm32f2, stm32f4, stm32f7), path = "f247.rs")]
#[cfg_attr(stm32c0, path = "c0.rs")]
#[cfg_attr(stm32c5, path = "c5.rs")]
#[cfg_attr(stm32g0, path = "g0.rs")]
#[cfg_attr(stm32g4, path = "g4.rs")]
#[cfg_attr(stm32g5, path = "g5.rs")]
#[cfg_attr(any(stm32h5, stm32h7, stm32h7rs), path = "h.rs")]
#[cfg_attr(any(stm32l0, stm32l1, stm32l4, stm32l5, stm32wb, stm32wl, stm32u0), path = "l.rs")]
#[cfg_attr(stm32u3, path = "u3.rs")]
#[cfg_attr(stm32u5, path = "u5.rs")]
#[cfg_attr(stm32wba, path = "wba.rs")]
#[cfg_attr(stm32n6, path = "n6.rs")]
mod _version;

pub use _version::*;
use stm32_metapac::RCC;

use crate::_generated::RefcountIndex;
pub use crate::_generated::{Clocks, mux};
use crate::time::Hertz;

#[cfg(feature = "low-power")]
/// Number of active `WakeGuard`s blocking each Stop mode (`Stop0..Stop3`).
///
/// Must be written within a critical section.
static mut STOP_BLOCKERS: [u32; 4] = [0; 4];

/// The clocks to bring back after Stop, captured by `init_rcc`.
#[cfg(all(feature = "low-power", not(feature = "_dual-core")))]
static mut STOP_RESTORE: Option<StopRestore> = None;

#[cfg(all(feature = "low-power", feature = "_dual-core"))]
static STOP_RESTORE_PTR: core::sync::atomic::AtomicPtr<MaybeUninit<Option<StopRestore>>> =
    core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());

#[cfg(backup_sram)]
pub(crate) static mut BKSRAM_RETAINED: bool = false;

#[cfg(not(feature = "_dual-core"))]
/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
static mut CLOCK_FREQS: MaybeUninit<Clocks> = MaybeUninit::uninit();

#[cfg(feature = "_dual-core")]
static CLOCK_FREQS_PTR: core::sync::atomic::AtomicPtr<MaybeUninit<Clocks>> =
    core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());

#[cfg(feature = "_dual-core")]
pub(crate) fn set_freqs_ptr(freqs: *mut MaybeUninit<Clocks>) {
    CLOCK_FREQS_PTR.store(freqs, core::sync::atomic::Ordering::SeqCst);
}

#[cfg(all(feature = "low-power", feature = "_dual-core"))]
pub(crate) fn set_stop_restore_ptr(ptr: *mut MaybeUninit<Option<StopRestore>>) {
    STOP_RESTORE_PTR.store(ptr, core::sync::atomic::Ordering::SeqCst);
}

#[cfg(all(not(feature = "_dual-core")))]
/// Sets the clock frequencies
///
/// Safety: Sets a mutable global.
pub(crate) unsafe fn set_freqs(freqs: Clocks) {
    debug!("rcc: {:?}", freqs);
    CLOCK_FREQS = MaybeUninit::new(freqs);
}

#[cfg(feature = "_dual-core")]
/// Sets the clock frequencies
///
/// Safety: Sets a mutable global.
pub(crate) unsafe fn set_freqs(freqs: Clocks) {
    debug!("rcc: {:?}", freqs);
    CLOCK_FREQS_PTR
        .load(core::sync::atomic::Ordering::SeqCst)
        .write(MaybeUninit::new(freqs));
}

#[cfg(not(feature = "_dual-core"))]
/// Safety: Reads a mutable global.
pub(crate) unsafe fn get_freqs() -> &'static Clocks {
    (*&raw const CLOCK_FREQS).assume_init_ref()
}

#[cfg(feature = "_dual-core")]
/// Safety: Reads a mutable global.
pub(crate) unsafe fn get_freqs() -> &'static Clocks {
    unwrap!(CLOCK_FREQS_PTR.load(core::sync::atomic::Ordering::SeqCst).as_ref()).assume_init_ref()
}

#[cfg(all(feature = "low-power", not(feature = "_dual-core")))]
/// Safety: Reads a mutable global.
unsafe fn get_stop_restore() -> Option<StopRestore> {
    STOP_RESTORE
}

#[cfg(all(feature = "low-power", feature = "_dual-core"))]
/// Safety: Reads a mutable global.
unsafe fn get_stop_restore() -> Option<StopRestore> {
    unwrap!(STOP_RESTORE_PTR.load(core::sync::atomic::Ordering::SeqCst).as_ref()).assume_init()
}

#[cfg(all(feature = "low-power", not(feature = "_dual-core")))]
/// Safety: Sets a mutable global.
unsafe fn set_stop_restore(restore: Option<StopRestore>) {
    STOP_RESTORE = restore;
}

#[cfg(all(feature = "low-power", feature = "_dual-core"))]
/// Safety: Sets a mutable global.
unsafe fn set_stop_restore(restore: Option<StopRestore>) {
    STOP_RESTORE_PTR
        .load(core::sync::atomic::Ordering::SeqCst)
        .write(MaybeUninit::new(restore));
}

#[cfg(feature = "low-power")]
#[derive(Clone, Copy, Default)]
pub(crate) struct ClockRestoreTiming {
    pub hse_osc: bool,
    pub hse_bypass: bool,
    pub hsi: bool,
    pub msi: bool,
    pub hsi48: bool,
    pub pll: bool,
    pub vos: bool,
    pub overdrive: bool,
    pub hpre_step: bool,
}

#[cfg(feature = "low-power")]
impl ClockRestoreTiming {
    /// Conservative software-observed RCC clock-restoration latency in microseconds
    /// after Stop exit (`StopRestore::exit()`).
    pub(crate) const fn exit_latency_us(self) -> u32 {
        let mut us = if self.hse_osc {
            2_000
        } else if self.hse_bypass {
            5
        } else {
            0
        };
        let rc_osc = if self.hsi || self.msi { 10 } else { 0 };
        if rc_osc > us {
            us = rc_osc;
        }
        if self.overdrive {
            us += 50;
        } else if self.vos {
            us += 30;
        }
        us += if self.pll {
            100
        } else if self.hsi48 {
            60
        } else {
            0
        };
        if self.hpre_step {
            us += 2;
        }
        us
    }
}

/// Prepare the clocks for Stop. Call right before entering Stop.
///
/// Stop turns off the PLLs and the high-speed oscillators except the wake-up one, and the chip
/// resumes on the wake-up clock. This makes the flash latency valid for the wake-up clock at the
/// voltage scale the chip wakes up in.
#[cfg(feature = "low-power")]
pub(crate) fn stop_enter(_cs: CriticalSection) {
    if let Some(restore) = unsafe { get_stop_restore() } {
        restore.enter();
    }
}

/// Bring the clocks back to what `init` configured, after a Stop or a `WFI` that didn't stop.
///
/// Only turns on and switches what Stop turned off, so it is cheap when nothing changed. The
/// configuration registers (PLL setup, bus prescalers, kernel clock muxes) survive Stop. After
/// this the hardware matches what `clocks()` reports again.
#[cfg(feature = "low-power")]
pub(crate) fn stop_exit(_cs: CriticalSection) {
    if let Some(restore) = unsafe { get_stop_restore() } {
        restore.exit();
    }
}

/// Get the current clock configuration of the chip.
pub fn clocks<'a>(_rcc: &'a crate::Peri<'a, crate::peripherals::RCC>) -> &'a Clocks {
    // Safety: the existence of a `Peri<RCC>` means that `rcc::init()`
    // has already been called, so `CLOCK_FREQS` must be initialized.
    // The clocks could be modified again by `reinit()`, but reinit
    // (for this reason) requires an exclusive reference to `Peri<RCC>`.
    unsafe { get_freqs() }
}

#[cfg(feature = "low-power")]
pub(crate) fn stop_restore_exit_latency_us() -> u32 {
    unsafe { get_stop_restore() }.map_or(0, |restore| restore.exit_latency_us())
}

#[cfg(feature = "low-power")]
pub(crate) fn get_power_mode(_cs: CriticalSection) -> Option<&'static crate::_generated::power::PowerModeEntry> {
    // If rcc is not initialized, then we're not ready to stop
    unsafe { get_stop_restore()? };
    if !crate::_generated::power::HAS_POWER {
        return None;
    }

    // `MODES` is ordered shallowest -> deepest; pick the deepest mode with zero blockers.
    let modes = crate::_generated::power::MODES;
    let mut i = modes.len();
    while i > 0 {
        i -= 1;
        let m = &modes[i];
        let idx = m.kind.depth() as usize;
        if unsafe { STOP_BLOCKERS[idx] } == 0 {
            return Some(m);
        }
    }
    None
}

#[cfg(feature = "low-power")]
pub(crate) fn select_power_mode(
    _cs: CriticalSection,
    time_until_next_alarm_us: Option<u64>,
    driver_overhead_us: u32,
) -> Option<(&'static crate::_generated::power::PowerModeEntry, u32)> {
    unsafe { get_stop_restore()? };
    if !crate::_generated::power::HAS_POWER {
        return None;
    }

    let clock_us = stop_restore_exit_latency_us();
    let modes = crate::_generated::power::MODES;
    let mut i = modes.len();
    while i > 0 {
        i -= 1;
        let m = &modes[i];
        let idx = m.kind.depth() as usize;
        if unsafe { STOP_BLOCKERS[idx] } == 0 {
            let exit_us = m.exit_latency_us.saturating_add(clock_us);
            let required_us = m.min_residency_us as u64 + exit_us as u64 + driver_overhead_us as u64;
            if time_until_next_alarm_us.is_none_or(|avail| avail >= required_us) {
                return Some((m, exit_us));
            }
        }
    }
    None
}

#[cfg(feature = "low-power")]
/// Get the deepest Stop mode currently permitted by active [`WakeGuard`]s.
pub fn get_stop_mode(cs: CriticalSection) -> Option<StopMode> {
    get_power_mode(cs).map(|m| m.kind)
}

#[cfg(feature = "low-power")]
pub(crate) fn increment_stop_blockers(_cs: CriticalSection, blocked: StopModes) {
    for (idx, count) in unsafe { (&raw mut STOP_BLOCKERS).as_mut().unwrap() }.iter_mut().enumerate() {
        if (blocked.0 & (1 << idx)) != 0 {
            *count += 1;
        }
    }
}

#[cfg(feature = "low-power")]
pub(crate) fn decrement_stop_blockers(_cs: CriticalSection, blocked: StopModes) {
    for (idx, count) in unsafe { (&raw mut STOP_BLOCKERS).as_mut().unwrap() }.iter_mut().enumerate() {
        if (blocked.0 & (1 << idx)) != 0 {
            *count -= 1;
        }
    }
}

pub(crate) trait SealedRccPeripheral {
    fn frequency() -> Hertz;
    #[allow(dead_code)]
    fn bus_frequency() -> Hertz;
    const RCC_INFO: RccInfo;
}

#[allow(private_bounds)]
pub trait RccPeripheral: SealedRccPeripheral + 'static {}

/// Bitmask of Stop modes (`Stop0`..`Stop3`) supported by a peripheral or kernel clock.
#[cfg(feature = "low-power")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StopModes(pub u8);

#[cfg(feature = "low-power")]
#[allow(dead_code)]
impl StopModes {
    pub const NONE: Self = Self(0);
    pub const STOP0: Self = Self(1 << 0);
    pub const STOP1: Self = Self(1 << 1);
    pub const STOP2: Self = Self(1 << 2);
    pub const STOP3: Self = Self(1 << 3);
    pub const ALL: Self = Self(0b1111);

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, mode: StopMode) -> bool {
        let bit = mode.bit();
        bit != 0 && (self.0 & bit) != 0
    }

    pub const fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub const fn without(self, allowed: Self) -> Self {
        Self(self.0 & !allowed.0)
    }
}

#[cfg(feature = "low-power")]
#[allow(dead_code)]
pub(crate) fn const_stop_modes<const MASK: u8>() -> StopModes {
    StopModes(MASK)
}

/// Runtime information necessary to reset, enable and disable a peripheral.
pub(crate) struct RccInfo {
    /// Offset in 32-bit words of the xxxRSTR register into the RCC register block, or 0xff if the
    /// peripheral has no reset bit (we don't use an `Option` to save one byte of storage).
    reset_offset_or_0xff: u8,
    /// Position of the xxxRST bit within the xxxRSTR register (0..=31).
    reset_bit: u8,
    /// Offset in 32-bit words of the xxxENR register into the RCC register block.
    enable_offset: u8,
    /// Position of the xxxEN bit within the xxxENR register (0..=31).
    enable_bit: u8,
    /// If this peripheral shares the same xxxRSTR bit and xxxEN bit with other peripherals, we
    /// maintain a refcount in `crate::_generated::REFCOUNTS` at this index.
    refcount_idx: Option<RefcountIndex>,
    /// Offset in 32-bit words of the xxxSMENR/LPENR/STPENR register into RCC, or 0xff if absent.
    #[cfg(feature = "low-power")]
    stop_enable_offset_or_0xff: u8,
    /// Position of the Stop clock enable bit.
    #[cfg(feature = "low-power")]
    stop_enable_bit: u8,
    /// Offset in 32-bit words of the xxxAMR autonomous enable register into RCC, or 0xff if absent.
    #[cfg(feature = "low-power")]
    autonomous_enable_offset_or_0xff: u8,
    /// Position of the autonomous enable bit.
    #[cfg(feature = "low-power")]
    autonomous_enable_bit: u8,
    /// Stop modes supported by this peripheral instance.
    #[cfg(feature = "low-power")]
    lp_modes: StopModes,
    /// Capability flags: bit 0 = autonomous, bit 1 = wake_capable.
    #[cfg(feature = "low-power")]
    lp_flags: u8,
    /// EXTI wakeup line if this peripheral wakes over a dedicated EXTI line.
    #[cfg(feature = "low-power")]
    wake_exti: Option<u8>,
    /// Returns Stop modes in which the peripheral's currently selected kernel clock survives.
    #[cfg(feature = "low-power")]
    kernel_stop_modes: fn() -> StopModes,
}

/// Specifies a limit for the stop mode of the peripheral or the stop mode to be entered.
/// E.g. if `StopMode::Stop1` is selected in [`WakeGuard::new`], Stop 1 and deeper modes are prevented.
#[cfg(feature = "low-power")]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StopMode {
    /// Stop 0 mode (main regulator active)
    Stop0,
    #[default]
    /// Peripheral prevents chip from entering Stop1 or executor will enter Stop1
    Stop1,
    /// Peripheral prevents chip from entering Stop2 or executor will enter Stop2
    Stop2,
    /// Peripheral prevents chip from entering Stop3 or executor will enter Stop3
    Stop3,
    /// Peripheral does not prevent chip from entering Stop
    Standby,
}

#[cfg(feature = "low-power")]
impl StopMode {
    const fn depth(self) -> u8 {
        match self {
            Self::Stop0 => 0,
            Self::Stop1 => 1,
            Self::Stop2 => 2,
            Self::Stop3 => 3,
            Self::Standby => 4,
        }
    }

    pub(crate) const fn bit(self) -> u8 {
        match self {
            Self::Stop0 => 1 << 0,
            Self::Stop1 => 1 << 1,
            Self::Stop2 => 1 << 2,
            Self::Stop3 => 1 << 3,
            Self::Standby => 0,
        }
    }

    /// Return whether this stop mode is at least as deep as another stop mode.
    pub const fn at_least(&self, other: StopMode) -> bool {
        self.depth() >= other.depth()
    }

    const fn blocked_modes(self) -> StopModes {
        match self {
            Self::Stop0 => StopModes::ALL,
            Self::Stop1 => StopModes(StopModes::STOP1.0 | StopModes::STOP2.0 | StopModes::STOP3.0),
            Self::Stop2 => StopModes(StopModes::STOP2.0 | StopModes::STOP3.0),
            Self::Stop3 => StopModes::STOP3,
            Self::Standby => StopModes::NONE,
        }
    }
}

impl RccInfo {
    /// Safety:
    /// - `reset_offset_and_bit`, if set, must correspond to valid xxxRST bit
    /// - `enable_offset_and_bit` must correspond to valid xxxEN bit
    #[allow(clippy::too_many_arguments)]
    pub(crate) const unsafe fn new(
        reset_offset_and_bit: Option<(u8, u8)>,
        enable_offset_and_bit: (u8, u8),
        refcount_idx: Option<RefcountIndex>,
        #[cfg(feature = "low-power")] stop_enable_offset_and_bit: Option<(u8, u8)>,
        #[cfg(feature = "low-power")] autonomous_enable_offset_and_bit: Option<(u8, u8)>,
        #[cfg(feature = "low-power")] lp_modes: StopModes,
        #[cfg(feature = "low-power")] lp_flags: u8,
        #[cfg(feature = "low-power")] wake_exti: Option<u8>,
        #[cfg(feature = "low-power")] kernel_stop_modes: fn() -> StopModes,
    ) -> Self {
        let (reset_offset_or_0xff, reset_bit) = match reset_offset_and_bit {
            Some((offset, bit)) => (offset, bit),
            None => (0xff, 0xff),
        };
        let (enable_offset, enable_bit) = enable_offset_and_bit;
        #[cfg(feature = "low-power")]
        let (stop_enable_offset_or_0xff, stop_enable_bit) = match stop_enable_offset_and_bit {
            Some((offset, bit)) => (offset, bit),
            None => (0xff, 0xff),
        };
        #[cfg(feature = "low-power")]
        let (autonomous_enable_offset_or_0xff, autonomous_enable_bit) = match autonomous_enable_offset_and_bit {
            Some((offset, bit)) => (offset, bit),
            None => (0xff, 0xff),
        };
        Self {
            reset_offset_or_0xff,
            reset_bit,
            enable_offset,
            enable_bit,
            refcount_idx,
            #[cfg(feature = "low-power")]
            stop_enable_offset_or_0xff,
            #[cfg(feature = "low-power")]
            stop_enable_bit,
            #[cfg(feature = "low-power")]
            autonomous_enable_offset_or_0xff,
            #[cfg(feature = "low-power")]
            autonomous_enable_bit,
            #[cfg(feature = "low-power")]
            lp_modes,
            #[cfg(feature = "low-power")]
            lp_flags,
            #[cfg(feature = "low-power")]
            wake_exti,
            #[cfg(feature = "low-power")]
            kernel_stop_modes,
        }
    }

    // TODO: should this be `unsafe`?
    pub(crate) fn enable_and_reset_with_cs(&self, cs: CriticalSection) {
        if let Some(refcount_idx) = self.refcount_idx {
            let refcount_idx = refcount_idx as usize;
            let refcount = unsafe { &mut (*&raw mut crate::_generated::REFCOUNTS)[refcount_idx] };
            *refcount += 1;
            if *refcount > 1 {
                return;
            }
        }

        // set the xxxRST bit
        if let Some(reset_ptr) = self.reset_ptr() {
            // if the other CPU has enabled the peripheral we don't want to reset it as that would reset the configuration
            // that the other CPU has set up. we hold a hardware lock to prevent the other CPU from enabling the peripheral
            // while we are resetting it.
            unsafe {
                #[cfg(stm32wl5x)]
                let _lock = crate::hsem::get_hsem(3).blocking_lock(0);

                if (self.enable_ptr().read_volatile() & (1u32 << self.enable_bit)) == 0 {
                    let val = reset_ptr.read_volatile();
                    reset_ptr.write_volatile(val | 1u32 << self.reset_bit);

                    trace!("rcc: reset 0x{:x}:{}", self.enable_offset, self.enable_bit);
                } else {
                    trace!("rcc: skip reset 0x{:x}:{}", self.enable_offset, self.enable_bit);
                }
            }
        }

        self.enable_with_cs(cs);
    }

    // TODO: should this be `unsafe`?
    pub(crate) fn enable_with_cs(&self, _cs: CriticalSection) {
        // set the xxxEN bit
        unsafe {
            let val = self.enable_ptr().read_volatile();
            self.enable_ptr().write_volatile(val | 1u32 << self.enable_bit);

            trace!("rcc: enabled 0x{:x}:{}", self.enable_offset, self.enable_bit);
        }

        // we must wait two peripheral clock cycles before the clock is active
        // this seems to work, but might be incorrect
        // see http://efton.sk/STM32/gotcha/g183.html

        // dummy read (like in the ST HALs)
        let _ = unsafe { self.enable_ptr().read_volatile() };

        // DSB for good measure
        cortex_m::asm::dsb();

        // clear the xxxRST bit
        if let Some(reset_ptr) = self.reset_ptr() {
            unsafe {
                let val = reset_ptr.read_volatile();
                reset_ptr.write_volatile(val & !(1u32 << self.reset_bit));
            }
        }
    }

    // TODO: should this be `unsafe`?
    pub(crate) fn disable_with_cs(&self, _cs: CriticalSection) {
        if let Some(refcount_idx) = self.refcount_idx {
            let refcount_idx = refcount_idx as usize;
            let refcount = unsafe { &mut (*&raw mut crate::_generated::REFCOUNTS)[refcount_idx] };
            *refcount -= 1;
            if *refcount > 0 {
                return;
            }
        }

        // clear the xxxEN bit
        unsafe {
            let val = self.enable_ptr().read_volatile();
            self.enable_ptr().write_volatile(val & !(1u32 << self.enable_bit));
            trace!("rcc: disabled 0x{:x}:{}", self.enable_offset, self.enable_bit);
        }
    }

    // TODO: should this be `unsafe`?
    pub(crate) fn enable_and_reset(&self) {
        critical_section::with(|cs| self.enable_and_reset_with_cs(cs))
    }

    // TODO: should this be `unsafe`?
    pub(crate) fn disable(&self) {
        critical_section::with(|cs| self.disable_with_cs(cs))
    }

    /// Block entering any Stop mode while an active peripheral transaction requires
    /// system/bus clocks.
    #[allow(dead_code)]
    pub(crate) fn wake_guard(&self) -> WakeGuard {
        WakeGuard::no_stop()
    }

    /// Stop modes supported by both this peripheral and its active kernel clock.
    #[cfg(feature = "low-power")]
    #[allow(dead_code)]
    pub(crate) fn surviving_stop_modes(&self) -> StopModes {
        self.lp_modes.intersect((self.kernel_stop_modes)())
    }

    #[cfg(feature = "low-power")]
    #[allow(dead_code)]
    pub(crate) const fn is_wake_capable(&self) -> bool {
        (self.lp_flags & (1 << 1)) != 0
    }

    #[cfg(feature = "low-power")]
    #[allow(dead_code)]
    pub(crate) const fn is_autonomous(&self) -> bool {
        (self.lp_flags & (1 << 0)) != 0
    }

    #[cfg(feature = "low-power")]
    #[allow(dead_code)]
    pub(crate) const fn wake_exti(&self) -> Option<u8> {
        self.wake_exti
    }

    /// Acquire a `WakeGuard` capping Stop to the modes in which this peripheral
    /// and its currently selected kernel clock survive (blocking Stop completely
    /// if none survive).
    #[allow(dead_code)]
    pub(crate) fn stop_wake_guard(&self) -> WakeGuard {
        #[cfg(feature = "low-power")]
        {
            WakeGuard::for_modes(self.surviving_stop_modes())
        }
        #[cfg(not(feature = "low-power"))]
        {
            WakeGuard::no_stop()
        }
    }

    /// Enable the peripheral's Stop-mode clock gating bit (`*SMENR`/`*LPENR`/`*STPENR`)
    /// and autonomous-mode bit (`*AMR`) if present.
    #[cfg(feature = "low-power")]
    #[allow(dead_code)]
    pub(crate) fn enable_stop_and_autonomous(&self) {
        if self.stop_enable_offset_or_0xff != 0xff {
            unsafe {
                let ptr = self.reg_ptr(self.stop_enable_offset_or_0xff, true);
                let val = ptr.read_volatile();
                ptr.write_volatile(val | (1u32 << self.stop_enable_bit));
            }
        }
        if self.autonomous_enable_offset_or_0xff != 0xff {
            unsafe {
                let ptr = self.reg_ptr(self.autonomous_enable_offset_or_0xff, false);
                let val = ptr.read_volatile();
                ptr.write_volatile(val | (1u32 << self.autonomous_enable_bit));
            }
        }
    }

    const fn reset_ptr(&self) -> Option<*mut u32> {
        if self.reset_offset_or_0xff != 0xff {
            Some(self.reg_ptr(self.reset_offset_or_0xff, false))
        } else {
            None
        }
    }

    const fn enable_ptr(&self) -> *mut u32 {
        self.reg_ptr(self.enable_offset, true)
    }

    const fn reg_ptr(&self, offset: u8, _mirror_wl_cm0p: bool) -> *mut u32 {
        #[cfg(all(stm32wl5x, feature = "_core-cm0p"))]
        if _mirror_wl_cm0p {
            return unsafe { (RCC.as_ptr() as *mut u32).add(offset as _).add(0x100 / 4) };
        }
        unsafe { (RCC.as_ptr() as *mut u32).add(offset as _) }
    }
}

pub struct WakeGuard {
    #[cfg(feature = "low-power")]
    blocked: StopModes,
}

impl WakeGuard {
    pub fn new(#[cfg(feature = "low-power")] stop_mode: StopMode) -> Self {
        #[cfg(feature = "low-power")]
        let blocked = stop_mode.blocked_modes();
        #[cfg(feature = "low-power")]
        critical_section::with(|cs| increment_stop_blockers(cs, blocked));

        Self {
            #[cfg(feature = "low-power")]
            blocked,
        }
    }

    #[cfg(feature = "low-power")]
    pub(crate) fn for_modes(allowed: StopModes) -> Self {
        let blocked = StopModes::ALL.without(allowed);
        critical_section::with(|cs| increment_stop_blockers(cs, blocked));
        Self { blocked }
    }

    /// Keeps the chip out of every Stop mode.
    pub fn no_stop() -> Self {
        Self::new(
            #[cfg(feature = "low-power")]
            StopMode::Stop0,
        )
    }
}

impl Drop for WakeGuard {
    fn drop(&mut self) {
        #[cfg(feature = "low-power")]
        critical_section::with(|cs| decrement_stop_blockers(cs, self.blocked));
    }
}

impl From<WakeGuard> for MaybeWakeGuard {
    fn from(wake_guard: WakeGuard) -> Self {
        #[cfg(not(feature = "low-power"))]
        let _ = wake_guard;

        MaybeWakeGuard {
            #[cfg(feature = "low-power")]
            blocked: wake_guard.blocked,
            #[cfg(feature = "low-power")]
            enabled: false,
        }
    }
}

pub struct MaybeWakeGuard {
    #[cfg(feature = "low-power")]
    blocked: StopModes,
    #[cfg(feature = "low-power")]
    enabled: bool,
}

impl MaybeWakeGuard {
    pub fn enable(&mut self) {
        #[cfg(feature = "low-power")]
        if !core::mem::replace(&mut self.enabled, true) {
            critical_section::with(|cs| increment_stop_blockers(cs, self.blocked));
        }
    }

    pub fn disable(&mut self) {
        #[cfg(feature = "low-power")]
        if core::mem::replace(&mut self.enabled, false) {
            critical_section::with(|cs| decrement_stop_blockers(cs, self.blocked));
        }
    }
}

impl Drop for MaybeWakeGuard {
    fn drop(&mut self) {
        self.disable();
    }
}

#[allow(unused)]
mod util {
    use crate::time::Hertz;

    pub fn calc_pclk<D>(hclk: Hertz, ppre: D) -> (Hertz, Hertz)
    where
        Hertz: core::ops::Div<D, Output = Hertz>,
    {
        let pclk = hclk / ppre;
        let pclk_tim = if hclk == pclk { pclk } else { pclk * 2u32 };
        (pclk, pclk_tim)
    }

    pub fn all_equal<T: Eq>(mut iter: impl Iterator<Item = T>) -> bool {
        let Some(x) = iter.next() else { return true };
        if !iter.all(|y| y == x) {
            return false;
        }
        true
    }

    pub fn get_equal<T: Eq>(mut iter: impl Iterator<Item = T>) -> Result<Option<T>, ()> {
        let Some(x) = iter.next() else { return Ok(None) };
        if !iter.all(|y| y == x) {
            return Err(());
        }
        Ok(Some(x))
    }

    #[allow(dead_code)]
    /// Computes VCO frequency for PLLs with a 13-bit fractional multiplier (`fracn < 8192`).
    pub fn calc_pll_vco(ref_clk: Hertz, mul: u16, fracn: Option<u16>) -> Hertz {
        match fracn {
            Some(fracn) => {
                assert!(fracn < 8192);
                let vco_hz = ((u64::from(ref_clk.0) * ((u64::from(mul) << 13) + u64::from(fracn))) >> 13) as u32;
                Hertz::hz(vco_hz)
            }
            None => ref_clk * u32::from(mul),
        }
    }
}

/// Get the kernel clock frequency of the peripheral `T`.
///
/// # Panics
///
/// Panics if the clock is not active.
pub fn frequency<T: RccPeripheral>() -> Hertz {
    T::frequency()
}

/// Enables and resets peripheral `T` without touching low-power state.
#[allow(dead_code)]
pub(crate) fn enable_and_reset_with_cs_no_refcount<T: RccPeripheral>(cs: CriticalSection) {
    T::RCC_INFO.enable_and_reset_with_cs(cs);
}

/// Enables and resets peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
// TODO: should this be `unsafe`?
pub fn enable_and_reset_with_cs<T: RccPeripheral>(cs: CriticalSection) {
    T::RCC_INFO.enable_and_reset_with_cs(cs);
}

/// Enables and clears the reset for peripheral `T` without touching low-power state.
#[allow(dead_code)]
pub(crate) fn enable_with_cs_no_refcount<T: RccPeripheral>(cs: CriticalSection) {
    T::RCC_INFO.enable_with_cs(cs);
}

/// Enables and clears the reset for peripheral `T`.
///
/// # Safety
///
/// The peripheral can be in use since this does not reset it
pub fn enable_with_cs<T: RccPeripheral>(cs: CriticalSection) {
    T::RCC_INFO.enable_with_cs(cs);
}

/// Disables peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
// TODO: should this be `unsafe`?
pub fn disable_with_cs<T: RccPeripheral>(cs: CriticalSection) {
    let _ = T::RCC_INFO.disable_with_cs(cs);
}

/// Enables and resets peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
// TODO: should this be `unsafe`?
pub fn enable_and_reset<T: RccPeripheral>() {
    T::RCC_INFO.enable_and_reset();
}

/// Disables peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
// TODO: should this be `unsafe`?
pub fn disable<T: RccPeripheral>() {
    T::RCC_INFO.disable();
}

/// Re-initialize the `embassy-stm32` clock configuration with the provided configuration.
///
/// This is useful when you need to alter the CPU clock after configuring peripherals.
/// For instance, configure an external clock via spi or i2c.
///
/// Please not this only re-configures the rcc and the time driver (not GPIO, EXTI, etc).
///
/// This should only be called after `init`.
#[cfg(not(feature = "_dual-core"))]
pub fn reinit(config: Config, _rcc: &'_ mut crate::Peri<'_, crate::peripherals::RCC>) {
    critical_section::with(|cs| {
        init_rcc(cs, config);

        // must be after rcc init
        #[cfg(feature = "_time-driver")]
        crate::time_driver::init(cs);
    })
}

pub(crate) fn init_rcc(_cs: CriticalSection, config: Config) {
    unsafe {
        init(config);

        #[cfg(feature = "low-power")]
        set_stop_restore(Some(StopRestore::capture()));
    }
}

/// Calculate intermediate prescaler number used to calculate peripheral prescalers
///
/// This function is intended to calculate a number indicating a minimum division
/// necessary to result in a frequency lower than the provided `freq_max`.
///
/// The returned value indicates the `val + 1` divider is necessary to result in
/// the output frequency that is below the maximum provided.
///
/// For example:
/// 0 = divider of 1 => no division necessary as the input frequency is below max
/// 1 = divider of 2 => division by 2 necessary
/// ...
///
/// The provided max frequency is inclusive. So if `freq_in == freq_max` the result
/// will be 0, indicating that no division is necessary. To accomplish that we subtract
/// 1 from the input frequency so that the integer rounding plays in our favor.
///
/// For example:
/// Let the input frequency be 110 and the max frequency be 55.
/// If we naiively do `110/55 = 2` the renult will indicate that we need a divider by 3
/// which in reality will be rounded up to 4 as usually a 3 division is not available.
/// In either case the resulting frequency will be either 36 or 27 which is lower than
/// what we would want. The result should be 1.
/// If we do the following instead `109/55 = 1` indicating that we need a divide by 2
/// which will result in the correct 55.
#[allow(unused)]
pub(crate) fn raw_prescaler(freq_in: u32, freq_max: u32) -> u32 {
    freq_in.saturating_sub(1) / freq_max
}

#[cfg(all(test, feature = "low-power"))]
mod tests {
    use super::ClockRestoreTiming;

    #[test]
    fn clock_restore_latency_reflects_oscillators_and_pll() {
        let rc_only = ClockRestoreTiming {
            hsi: true,
            ..Default::default()
        };
        assert_eq!(rc_only.exit_latency_us(), 10);

        let hsi_pll = ClockRestoreTiming {
            hsi: true,
            pll: true,
            hpre_step: true,
            ..Default::default()
        };
        assert_eq!(hsi_pll.exit_latency_us(), 112);

        let hse_bypass_pll = ClockRestoreTiming {
            hse_bypass: true,
            hsi: true,
            pll: true,
            ..Default::default()
        };
        assert_eq!(hse_bypass_pll.exit_latency_us(), 110);

        let hse_crystal_pll_vos = ClockRestoreTiming {
            hse_osc: true,
            hsi: true,
            pll: true,
            vos: true,
            ..Default::default()
        };
        assert_eq!(hse_crystal_pll_vos.exit_latency_us(), 2_130);
    }
}
