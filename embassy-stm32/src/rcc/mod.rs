//! Reset and Clock Control (RCC)

#![macro_use]
#![allow(missing_docs)] // TODO

use core::mem::MaybeUninit;

mod bd;
pub use bd::*;

#[cfg(any(mco, mco1, mco2))]
mod mco;
use critical_section::CriticalSection;
#[cfg(any(mco, mco1, mco2))]
pub use mco::*;

#[cfg(crs)]
mod hsi48;
#[cfg(crs)]
pub use hsi48::*;

#[cfg_attr(any(stm32f0, stm32f1, stm32f3), path = "f013.rs")]
#[cfg_attr(any(stm32f2, stm32f4, stm32f7), path = "f247.rs")]
#[cfg_attr(stm32c0, path = "c0.rs")]
#[cfg_attr(stm32g0, path = "g0.rs")]
#[cfg_attr(stm32g4, path = "g4.rs")]
#[cfg_attr(any(stm32h5, stm32h7, stm32h7rs), path = "h.rs")]
#[cfg_attr(any(stm32l0, stm32l1, stm32l4, stm32l5, stm32wb, stm32wl, stm32u0), path = "l.rs")]
#[cfg_attr(stm32u3, path = "u3.rs")]
#[cfg_attr(stm32u5, path = "u5.rs")]
#[cfg_attr(stm32wba, path = "wba.rs")]
#[cfg_attr(stm32n6, path = "n6.rs")]
mod _version;

pub use _version::*;
use stm32_metapac::RCC;

pub use crate::_generated::{Clocks, mux};
use crate::time::Hertz;

#[cfg(feature = "low-power")]
/// Must be written within a critical section
///
/// May be read without a critical section
pub(crate) static mut REFCOUNT_STOP1: u32 = 0;

#[cfg(feature = "low-power")]
/// Must be written within a critical section
///
/// May be read without a critical section
pub(crate) static mut REFCOUNT_STOP2: u32 = 0;

#[cfg(all(feature = "low-power", not(feature = "_dual-core")))]
pub(crate) static mut RCC_CONFIG: Option<Config> = None;

#[cfg(all(feature = "low-power", feature = "_dual-core"))]
static RCC_CONFIG_PTR: core::sync::atomic::AtomicPtr<MaybeUninit<Option<Config>>> =
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
pub(crate) fn set_rcc_config_ptr(config: *mut MaybeUninit<Option<Config>>) {
    RCC_CONFIG_PTR.store(config, core::sync::atomic::Ordering::SeqCst);
}

#[cfg(not(feature = "_dual-core"))]
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
    (*core::ptr::addr_of_mut!(CLOCK_FREQS)).assume_init_ref()
}

#[cfg(feature = "_dual-core")]
/// Safety: Reads a mutable global.
pub(crate) unsafe fn get_freqs() -> &'static Clocks {
    unwrap!(CLOCK_FREQS_PTR.load(core::sync::atomic::Ordering::SeqCst).as_ref()).assume_init_ref()
}

#[cfg(all(feature = "low-power", not(feature = "_dual-core")))]
/// Safety: Reads a mutable global.
pub(crate) unsafe fn get_rcc_config() -> Option<Config> {
    RCC_CONFIG
}

#[cfg(all(feature = "low-power", feature = "_dual-core"))]
/// Safety: Reads a mutable global.
pub(crate) unsafe fn get_rcc_config() -> Option<Config> {
    unwrap!(RCC_CONFIG_PTR.load(core::sync::atomic::Ordering::SeqCst).as_ref()).assume_init()
}

#[cfg(all(feature = "low-power", not(feature = "_dual-core")))]
/// Safety: Sets a mutable global.
pub(crate) unsafe fn set_rcc_config(config: Option<Config>) {
    RCC_CONFIG = config;
}

