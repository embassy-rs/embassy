//! # OSTIMER Driver with Robustness Features
//!
//! This module provides an async timer driver for the NXP MCXA276 OSTIMER peripheral
//! with protection against race conditions and timer rollover issues.
//!
//! ## Features
//!
//! - Async timing with embassy-time integration
//! - Gray code counter handling (42-bit counter)
//! - Interrupt-driven wakeups
//! - Configurable interrupt priority
//! - **Race condition protection**: Critical sections and atomic operations
//! - **Timer rollover handling**: Bounds checking and rollover prevention
//!
//! ## Clock Frequency Configuration
//!
//! The OSTIMER frequency depends on your system's clock configuration. You must provide
//! the actual frequency when calling `time_driver::init()`.
//!
//! ## Race Condition Protection
//! - Critical sections in interrupt handlers prevent concurrent access
//! - Atomic register operations with memory barriers
//! - Proper interrupt flag clearing and validation
//!
//! ## Timer Rollover Handling
//! - Bounds checking prevents scheduling beyond timer capacity
//! - Immediate wake for timestamps that would cause rollover issues
#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_hal_internal::{Peri, PeripheralType};

use crate::clocks::periph_helpers::{OsTimerConfig, OstimerClockSel};
use crate::clocks::{Gate, PoweredClock, assert_reset, enable_and_reset, is_reset_released, release_reset};
use crate::interrupt::InterruptExt;
use crate::pac;

// PAC defines the shared RegisterBlock under `ostimer0`.
type Regs = pac::ostimer0::RegisterBlock;

// OSTIMER EVTIMER register layout constants
/// Total width of the EVTIMER counter in bits (42 bits total)
const EVTIMER_TOTAL_BITS: u32 = 42;
/// Width of the low part of EVTIMER (bits 31:0)
const EVTIMER_LO_BITS: u32 = 32;
/// Width of the high part of EVTIMER (bits 41:32)
const EVTIMER_HI_BITS: u32 = 10;
/// Bit position where high part starts in the combined 64-bit value
const EVTIMER_HI_SHIFT: u32 = 32;

/// Bit mask for the high part of EVTIMER
const EVTIMER_HI_MASK: u16 = (1 << EVTIMER_HI_BITS) - 1;

/// Maximum value for MATCH_L register (32-bit)
const MATCH_L_MAX: u32 = u32::MAX;
/// Maximum value for MATCH_H register (10-bit)
const MATCH_H_MAX: u16 = EVTIMER_HI_MASK;

/// Bit mask for extracting the low 32 bits from a 64-bit value
const LOW_32_BIT_MASK: u64 = u32::MAX as u64;

/// Gray code conversion bit shifts (most significant to least)
const GRAY_CONVERSION_SHIFTS: [u32; 6] = [32, 16, 8, 4, 2, 1];

/// Maximum timer value before rollover (2^42 - 1 ticks)
/// Actual rollover time depends on the configured clock frequency
const TIMER_MAX_VALUE: u64 = (1u64 << EVTIMER_TOTAL_BITS) - 1;

/// Threshold for detecting timer rollover in comparisons (1 second at 1MHz)
const TIMER_ROLLOVER_THRESHOLD: u64 = 1_000_000;

/// Common default interrupt priority for OSTIMER
const DEFAULT_INTERRUPT_PRIORITY: u8 = 3;

// Global alarm state for interrupt handling
static ALARM_ACTIVE: AtomicBool = AtomicBool::new(false);
static mut ALARM_CALLBACK: Option<fn()> = None;
static mut ALARM_FLAG: Option<*const AtomicBool> = None;
static mut ALARM_TARGET_TIME: u64 = 0;

/// Number of tight spin iterations between elapsed time checks while waiting for MATCH writes to return to the idle (0) state.
const MATCH_WRITE_READY_SPINS: usize = 512;
/// Maximum time (in OSTIMER ticks) to wait for MATCH registers to become writable (~5 ms at 1 MHz).
const MATCH_WRITE_READY_TIMEOUT_TICKS: u64 = 5_000;
/// Short stabilization delay executed after toggling the MRCC reset line to let the OSTIMER bus interface settle.
const RESET_STABILIZE_SPINS: usize = 512;

