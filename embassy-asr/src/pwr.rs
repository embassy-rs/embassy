//! ASR6601 power-control driver.
//!
//! The implementation follows the ASR SDK `tremo_pwr.c` sequences. The
//! crate-level [`crate::Config`] does not currently contain power settings, so
//! runtime power transitions are made through an owned [`Pwr`] constructed from
//! [`peripherals::PWR`].
//!
//! The vendor SVD does not describe the fields in the PWR status registers.
//! Consequently this module exposes those registers only as [`RawStatus`].

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_hal_internal::interrupt::InterruptExt;

use crate::pac::{self, Interrupt};
use crate::{Peri, interrupt, peripherals};

const CR0_LOW_POWER_MODE_MASK: u32 = 0x03;
const CR0_WAIT_FOR_EVENT: u32 = 1 << 5;

const CR1_DEEP_SLEEP_ENABLE: u32 = 1 << 4;
const CR1_TRIM_MASK: u32 = 0x0f << 20;
const CR1_TRIM_DEFAULT: u32 = 1 << 20;
const CR1_STOP3_NOT_STANDBY: u32 = 1 << 24;

const FLASH_INFO_TRIM: *const u32 = 0x1000_2010 as *const u32;
const FLASH_PROTECT_SEQUENCE_0: u32 = 0x8c9d_aebf;
const FLASH_PROTECT_SEQUENCE_1: u32 = 0x1314_1516;

const AFEC_BASE: usize = 0x4000_8000;
const AFEC_XO32K_CONTROL: usize = 0x03;
const AFEC_LOW_POWER_CONTROL_0: usize = 0x05;
const AFEC_LOW_POWER_CONTROL_1: usize = 0x06;
const AFEC_STOP3_CONTROL: usize = 0x0c;

const XO32K_LOW_POWER_ENABLE: u32 = 1 << 7;
const LOW_POWER_RUN_ENABLE_0: u32 = 1 << 3;
const LOW_POWER_RUN_ENABLE_1: u32 = 0x03 << 20;
const STOP3_ANALOG_ENABLE: u32 = 1 << 14;

static PWR_INTERRUPT_OBSERVED: AtomicBool = AtomicBool::new(false);

/// Deep-sleep mode selected in the PWR controller.
///
/// The SDK names the encodings but does not document which clocks, memories, or
/// peripheral domains remain available in `Stop0` through `Stop2`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum DeepSleepMode {
    /// SDK `PWR_LP_MODE_STOP0`.
    Stop0 = 0,
    /// SDK `PWR_LP_MODE_STOP1`.
    Stop1 = 1,
    /// SDK `PWR_LP_MODE_STOP2`.
    Stop2 = 2,
    /// Retention stop mode used by the SDK GPIO, RTC, LPUART, and LPTIMER examples.
    Stop3 = 3,
    /// Standby mode. A successful wake from standby resets the processor.
    Standby = 4,
}

impl DeepSleepMode {
    fn needs_stop3_sequence(self) -> bool {
        matches!(self, Self::Stop3 | Self::Standby)
    }
}

/// Uninterpreted contents of the PWR status registers.
///
/// Neither the vendor SVD nor `tremo_pwr.h` defines fields or clear semantics
/// for these registers. In particular, do not assume that writing a captured
/// value back is a valid way to clear status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawStatus {
    sr0: u32,
    sr1: u32,
}

impl RawStatus {
    /// Raw PWR `SR0` value.
    pub const fn sr0(self) -> u32 {
        self.sr0
    }

    /// Raw PWR `SR1` value.
    pub const fn sr1(self) -> u32 {
        self.sr1
    }
}

/// Status captured after a WFI or WFE returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WakeStatus {
    raw: RawStatus,
    pwr_interrupt_observed: bool,
    pwr_interrupt_pending: bool,
    standby_reset: bool,
}

impl WakeStatus {
    /// Raw PWR status-register snapshot.
    pub const fn raw(self) -> RawStatus {
        self.raw
    }