#[cfg(all(feature = "low-power", feature = "_dual-core"))]
/// Safety: Sets a mutable global.
pub(crate) unsafe fn set_rcc_config(config: Option<Config>) {
    RCC_CONFIG_PTR
        .load(core::sync::atomic::Ordering::SeqCst)
        .write(MaybeUninit::new(config));
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
fn increment_stop_refcount(_cs: CriticalSection, stop_mode: StopMode) {
    match stop_mode {
        StopMode::Standby => {}
        StopMode::Stop2 => unsafe {
            REFCOUNT_STOP2 += 1;
        },
        StopMode::Stop1 => unsafe {
            REFCOUNT_STOP1 += 1;
        },
    }
}

#[cfg(feature = "low-power")]
fn decrement_stop_refcount(_cs: CriticalSection, stop_mode: StopMode) {
    match stop_mode {
        StopMode::Standby => {}
        StopMode::Stop2 => unsafe {
            REFCOUNT_STOP2 -= 1;
        },
        StopMode::Stop1 => unsafe {
            REFCOUNT_STOP1 -= 1;
        },
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
    /// maintain a refcount in `crate::_generated::REFCOUNTS` at this index. If the bit is not
    /// shared, this is 0xff (we don't use an `Option` to save one byte of storage).
    refcount_idx_or_0xff: u8,
    /// Stop mode of the peripheral, used to maintain `REFCOUNT_STOP1` and `REFCOUNT_STOP2`.
    #[cfg(feature = "low-power")]
    stop_mode: StopMode,
}

/// Specifies a limit for the stop mode of the peripheral or the stop mode to be entered.
/// E.g. if `StopMode::Stop1` is selected, the peripheral prevents the chip from entering Stop1 mode.
#[cfg(feature = "low-power")]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StopMode {
    #[default]
    /// Peripheral prevents chip from entering Stop1 or executor will enter Stop1
    Stop1,
    /// Peripheral prevents chip from entering Stop2 or executor will enter Stop2
    Stop2,
    /// Peripheral does not prevent chip from entering Stop
    Standby,
}

impl RccInfo {
    /// Safety:
    /// - `reset_offset_and_bit`, if set, must correspond to valid xxxRST bit
    /// - `enable_offset_and_bit` must correspond to valid xxxEN bit
    /// - `refcount_idx`, if set, must correspond to valid refcount in `_generated::REFCOUNTS`
    /// - `stop_mode` must be valid
    pub(crate) const unsafe fn new(
        reset_offset_and_bit: Option<(u8, u8)>,
        enable_offset_and_bit: (u8, u8),
        refcount_idx: Option<u8>,
        #[cfg(feature = "low-power")] stop_mode: StopMode,
    ) -> Self {
        let (reset_offset_or_0xff, reset_bit) = match reset_offset_and_bit {
            Some((offset, bit)) => (offset, bit),
            None => (0xff, 0xff),
        };
        let (enable_offset, enable_bit) = enable_offset_and_bit;
        let refcount_idx_or_0xff = match refcount_idx {
            Some(idx) => idx,
            None => 0xff,
        };
        Self {
            reset_offset_or_0xff,
            reset_bit,
            enable_offset,
            enable_bit,
            refcount_idx_or_0xff,
            #[cfg(feature = "low-power")]
            stop_mode,
        }
    }

    // TODO: should this be `unsafe`?
    pub(crate) fn enable_and_reset_with_cs(&self, _cs: CriticalSection) {
        if self.refcount_idx_or_0xff != 0xff {
            let refcount_idx = self.refcount_idx_or_0xff as usize;

            // Use .get_mut instead of []-operator so that we control how bounds checks happen.
            // Otherwise, core::fmt will be pulled in here in order to format the integer in the
            // out-of-bounds error.
            if let Some(refcount) =
                unsafe { (*core::ptr::addr_of_mut!(crate::_generated::REFCOUNTS)).get_mut(refcount_idx) }
            {
                *refcount += 1;
                if *refcount > 1 {
                    return;
                }
            } else {
                panic!("refcount_idx out of bounds: {}", refcount_idx)
            }
        }

        // set the xxxRST bit
        let reset_ptr = self.reset_ptr();
        if let Some(reset_ptr) = reset_ptr {
            #[cfg(not(stm32wl5x))]
            unsafe {
                let val = reset_ptr.read_volatile();
                reset_ptr.write_volatile(val | 1u32 << self.reset_bit);
            }

            // on stm32wl5x each CPU has its own peripheral enable bits and if the othert CPU has enabled the peripheral we don;t want to reset it
            // as that would reset the configuration that the other CPU has set up.
            // we hold a hardware lock to prevent the other CPU from enabling the peripheral while we are resetting it.
            #[cfg(stm32wl5x)]
            unsafe {
                let _lock = crate::hsem::get_hsem(3).blocking_lock(0);

                if !self.is_enabled_by_other_core() {
                    let val = reset_ptr.read_volatile();
                    reset_ptr.write_volatile(val | 1u32 << self.reset_bit);

                    trace!("rcc: reset 0x{:x}:{}", self.enable_offset, self.enable_bit);
                }
            }
        }

        // set the xxxEN bit
        let enable_ptr = self.enable_ptr();
        unsafe {
            #[cfg(not(all(stm32wl5x, feature = "_core-cm0p")))]
            {
                let val = enable_ptr.read_volatile();
                enable_ptr.write_volatile(val | 1u32 << self.enable_bit);
            }
            // second core enable for stm32wl5x is at offset 0x100
            #[cfg(all(stm32wl5x, feature = "_core-cm0p"))]
            {
                let enable_ptr = enable_ptr.add(0x100 / 4);
                let val = enable_ptr.read_volatile();
                enable_ptr.write_volatile(val | 1u32 << self.enable_bit);
            }
            trace!("rcc: enabled 0x{:x}:{}", self.enable_offset, self.enable_bit);
        }

        // we must wait two peripheral clock cycles before the clock is active
        // this seems to work, but might be incorrect
        // see http://efton.sk/STM32/gotcha/g183.html

        // dummy read (like in the ST HALs)
        let _ = unsafe { enable_ptr.read_volatile() };

        // DSB for good measure
        cortex_m::asm::dsb();

        // clear the xxxRST bit
        if let Some(reset_ptr) = reset_ptr {
            unsafe {
                let val = reset_ptr.read_volatile();
                reset_ptr.write_volatile(val & !(1u32 << self.reset_bit));
            }
        }
    }

    // TODO: should this be `unsafe`?
    pub(crate) fn disable_with_cs(&self, _cs: CriticalSection) {
        if self.refcount_idx_or_0xff != 0xff {
            let refcount_idx = self.refcount_idx_or_0xff as usize;

            // Use .get_mut instead of []-operator so that we control how bounds checks happen.
            // Otherwise, core::fmt will be pulled in here in order to format the integer in the
            // out-of-bounds error.
            if let Some(refcount) =
                unsafe { (*core::ptr::addr_of_mut!(crate::_generated::REFCOUNTS)).get_mut(refcount_idx) }
            {
                *refcount -= 1;
                if *refcount > 0 {
                    return;
                }
            } else {
                panic!("refcount_idx out of bounds: {}", refcount_idx)
            }
        }

        // clear the xxxEN bit
        let enable_ptr = self.enable_ptr();
        unsafe {
            #[cfg(not(all(stm32wl5x, feature = "_core-cm0p")))]
            {
                let val = enable_ptr.read_volatile();
                enable_ptr.write_volatile(val & !(1u32 << self.enable_bit));
            }
            // second core enable for stm32wl5x is at offset 0x100
            #[cfg(all(stm32wl5x, feature = "_core-cm0p"))]
            {
                let enable_ptr = enable_ptr.add(0x100 / 4);
                let val = enable_ptr.read_volatile();
                enable_ptr.write_volatile(val & !(1u32 << self.enable_bit));
            }
            trace!("rcc: disabled 0x{:x}:{}", self.enable_offset, self.enable_bit);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn increment_stop_refcount_with_cs(&self, _cs: CriticalSection) {
        #[cfg(feature = "low-power")]
        increment_stop_refcount(_cs, self.stop_mode);
    }

    #[allow(dead_code)]
    fn increment_minimum_stop_refcount_with_cs(&self, _cs: CriticalSection) {
        #[cfg(all(any(stm32wl, stm32wb), feature = "low-power"))]
        match self.stop_mode {
            StopMode::Stop1 | StopMode::Stop2 => increment_stop_refcount(_cs, StopMode::Stop2),
            _ => {}
        }
    }

    #[allow(dead_code)]
    pub(crate) fn increment_stop_refcount(&self) {
        #[cfg(feature = "low-power")]
        critical_section::with(|cs| self.increment_stop_refcount_with_cs(cs))
    }

    #[allow(dead_code)]
    pub(crate) fn decrement_stop_refcount_with_cs(&self, _cs: CriticalSection) {
        #[cfg(feature = "low-power")]
        decrement_stop_refcount(_cs, self.stop_mode);
    }

    #[allow(dead_code)]
    fn decrement_minimum_stop_refcount_with_cs(&self, _cs: CriticalSection) {
        #[cfg(all(any(stm32wl, stm32wb), feature = "low-power"))]
        match self.stop_mode {
            StopMode::Stop1 | StopMode::Stop2 => decrement_stop_refcount(_cs, StopMode::Stop2),
            _ => {}
        }
    }

    #[allow(dead_code)]
    pub(crate) fn decrement_stop_refcount(&self) {
        #[cfg(feature = "low-power")]
        critical_section::with(|cs| self.decrement_stop_refcount_with_cs(cs))
    }

    // TODO: should this be `unsafe`?
    pub(crate) fn enable_and_reset(&self) {
        critical_section::with(|cs| {
            self.enable_and_reset_with_cs(cs);
            self.increment_stop_refcount_with_cs(cs);
        })
    }

    #[allow(dead_code)]
    pub(crate) fn enable_and_reset_without_stop(&self) {
        critical_section::with(|cs| {
            self.enable_and_reset_with_cs(cs);
            self.increment_minimum_stop_refcount_with_cs(cs);
        })
    }

    // TODO: should this be `unsafe`?
    pub(crate) fn disable(&self) {
        critical_section::with(|cs| {
            self.disable_with_cs(cs);
            self.decrement_stop_refcount_with_cs(cs);
        })
    }

    // TODO: should this be `unsafe`?
    #[allow(dead_code)]
    pub(crate) fn disable_without_stop(&self) {
        critical_section::with(|cs| {
            self.disable_with_cs(cs);
            self.decrement_minimum_stop_refcount_with_cs(cs);
        })
    }

    #[allow(dead_code)]
    pub(crate) fn wake_guard(&self) -> WakeGuard {
        WakeGuard::new(
            #[cfg(feature = "low-power")]
            self.stop_mode,
        )
    }

    fn reset_ptr(&self) -> Option<*mut u32> {
        if self.reset_offset_or_0xff != 0xff {
            Some(unsafe { (RCC.as_ptr() as *mut u32).add(self.reset_offset_or_0xff as _) })
        } else {
            None
        }
    }

    fn enable_ptr(&self) -> *mut u32 {
        unsafe { (RCC.as_ptr() as *mut u32).add(self.enable_offset as _) }
    }

    #[cfg(stm32wl5x)]
    unsafe fn is_enabled_by_other_core(&self) -> bool {
        let ptr = self.enable_ptr();
        #[cfg(feature = "_core-cm4")]
        let ptr = ptr.add(0x100);

        (ptr.read_volatile() & (1u32 << self.enable_bit)) != 0
    }
}

pub struct WakeGuard {
    #[cfg(feature = "low-power")]
    stop_mode: StopMode,
}

impl WakeGuard {
    pub fn new(#[cfg(feature = "low-power")] stop_mode: StopMode) -> Self {
        #[cfg(feature = "low-power")]
        critical_section::with(|cs| increment_stop_refcount(cs, stop_mode));

        Self {
            #[cfg(feature = "low-power")]
            stop_mode,
        }
    }
}

impl Drop for WakeGuard {
    fn drop(&mut self) {
        #[cfg(feature = "low-power")]
        critical_section::with(|cs| decrement_stop_refcount(cs, self.stop_mode));
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
}

/// Get the kernel clock frequency of the peripheral `T`.
///
/// # Panics
///
/// Panics if the clock is not active.
pub fn frequency<T: RccPeripheral>() -> Hertz {
    T::frequency()
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

/// Disables peripheral `T`.
///
/// # Safety
///
/// Peripheral must not be in use.
// TODO: should this be `unsafe`?
pub fn disable_with_cs<T: RccPeripheral>(cs: CriticalSection) {
    T::RCC_INFO.disable_with_cs(cs);
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

/// Enables and resets peripheral `T` without incrementing the stop refcount.
///
/// # Safety
///
/// Peripheral must not be in use.
// TODO: should this be `unsafe`?
pub fn enable_and_reset_without_stop<T: RccPeripheral>() {
    T::RCC_INFO.enable_and_reset_without_stop();
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
    critical_section::with(|cs| init_rcc(cs, config))
}

pub(crate) fn init_rcc(_cs: CriticalSection, config: Config) {
    unsafe {
        // if we are using a lp timer as the time driver, we need to make sure that the clock selection is sane
        // TODO: how to share this config from the secondary core thats using a lp timer? (right now its not detected as missing!)
        #[cfg(feature = "_lp-time-driver")]
        let config = {
            use crate::pac::rcc::vals::Lptimsel;
            let mut config = config;
            #[cfg(time_driver_lptim1)]
            {
                match config.mux.lptim1sel {
                    Lptimsel::PCLK1 => {
                        // set it to LSI
                        config.mux.lptim1sel = Lptimsel::LSI;
                        config.ls.lsi = true;
                    }
                    Lptimsel::LSI => {
                        // ok but insure the lsi is enabled
                        config.ls.lsi = true;
                    }
                    Lptimsel::HSI => {
                        // ok but insure the hsi is enabled
                        config.hsi = true;
                    }
                    Lptimsel::LSE => {
                        // ok but insure the lse is configured with peripherals_clocked = true!!!
                        if let Some(mut lse_config) = config.ls.lse {
                            lse_config.peripherals_clocked = true;
                            config.ls.lse = Some(lse_config);
                        } else {
                            panic!("LSE is not not configured, but selected for time_driver!!!");
                        }
                    }
                }
            }
            #[cfg(time_driver_lptim2)]
            {
                match config.mux.lptim2sel {
                    Lptimsel::PCLK1 => {
                        // set it to LSI
                        config.mux.lptim2sel = Lptimsel::LSI;
                        config.ls.lsi = true;
                    }
                    Lptimsel::LSI => {
                        // ok but insure the lsi is enabled
                        config.ls.lsi = true;
                    }
                    Lptimsel::HSI => {
                        // ok but insure the hsi is enabled
                        config.hsi = true;
                    }
                    Lptimsel::LSE => {
                        // ok but insure the lse is configured with peripherals_clocked = true!!!
                        if let Some(mut lse_config) = config.ls.lse {
                            lse_config.peripherals_clocked = true;
                            config.ls.lse = Some(lse_config);
                        } else {
                            panic!("LSE is not not configured, but selected for time_driver!!!");
                        }
                    }
                }
            }
            #[cfg(time_driver_lptim3)]
            {
                match config.mux.lptim3sel {
                    Lptimsel::PCLK1 => {
                        // set it to LSI
                        config.mux.lptim3sel = Lptimsel::LSI;
                        config.ls.lsi = true;
                    }
                    Lptimsel::LSI => {
                        // ok but insure the lsi is enabled
                        config.ls.lsi = true;
                    }
                    Lptimsel::HSI => {
                        // ok but insure the hsi is enabled
                        config.hsi = true;
                    }
                    Lptimsel::LSE => {
                        // ok but insure the lse is configured with peripherals_clocked = true!!!
                        if let Some(mut lse_config) = config.ls.lse {
                            lse_config.peripherals_clocked = true;
                            config.ls.lse = Some(lse_config);
                        } else {
                            panic!("LSE is not not configured, but selected for time_driver!!!");
                        }
                    }
                }
            }
            config
        };

        init(config);

        // must be after rcc init
        #[cfg(feature = "_time-driver")]
        crate::time_driver::init(_cs);

        #[cfg(feature = "low-power")]
        {
            set_rcc_config(Some(config));
            #[cfg(not(feature = "_lp-time-driver"))]
            {
                REFCOUNT_STOP2 = 0;
                REFCOUNT_STOP1 = 0;
            }
        }
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