pub(super) fn wait_for_match_write_ready(r: &Regs) -> bool {
    let start = now_ticks_read();
    let mut spin_budget = 0usize;

    loop {
        if r.osevent_ctrl().read().match_wr_rdy().bit_is_clear() {
            return true;
        }

        cortex_m::asm::nop();
        spin_budget += 1;

        if spin_budget >= MATCH_WRITE_READY_SPINS {
            spin_budget = 0;

            let elapsed = now_ticks_read().wrapping_sub(start);
            if elapsed >= MATCH_WRITE_READY_TIMEOUT_TICKS {
                return false;
            }
        }
    }
}

pub(super) fn wait_for_match_write_complete(r: &Regs) -> bool {
    let start = now_ticks_read();
    let mut spin_budget = 0usize;

    loop {
        if r.osevent_ctrl().read().match_wr_rdy().bit_is_clear() {
            return true;
        }

        cortex_m::asm::nop();
        spin_budget += 1;

        if spin_budget >= MATCH_WRITE_READY_SPINS {
            spin_budget = 0;

            let elapsed = now_ticks_read().wrapping_sub(start);
            if elapsed >= MATCH_WRITE_READY_TIMEOUT_TICKS {
                return false;
            }
        }
    }
}

fn prime_match_registers(r: &Regs) {
    // Disable the interrupt, clear any pending flag, then wait until the MATCH registers are writable.
    r.osevent_ctrl()
        .write(|w| w.ostimer_intrflag().clear_bit_by_one().ostimer_intena().clear_bit());

    if wait_for_match_write_ready(r) {
        r.match_l().write(|w| unsafe { w.match_value().bits(MATCH_L_MAX) });
        r.match_h().write(|w| unsafe { w.match_value().bits(MATCH_H_MAX) });
        let _ = wait_for_match_write_complete(r);
    }
}

/// Single-shot alarm functionality for OSTIMER
pub struct Alarm<'d> {
    /// Whether the alarm is currently active
    active: AtomicBool,
    /// Callback to execute when alarm expires (optional)
    callback: Option<fn()>,
    /// Flag that gets set when alarm expires (optional)
    flag: Option<&'d AtomicBool>,
    _phantom: core::marker::PhantomData<&'d mut ()>,
}

impl<'d> Default for Alarm<'d> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'d> Alarm<'d> {
    /// Create a new alarm instance
    pub fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            callback: None,
            flag: None,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Set a callback that will be executed when the alarm expires
    /// Note: Due to interrupt handler constraints, callbacks must be static function pointers
    pub fn with_callback(mut self, callback: fn()) -> Self {
        self.callback = Some(callback);
        self
    }

    /// Set a flag that will be set to true when the alarm expires
    pub fn with_flag(mut self, flag: &'d AtomicBool) -> Self {
        self.flag = Some(flag);
        self
    }

    /// Check if the alarm is currently active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Cancel the alarm if it's active
    pub fn cancel(&self) {
        self.active.store(false, Ordering::Release);
    }
}

/// Configuration for Ostimer::new()
#[derive(Copy, Clone)]
pub struct Config {
    /// Initialize MATCH registers to their max values and mask/clear the interrupt flag.
    pub init_match_max: bool,
    pub power: PoweredClock,
    pub source: OstimerClockSel,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            init_match_max: true,
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: OstimerClockSel::Clk1M,
        }
    }
}

/// OSTIMER peripheral instance
pub struct Ostimer<'d, I: Instance> {
    _inst: core::marker::PhantomData<I>,
    clock_frequency_hz: u64,
    _phantom: core::marker::PhantomData<&'d mut ()>,
}