    /// Whether the bound PWR interrupt handler ran during the latest wait.
    ///
    /// This is a software latch, not a decoded PWR hardware status bit.
    pub const fn pwr_interrupt_observed(self) -> bool {
        self.pwr_interrupt_observed
    }

    /// Whether the PWR interrupt is pending in the NVIC.
    ///
    /// This does not identify the underlying wake source.
    pub const fn pwr_interrupt_pending(self) -> bool {
        self.pwr_interrupt_pending
    }

    /// Whether RCC reports that this boot followed a standby wake/reset.
    pub const fn standby_reset(self) -> bool {
        self.standby_reset
    }
}

/// Interrupt handler for the PWR interrupt.
///
/// The SDK handler is empty and the SVD contains no interrupt-status fields.
/// This handler therefore records only that the vector ran.
pub struct InterruptHandler;

impl interrupt::typelevel::Handler<interrupt::typelevel::PWR> for InterruptHandler {
    unsafe fn on_interrupt() {
        PWR_INTERRUPT_OBSERVED.store(true, Ordering::Release);
    }
}

/// Owned ASR6601 power controller.
pub struct Pwr<'d> {
    _peri: Peri<'d, peripherals::PWR>,
}

impl<'d> Pwr<'d> {
    /// Create the power driver and enable the PWR register clock.
    ///
    /// `irq` proves that the PWR vector is bound before the WFI APIs temporarily
    /// enable it, matching `pwr_sleep_wfi()` and `pwr_deepsleep_wfi()`.
    pub fn new(
        peri: Peri<'d, peripherals::PWR>,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::PWR, InterruptHandler> + 'd,
    ) -> Self {
        critical_section::with(|_| {
            let rcc = unsafe { pac::Rcc::steal() };
            rcc.cgr0().modify(|_, w| w.pwr_clk_en().set_bit());
        });

        Interrupt::PWR.disable();
        Interrupt::PWR.unpend();
        PWR_INTERRUPT_OBSERVED.store(false, Ordering::Release);

        Self { _peri: peri }
    }

    /// Enter ordinary sleep and wait for an interrupt.
    ///
    /// The PWR interrupt is enabled only for the duration of the wait, as in the
    /// SDK. Other enabled interrupts can also wake the processor.
    pub fn sleep_wfi(&mut self) -> WakeStatus {
        clear_sleepdeep();
        self.wait_wfi();
        self.wake_status()
    }

    /// Enter ordinary sleep and wait for an event.
    ///
    /// This performs one WFE, matching `pwr_sleep_wfe()`. A previously latched
    /// event can therefore make it return immediately.
    pub fn sleep_wfe(&mut self) -> WakeStatus {
        clear_sleepdeep();
        PWR_INTERRUPT_OBSERVED.store(false, Ordering::Release);
        cortex_m::asm::dsb();
        cortex_m::asm::wfe();
        self.wake_status()
    }

    /// Enter a deep-sleep mode and wait for an interrupt.
    ///
    /// `SLEEPDEEP` is cleared before this function returns so a later Embassy
    /// executor WFE does not unexpectedly re-enter deep sleep.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// - every active clock and peripheral can tolerate the selected mode, and
    ///   all transfers or flash operations that cannot tolerate it have ended;
    /// - a valid wake source and its interrupt are configured;
    /// - the clock tree and affected peripherals are restored as required after
    ///   wake;
    /// - for `Stop3`/`Standby`, disabling flash prefetch and setting AFEC analog
    ///   register 0x0c bit 14 is valid for this silicon and application;
    /// - code after a `Standby` request does not rely on the request returning.
    pub unsafe fn deep_sleep_wfi(&mut self, mode: DeepSleepMode) -> WakeStatus {
        unsafe { self.prepare_deep_sleep(mode, WaitKind::Interrupt) };
        let _sleepdeep = SleepDeepGuard::enter();
        self.wait_wfi();
        self.wake_status()
    }

    /// Enter a deep-sleep mode and wait for an event.
    ///
    /// This uses the SDK's `SEV; WFE; WFE` sequence to discard a stale event
    /// before sleeping. `SLEEPDEEP` is cleared before this function returns.
    ///
    /// # Safety
    ///
    /// The caller must satisfy the clock, peripheral, flash, AFEC, wake-source,
    /// and standby requirements of [`Self::deep_sleep_wfi`]. The configured
    /// wake source must also generate an event that can wake WFE.
    pub unsafe fn deep_sleep_wfe(&mut self, mode: DeepSleepMode) -> WakeStatus {
        unsafe { self.prepare_deep_sleep(mode, WaitKind::Event) };
        let _sleepdeep = SleepDeepGuard::enter();

        PWR_INTERRUPT_OBSERVED.store(false, Ordering::Release);
        cortex_m::asm::dsb();
        cortex_m::asm::sev();
        cortex_m::asm::wfe();
        cortex_m::asm::wfe();

        self.wake_status()
    }

    /// Enter low-power run mode until the returned guard is dropped.
    ///
    /// # Safety
    ///
    /// The caller must first put the CPU and bus clocks within the undocumented
    /// low-power-run limits for this silicon and ensure all active peripherals
    /// can operate with the reduced regulator settings. The SDK provides no
    /// frequency or peripheral-domain limits, so this cannot be checked here.
    pub unsafe fn enter_low_power_run(&mut self) -> LowPowerRun<'_, 'd> {
        set_low_power_run(true);
        LowPowerRun {
            pwr: self,
            active: true,
        }
    }

    /// Sleep with WFI while low-power run mode is active.
    ///
    /// Low-power run mode is exited before this function returns.
    ///
    /// # Safety
    ///
    /// The caller must satisfy [`Self::enter_low_power_run`]'s clock and
    /// peripheral requirements.
    pub unsafe fn low_power_sleep_wfi(&mut self) -> WakeStatus {
        let mut low_power = unsafe { self.enter_low_power_run() };
        let status = low_power.sleep_wfi();
        drop(low_power);
        status
    }

    /// Sleep with WFE while low-power run mode is active.
    ///
    /// Low-power run mode is exited before this function returns.
    ///
    /// # Safety
    ///
    /// The caller must satisfy [`Self::enter_low_power_run`]'s clock and
    /// peripheral requirements.
    pub unsafe fn low_power_sleep_wfe(&mut self) -> WakeStatus {
        let mut low_power = unsafe { self.enter_low_power_run() };
        let status = low_power.sleep_wfe();
        drop(low_power);
        status
    }

    /// Whether both SDK low-power-run control fields are set.
    pub fn low_power_run_enabled(&self) -> bool {
        read_analog(AFEC_LOW_POWER_CONTROL_0) & LOW_POWER_RUN_ENABLE_0 != 0
            && read_analog(AFEC_LOW_POWER_CONTROL_1) & LOW_POWER_RUN_ENABLE_1 == LOW_POWER_RUN_ENABLE_1
    }

    /// Enable XO32K low-power mode.
    ///
    /// # Safety
    ///
    /// XO32K must already be running and stable. The board's crystal and PCB
    /// routing must be qualified for ASR6601 XO32K low-power mode, and no code
    /// may concurrently reconfigure XO32K. The datasheet warns that unsuitable
    /// routing requires normal mode.
    pub unsafe fn enable_xo32k_low_power_mode(&mut self) {
        modify_analog(AFEC_XO32K_CONTROL, |value| value | XO32K_LOW_POWER_ENABLE);
    }

    /// Disable XO32K low-power mode, selecting its normal operating mode.
    pub fn disable_xo32k_low_power_mode(&mut self) {
        modify_analog(AFEC_XO32K_CONTROL, |value| value & !XO32K_LOW_POWER_ENABLE);
    }