impl<'d, I: Instance> Ostimer<'d, I> {
    /// Construct OSTIMER handle.
    /// Requires clocks for the instance to be enabled by the board before calling.
    /// Does not enable NVIC or INTENA; use time_driver::init() for async operation.
    pub fn new(_inst: Peri<'d, I>, cfg: Config) -> Self {
        let clock_freq = unsafe {
            enable_and_reset::<I>(&OsTimerConfig {
                power: cfg.power,
                source: cfg.source,
            })
            .expect("Enabling OsTimer clock should not fail")
        };

        assert!(clock_freq > 0, "OSTIMER frequency must be greater than 0");

        if cfg.init_match_max {
            let r: &Regs = unsafe { &*I::ptr() };
            // Mask INTENA, clear pending flag, and set MATCH to max so no spurious IRQ fires.
            prime_match_registers(r);
        }

        Self {
            _inst: core::marker::PhantomData,
            clock_frequency_hz: clock_freq as u64,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Get the configured clock frequency in Hz
    pub fn clock_frequency_hz(&self) -> u64 {
        self.clock_frequency_hz
    }

    /// Read the current timer counter value in timer ticks
    ///
    /// # Returns
    /// Current timer counter value as a 64-bit unsigned integer
    pub fn now(&self) -> u64 {
        now_ticks_read()
    }

    /// Reset the timer counter to zero
    ///
    /// This performs a hardware reset of the OSTIMER peripheral, which will reset
    /// the counter to zero and clear any pending interrupts. Note that this will
    /// affect all timer operations including embassy-time.
    ///
    /// # Safety
    /// This operation will reset the entire OSTIMER peripheral. Any active alarms
    /// or time_driver operations will be disrupted. Use with caution.
    pub fn reset(&self, _peripherals: &crate::pac::Peripherals) {
        critical_section::with(|_| {
            let r: &Regs = unsafe { &*I::ptr() };

            // Mask the peripheral interrupt flag before we toggle the reset line so that
            // no new NVIC activity races with the reset sequence.
            r.osevent_ctrl()
                .write(|w| w.ostimer_intrflag().clear_bit_by_one().ostimer_intena().clear_bit());

            unsafe {
                assert_reset::<I>();

                for _ in 0..RESET_STABILIZE_SPINS {
                    cortex_m::asm::nop();
                }

                release_reset::<I>();

                while !is_reset_released::<I>() {
                    cortex_m::asm::nop();
                }
            }

            for _ in 0..RESET_STABILIZE_SPINS {
                cortex_m::asm::nop();
            }

            // Clear alarm bookkeeping before re-arming MATCH registers.
            ALARM_ACTIVE.store(false, Ordering::Release);
            unsafe {
                ALARM_TARGET_TIME = 0;
                ALARM_CALLBACK = None;
                ALARM_FLAG = None;
            }

            prime_match_registers(r);
        });

        // Ensure no stale OS_EVENT request remains pending after the reset sequence.
        crate::interrupt::OS_EVENT.unpend();
    }

    /// Schedule a single-shot alarm to expire after the specified delay in microseconds
    ///
    /// # Parameters
    /// * `alarm` - The alarm instance to schedule
    /// * `delay_us` - Delay in microseconds from now
    ///
    /// # Returns
    /// `true` if the alarm was scheduled successfully, `false` if it would exceed timer capacity
    pub fn schedule_alarm_delay(&self, alarm: &Alarm, delay_us: u64) -> bool {
        let delay_ticks = (delay_us * self.clock_frequency_hz) / 1_000_000;
        let target_time = now_ticks_read() + delay_ticks;
        self.schedule_alarm_at(alarm, target_time)
    }

    /// Schedule a single-shot alarm to expire at the specified absolute time in timer ticks
    ///
    /// # Parameters
    /// * `alarm` - The alarm instance to schedule
    /// * `target_ticks` - Absolute time in timer ticks when the alarm should expire
    ///
    /// # Returns
    /// `true` if the alarm was scheduled successfully, `false` if it would exceed timer capacity
    pub fn schedule_alarm_at(&self, alarm: &Alarm, target_ticks: u64) -> bool {
        let now = now_ticks_read();

        // Check if target time is in the past
        if target_ticks <= now {
            // Execute callback immediately if alarm was supposed to be active
            if alarm.active.load(Ordering::Acquire) {
                alarm.active.store(false, Ordering::Release);
                if let Some(callback) = alarm.callback {
                    callback();
                }
                if let Some(flag) = &alarm.flag {
                    flag.store(true, Ordering::Release);
                }
            }
            return true;
        }

        // Check for timer rollover
        let max_future = now + TIMER_MAX_VALUE;
        if target_ticks > max_future {
            return false; // Would exceed timer capacity
        }

        // Program the timer
        let r: &Regs = unsafe { &*I::ptr() };

        critical_section::with(|_| {
            // Disable interrupt and clear flag
            r.osevent_ctrl()
                .write(|w| w.ostimer_intrflag().clear_bit_by_one().ostimer_intena().clear_bit());

            if !wait_for_match_write_ready(r) {
                prime_match_registers(r);

                if !wait_for_match_write_ready(r) {
                    alarm.active.store(false, Ordering::Release);
                    ALARM_ACTIVE.store(false, Ordering::Release);
                    unsafe {
                        ALARM_TARGET_TIME = 0;
                        ALARM_CALLBACK = None;
                        ALARM_FLAG = None;
                    }
                    return false;
                }
            }

            // Mark alarm as active now that we know the MATCH registers are writable
            alarm.active.store(true, Ordering::Release);

            // Set global alarm state for interrupt handler
            ALARM_ACTIVE.store(true, Ordering::Release);
            unsafe {
                ALARM_TARGET_TIME = target_ticks;
                ALARM_CALLBACK = alarm.callback;
                ALARM_FLAG = alarm.flag.map(|f| f as *const AtomicBool);
            }

            // Program MATCH registers (Gray-coded)
            let gray = bin_to_gray(target_ticks);
            let l = (gray & LOW_32_BIT_MASK) as u32;
            let h = (((gray >> EVTIMER_HI_SHIFT) as u16) & EVTIMER_HI_MASK) as u16;

            r.match_l().write(|w| unsafe { w.match_value().bits(l) });
            r.match_h().write(|w| unsafe { w.match_value().bits(h) });

            if !wait_for_match_write_complete(r) {
                alarm.active.store(false, Ordering::Release);
                ALARM_ACTIVE.store(false, Ordering::Release);
                unsafe {
                    ALARM_TARGET_TIME = 0;
                    ALARM_CALLBACK = None;
                    ALARM_FLAG = None;
                }
                return false;
            }

            let now_after_program = now_ticks_read();
            let intrflag_set = r.osevent_ctrl().read().ostimer_intrflag().bit_is_set();
            if now_after_program >= target_ticks && !intrflag_set {
                alarm.active.store(false, Ordering::Release);
                ALARM_ACTIVE.store(false, Ordering::Release);
                unsafe {
                    ALARM_TARGET_TIME = 0;
                    ALARM_CALLBACK = None;
                    ALARM_FLAG = None;
                }
                return false;
            }

            // Enable interrupt
            r.osevent_ctrl().write(|w| w.ostimer_intena().set_bit());

            true
        })
    }

    /// Cancel any active alarm
    pub fn cancel_alarm(&self, alarm: &Alarm) {
        critical_section::with(|_| {
            alarm.cancel();

            // Clear global alarm state
            ALARM_ACTIVE.store(false, Ordering::Release);
            unsafe { ALARM_TARGET_TIME = 0 };

            // Reset MATCH registers to maximum values to prevent spurious interrupts
            let r: &Regs = unsafe { &*I::ptr() };
            prime_match_registers(r);
        });
    }

    /// Check if an alarm has expired (call this from your interrupt handler)
    /// Returns true if the alarm was active and has now expired
    pub fn check_alarm_expired(&self, alarm: &Alarm) -> bool {
        if alarm.active.load(Ordering::Acquire) {
            alarm.active.store(false, Ordering::Release);

            // Execute callback
            if let Some(callback) = alarm.callback {
                callback();
            }

            // Set flag
            if let Some(flag) = &alarm.flag {
                flag.store(true, Ordering::Release);
            }

            true
        } else {
            false
        }
    }
}

/// Read current EVTIMER (Gray-coded) and convert to binary ticks.
#[inline(always)]
fn now_ticks_read() -> u64 {
    let r: &Regs = unsafe { &*pac::Ostimer0::ptr() };

    // Read high then low to minimize incoherent snapshots
    let hi = (r.evtimerh().read().evtimer_count_value().bits() as u64) & (EVTIMER_HI_MASK as u64);
    let lo = r.evtimerl().read().evtimer_count_value().bits() as u64;
    // Combine and convert from Gray code to binary
    let gray = lo | (hi << EVTIMER_HI_SHIFT);
    gray_to_bin(gray)
}

// Instance trait like other drivers, providing a PAC pointer for this OSTIMER instance
pub trait Instance: Gate<MrccPeriphConfig = OsTimerConfig> + PeripheralType {
    fn ptr() -> *const Regs;
}

#[inline(always)]
fn bin_to_gray(x: u64) -> u64 {
    x ^ (x >> 1)
}

#[inline(always)]
fn gray_to_bin(gray: u64) -> u64 {
    // More efficient iterative conversion using predefined shifts
    let mut bin = gray;
    for &shift in &GRAY_CONVERSION_SHIFTS {
        bin ^= bin >> shift;
    }
    bin
}

pub mod time_driver {
    use core::sync::atomic::Ordering;
    use core::task::Waker;

    use embassy_sync::waitqueue::AtomicWaker;
    use embassy_time_driver as etd;

    use super::{
        ALARM_ACTIVE, ALARM_CALLBACK, ALARM_FLAG, ALARM_TARGET_TIME, EVTIMER_HI_MASK, EVTIMER_HI_SHIFT,
        LOW_32_BIT_MASK, Regs, bin_to_gray, now_ticks_read,
    };
    use crate::clocks::periph_helpers::{OsTimerConfig, OstimerClockSel};
    use crate::clocks::{PoweredClock, enable_and_reset};
    use crate::interrupt::InterruptExt;
    use crate::pac;

    #[allow(non_camel_case_types)]
    pub(crate) struct _OSTIMER0_TIME_DRIVER {
        _x: (),
    }

    impl crate::clocks::Gate for _OSTIMER0_TIME_DRIVER {
        type MrccPeriphConfig = crate::clocks::periph_helpers::OsTimerConfig;

        #[inline]
        unsafe fn enable_clock() {
            let mrcc = unsafe { pac::Mrcc0::steal() };
            mrcc.mrcc_glb_cc1().modify(|_, w| w.ostimer0().enabled());
        }

        #[inline]
        unsafe fn disable_clock() {
            let mrcc = unsafe { pac::Mrcc0::steal() };
            mrcc.mrcc_glb_cc1().modify(|_r, w| w.ostimer0().disabled());
        }

        #[inline]
        fn is_clock_enabled() -> bool {
            let mrcc = unsafe { pac::Mrcc0::steal() };
            mrcc.mrcc_glb_cc1().read().ostimer0().is_enabled()
        }

        #[inline]
        unsafe fn release_reset() {
            let mrcc = unsafe { pac::Mrcc0::steal() };
            mrcc.mrcc_glb_rst1().modify(|_, w| w.ostimer0().enabled());
        }

        #[inline]
        unsafe fn assert_reset() {
            let mrcc = unsafe { pac::Mrcc0::steal() };
            mrcc.mrcc_glb_rst1().modify(|_, w| w.ostimer0().disabled());
        }

        #[inline]
        fn is_reset_released() -> bool {
            let mrcc = unsafe { pac::Mrcc0::steal() };
            mrcc.mrcc_glb_rst1().read().ostimer0().is_enabled()
        }
    }

    pub struct Driver;
    static TIMER_WAKER: AtomicWaker = AtomicWaker::new();

    impl etd::Driver for Driver {
        fn now(&self) -> u64 {
            // Use the hardware counter (frequency configured in init)
            super::now_ticks_read()
        }

        fn schedule_wake(&self, timestamp: u64, waker: &Waker) {
            let now = self.now();

            // If timestamp is in the past or very close to now, wake immediately
            if timestamp <= now {
                waker.wake_by_ref();
                return;
            }

            // Prevent scheduling too far in the future (beyond timer rollover)
            // This prevents wraparound issues
            let max_future = now + super::TIMER_MAX_VALUE;
            if timestamp > max_future {
                // For very long timeouts, wake immediately to avoid rollover issues
                waker.wake_by_ref();
                return;
            }

            // Register the waker first so any immediate wake below is observed by the executor.
            TIMER_WAKER.register(waker);

            let r: &Regs = unsafe { &*pac::Ostimer0::ptr() };

            critical_section::with(|_| {
                // Mask INTENA and clear flag
                r.osevent_ctrl()
                    .write(|w| w.ostimer_intrflag().clear_bit_by_one().ostimer_intena().clear_bit());

                // Read back to ensure W1C took effect on hardware
                let _ = r.osevent_ctrl().read().ostimer_intrflag().bit();

                if !super::wait_for_match_write_ready(r) {
                    super::prime_match_registers(r);

                    if !super::wait_for_match_write_ready(r) {
                        // If we can't safely program MATCH, wake immediately and leave INTENA masked.
                        waker.wake_by_ref();
                        return;
                    }
                }

                // Program MATCH (Gray-coded). Write low then high, then fence.
                let gray = bin_to_gray(timestamp);
                let l = (gray & LOW_32_BIT_MASK) as u32;

                let h = (((gray >> EVTIMER_HI_SHIFT) as u16) & EVTIMER_HI_MASK) as u16;

                r.match_l().write(|w| unsafe { w.match_value().bits(l) });
                r.match_h().write(|w| unsafe { w.match_value().bits(h) });

                if !super::wait_for_match_write_complete(r) {
                    waker.wake_by_ref();
                    return;
                }

                let now_after_program = super::now_ticks_read();
                let intrflag_set = r.osevent_ctrl().read().ostimer_intrflag().bit_is_set();
                if now_after_program >= timestamp && !intrflag_set {
                    waker.wake_by_ref();
                    return;
                }

                // Enable peripheral interrupt
                r.osevent_ctrl().write(|w| w.ostimer_intena().set_bit());
            });
        }
    }

    /// Install the global embassy-time driver and configure NVIC priority for OS_EVENT.
    pub fn init() {
        let _clock_freq = unsafe {
            enable_and_reset::<_OSTIMER0_TIME_DRIVER>(&OsTimerConfig {
                power: PoweredClock::AlwaysEnabled,
                source: OstimerClockSel::Clk1M,
            })
            .expect("Enabling OsTimer clock should not fail")
        };

        // Mask/clear at peripheral and set default MATCH
        let r: &Regs = unsafe { &*pac::Ostimer0::ptr() };
        super::prime_match_registers(r);

        // Configure NVIC for timer operation
        crate::interrupt::OS_EVENT.unpend();

        unsafe {
            crate::interrupt::OS_EVENT.enable();
        }
    }

    // Export the global time driver expected by embassy-time
    embassy_time_driver::time_driver_impl!(static DRIVER: Driver = Driver);

    /// To be called from the OS_EVENT IRQ.
    pub fn on_interrupt() {
        let r: &Regs = unsafe { &*pac::Ostimer0::ptr() };

        // Critical section to prevent races with schedule_wake
        critical_section::with(|_| {
            // Check if interrupt is actually pending and handle it atomically
            if r.osevent_ctrl().read().ostimer_intrflag().bit_is_set() {
                // Clear flag and disable interrupt atomically
                r.osevent_ctrl().write(|w| {
                    w.ostimer_intrflag()
                        .clear_bit_by_one() // Write-1-to-clear using safe helper
                        .ostimer_intena()
                        .clear_bit()
                });

                // Wake the waiting task
                TIMER_WAKER.wake();

                // Handle alarm callback if active and this interrupt is for the alarm
                if ALARM_ACTIVE.load(Ordering::SeqCst) {
                    let current_time = now_ticks_read();
                    let target_time = unsafe { ALARM_TARGET_TIME };

                    // Check if current time is close to alarm target time (within 1000 ticks for timing variations)
                    if current_time >= target_time && current_time <= target_time + 1000 {
                        ALARM_ACTIVE.store(false, Ordering::SeqCst);
                        unsafe { ALARM_TARGET_TIME = 0 };

                        // Execute callback if set
                        unsafe {
                            if let Some(callback) = ALARM_CALLBACK {
                                callback();
                            }
                        }

                        // Set flag if provided
                        unsafe {
                            if let Some(flag) = ALARM_FLAG {
                                (*flag).store(true, Ordering::SeqCst);
                            }
                        }
                    }
                }
            }
        });
    }
}

use crate::pac::interrupt;

#[allow(non_snake_case)]
#[interrupt]
fn OS_EVENT() {
    time_driver::on_interrupt()
}