    /// Whether the XO32K low-power-mode control bit is set.
    ///
    /// This reports the requested mode, not oscillator stability.
    pub fn xo32k_low_power_mode_enabled(&self) -> bool {
        read_analog(AFEC_XO32K_CONTROL) & XO32K_LOW_POWER_ENABLE != 0
    }

    /// Capture the two raw PWR status registers.
    pub fn raw_status(&self) -> RawStatus {
        let pwr = regs();
        RawStatus {
            sr0: pwr.sr0().read().bits(),
            sr1: pwr.sr1().read().bits(),
        }
    }

    /// Whether the PWR vector has run since the latest wait began or latch clear.
    pub fn pwr_interrupt_observed(&self) -> bool {
        PWR_INTERRUPT_OBSERVED.load(Ordering::Acquire)
    }

    /// Return and clear the software PWR-interrupt latch.
    pub fn take_pwr_interrupt_observed(&mut self) -> bool {
        PWR_INTERRUPT_OBSERVED.swap(false, Ordering::AcqRel)
    }

    /// Whether the PWR interrupt is pending in the NVIC.
    pub fn pwr_interrupt_pending(&self) -> bool {
        Interrupt::PWR.is_pending()
    }

    /// Clear the NVIC pending bit for PWR.
    ///
    /// This does not clear an undocumented peripheral status source. A
    /// level-sensitive source can pend the interrupt again immediately.
    pub fn clear_pwr_interrupt_pending(&mut self) {
        Interrupt::PWR.unpend();
    }

    /// Whether RCC reports that the current boot followed a standby wake/reset.
    pub fn was_standby_reset(&self) -> bool {
        let rcc = unsafe { pac::Rcc::steal() };
        rcc.rst_sr().read().standby_reset_sr().bit_is_set()
    }

    fn wait_wfi(&mut self) {
        PWR_INTERRUPT_OBSERVED.store(false, Ordering::Release);
        Interrupt::PWR.unpend();
        unsafe { Interrupt::PWR.enable() };

        cortex_m::asm::dsb();
        cortex_m::asm::wfi();

        Interrupt::PWR.disable();
    }

    fn wake_status(&self) -> WakeStatus {
        WakeStatus {
            raw: self.raw_status(),
            pwr_interrupt_observed: self.pwr_interrupt_observed(),
            pwr_interrupt_pending: self.pwr_interrupt_pending(),
            standby_reset: self.was_standby_reset(),
        }
    }

    unsafe fn prepare_deep_sleep(&mut self, mode: DeepSleepMode, wait: WaitKind) {
        let pwr = regs();

        pwr.cr1().modify(|r, w| unsafe {
            let mut bits = r.bits() | CR1_DEEP_SLEEP_ENABLE;
            if unsafe { core::ptr::read_volatile(FLASH_INFO_TRIM) } & 0x03 == 0 {
                bits = (bits & !CR1_TRIM_MASK) | CR1_TRIM_DEFAULT;
            }
            w.bits(bits)
        });

        pwr.cr0().modify(|r, w| unsafe {
            let bits = match wait {
                WaitKind::Interrupt => r.bits() & !CR0_WAIT_FOR_EVENT,
                WaitKind::Event => r.bits() | CR0_WAIT_FOR_EVENT,
            };
            w.bits(bits)
        });

        if mode.needs_stop3_sequence() {
            disable_flash_prefetch();
            modify_analog(AFEC_STOP3_CONTROL, |value| value | STOP3_ANALOG_ENABLE);

            pwr.cr0()
                .modify(|r, w| unsafe { w.bits((r.bits() & !CR0_LOW_POWER_MODE_MASK) | DeepSleepMode::Stop3 as u32) });
            pwr.cr1().modify(|r, w| unsafe {
                let bits = match mode {
                    DeepSleepMode::Stop3 => r.bits() | CR1_STOP3_NOT_STANDBY,
                    DeepSleepMode::Standby => r.bits() & !CR1_STOP3_NOT_STANDBY,
                    _ => r.bits(),
                };
                w.bits(bits)
            });
        } else {
            pwr.cr0()
                .modify(|r, w| unsafe { w.bits((r.bits() & !CR0_LOW_POWER_MODE_MASK) | mode as u32) });
        }
    }
}

impl Drop for Pwr<'_> {
    fn drop(&mut self) {
        Interrupt::PWR.disable();
    }
}

/// Guard proving that low-power run mode is active.
///
/// Dropping the guard restores normal run mode in the reverse order used by the
/// SDK.
pub struct LowPowerRun<'a, 'd> {
    pwr: &'a mut Pwr<'d>,
    active: bool,
}

impl LowPowerRun<'_, '_> {
    /// Enter ordinary sleep and wait for an interrupt.
    pub fn sleep_wfi(&mut self) -> WakeStatus {
        self.pwr.sleep_wfi()
    }

    /// Enter ordinary sleep and wait for an event.
    pub fn sleep_wfe(&mut self) -> WakeStatus {
        self.pwr.sleep_wfe()
    }

    /// Capture status without leaving low-power run mode.
    pub fn status(&self) -> WakeStatus {
        self.pwr.wake_status()
    }

    /// Exit low-power run mode.
    pub fn exit(self) {}
}

impl Drop for LowPowerRun<'_, '_> {
    fn drop(&mut self) {
        if self.active {
            set_low_power_run(false);
            self.active = false;
        }
    }
}

#[derive(Clone, Copy)]
enum WaitKind {
    Interrupt,
    Event,
}

struct SleepDeepGuard;

impl SleepDeepGuard {
    fn enter() -> Self {
        let mut cp = unsafe { cortex_m::Peripherals::steal() };
        cp.SCB.set_sleepdeep();
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
        Self
    }
}

impl Drop for SleepDeepGuard {
    fn drop(&mut self) {
        clear_sleepdeep();
    }
}

fn clear_sleepdeep() {
    let mut cp = unsafe { cortex_m::Peripherals::steal() };
    cp.SCB.clear_sleepdeep();
    cortex_m::asm::dsb();
    cortex_m::asm::isb();
}

fn regs() -> pac::Pwr {
    unsafe { pac::Pwr::steal() }
}

fn read_analog(index: usize) -> u32 {
    unsafe { core::ptr::read_volatile((AFEC_BASE + index * 4) as *const u32) }
}

fn modify_analog(index: usize, f: impl FnOnce(u32) -> u32) {
    critical_section::with(|_| unsafe {
        let register = (AFEC_BASE + index * 4) as *mut u32;
        let value = core::ptr::read_volatile(register);
        core::ptr::write_volatile(register, f(value));
    });
}

fn set_low_power_run(enabled: bool) {
    if enabled {
        modify_analog(AFEC_LOW_POWER_CONTROL_0, |value| value | LOW_POWER_RUN_ENABLE_0);
        modify_analog(AFEC_LOW_POWER_CONTROL_1, |value| value | LOW_POWER_RUN_ENABLE_1);
    } else {
        modify_analog(AFEC_LOW_POWER_CONTROL_1, |value| value & !LOW_POWER_RUN_ENABLE_1);
        modify_analog(AFEC_LOW_POWER_CONTROL_0, |value| value & !LOW_POWER_RUN_ENABLE_0);
    }
}

fn disable_flash_prefetch() {
    critical_section::with(|_| {
        let efc = unsafe { pac::Efc::steal() };
        if efc.cr().read().prefetch_en().bit_is_clear() {
            return;
        }

        unsafe {
            efc.protect_seq().write_with_zero(|w| w.bits(FLASH_PROTECT_SEQUENCE_0));
            efc.protect_seq().write_with_zero(|w| w.bits(FLASH_PROTECT_SEQUENCE_1));
        }
        efc.cr().modify(|_, w| w.prefetch_en().clear_bit());
        unsafe {
            efc.protect_seq().write_with_zero(|w| w.bits(FLASH_PROTECT_SEQUENCE_0));
            efc.protect_seq().write_with_zero(|w| w.bits(0));
        }
    });
}
