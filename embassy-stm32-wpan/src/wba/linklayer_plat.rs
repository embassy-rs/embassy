// /* USER CODE BEGIN Header */
// /**
//   ******************************************************************************
//   * @file    linklayer_plat.c
//   * @author  MCD Application Team
//   * @brief   Source file for the linklayer plateform adaptation layer
//   ******************************************************************************
//   * @attention
//   *
//   * Copyright (c) 2024 STMicroelectronics.
//   * All rights reserved.
//   *
//   * This software is licensed under terms that can be found in the LICENSE file
//   * in the root directory of this software component.
//   * If no LICENSE file comes with this software, it is provided AS-IS.
//   *
//   ******************************************************************************
//   */
// /* USER CODE END Header */
//
// #include "stm32wbaxx_hal.h"
// #include "stm32wbaxx_hal_conf.h"
// #include "stm32wbaxx_ll_rcc.h"
//
// #include "app_common.h"
// #include "app_conf.h"
// #include "linklayer_plat.h"
// #include "scm.h"
// #include "log_module.h"
// #if (USE_TEMPERATURE_BASED_RADIO_CALIBRATION == 1)
// #include "adc_ctrl.h"
// #endif /* (USE_TEMPERATURE_BASED_RADIO_CALIBRATION == 1) */
//
// #if (CFG_LPM_LEVEL != 0)
// #include "stm32_lpm.h"
// #include "stm32_lpm_if.h"
// #endif /* (CFG_LPM_LEVEL != 0) */
//
// /* USER CODE BEGIN Includes */
//
// /* USER CODE END Includes */
//
// #define max(a,b) ((a) > (b) ? a : b)
//
// /* 2.4GHz RADIO ISR callbacks */
// void (*radio_callback)(void) = NULL;
// void (*low_isr_callback)(void) = NULL;
//
// /* RNG handle */
// extern RNG_HandleTypeDef hrng;
//
// #if (USE_TEMPERATURE_BASED_RADIO_CALIBRATION == 1)
// /* Link Layer temperature request from background */
// extern void ll_sys_bg_temperature_measurement(void);
// #endif /* (USE_TEMPERATURE_BASED_RADIO_CALIBRATION == 1) */
//
// /* Radio critical sections */
// static uint32_t primask_bit = 0;
// volatile int32_t prio_high_isr_counter = 0;
// volatile int32_t prio_low_isr_counter = 0;
// volatile int32_t prio_sys_isr_counter = 0;
// volatile int32_t irq_counter = 0;
// volatile uint32_t local_basepri_value = 0;
//
// /* Radio SW low ISR global variable */
// volatile uint8_t radio_sw_low_isr_is_running_high_prio = 0;
//
// /* Radio bus clock control variables */
// uint8_t AHB5_SwitchedOff = 0;
// uint32_t radio_sleep_timer_val = 0;
//
// /* USER CODE BEGIN LINKLAYER_PLAT 0 */
//
// /* USER CODE END LINKLAYER_PLAT 0 */
#![cfg(feature = "wba")]
#![allow(clippy::missing_safety_doc)]

//! STM32WBA Link Layer platform adaptation layer.
//!
//! Based on STMicroelectronics original C source `linklayer_plat.c` (2024).

use core::hint::spin_loop;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU32, Ordering};

use cortex_m::asm::{dsb, isb};
use cortex_m::interrupt::InterruptNumber;
use cortex_m::peripheral::NVIC;
use cortex_m::register::{basepri, primask};
use embassy_stm32::NVIC_PRIO_BITS;
use embassy_time::{Duration, block_for};

use super::bindings::{link_layer, mac};

// Missing constant from stm32-bindings - RADIO_SW_LOW interrupt number
// For STM32WBA, this is typically RADIO_IRQ_BUSY (interrupt 43)
const RADIO_SW_LOW_INTR_NUM: u32 = 43;

type Callback = unsafe extern "C" fn();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
struct RawInterrupt(u16);

impl RawInterrupt {
    #[inline(always)]
    fn new(irq: u32) -> Self {
        debug_assert!(irq <= u16::MAX as u32);
        Self(irq as u16)
    }
}

impl From<u32> for RawInterrupt {
    #[inline(always)]
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

unsafe impl InterruptNumber for RawInterrupt {
    fn number(self) -> u16 {
        self.0
    }
}

static RADIO_CALLBACK: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());
static LOW_ISR_CALLBACK: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

static IRQ_COUNTER: AtomicI32 = AtomicI32::new(0);
static PRIMASK_SNAPSHOT: AtomicU32 = AtomicU32::new(0);

static PRIO_HIGH_ISR_COUNTER: AtomicI32 = AtomicI32::new(0);
static PRIO_LOW_ISR_COUNTER: AtomicI32 = AtomicI32::new(0);
static PRIO_SYS_ISR_COUNTER: AtomicI32 = AtomicI32::new(0);
static LOCAL_BASEPRI_VALUE: AtomicU32 = AtomicU32::new(0);

static RADIO_SW_LOW_ISR_RUNNING_HIGH_PRIO: AtomicBool = AtomicBool::new(false);
static AHB5_SWITCHED_OFF: AtomicBool = AtomicBool::new(false);
static RADIO_SLEEP_TIMER_VAL: AtomicU32 = AtomicU32::new(0);

static PRNG_STATE: AtomicU32 = AtomicU32::new(0);
static PRNG_INIT: AtomicBool = AtomicBool::new(false);

unsafe extern "C" {
    static SystemCoreClock: u32;
}

#[inline(always)]
fn read_system_core_clock() -> u32 {
    unsafe { ptr::read_volatile(&SystemCoreClock) }
}

#[inline(always)]
fn store_callback(slot: &AtomicPtr<()>, cb: Option<Callback>) {
    let ptr = cb.map_or(ptr::null_mut(), |f| f as *mut ());
    slot.store(ptr, Ordering::Release);
}

#[inline(always)]
fn load_callback(slot: &AtomicPtr<()>) -> Option<Callback> {
    let ptr = slot.load(Ordering::Acquire);
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { core::mem::transmute::<*mut (), Callback>(ptr) })
    }
}

#[inline(always)]
fn priority_shift() -> u8 {
    8 - NVIC_PRIO_BITS as u8
}

fn pack_priority(raw: u32) -> u8 {
    let shift = priority_shift();
    let priority_bits = NVIC_PRIO_BITS as u32;
    let mask = if priority_bits >= 32 {
        u32::MAX
    } else {
        (1u32 << priority_bits) - 1
    };
    let clamped = raw & mask;
    (clamped << u32::from(shift)) as u8
}

#[inline(always)]
fn counter_release(counter: &AtomicI32) -> bool {
    counter.fetch_sub(1, Ordering::SeqCst) <= 1
}

#[inline(always)]
fn counter_acquire(counter: &AtomicI32) -> bool {
    counter.fetch_add(1, Ordering::SeqCst) == 0
}

unsafe fn nvic_enable(irq: u32) {
    NVIC::unmask(RawInterrupt::new(irq));
    dsb();
    isb();
}

unsafe fn nvic_disable(irq: u32) {
    NVIC::mask(RawInterrupt::new(irq));
    dsb();
    isb();
}

unsafe fn nvic_set_pending(irq: u32) {
    NVIC::pend(RawInterrupt::new(irq));
    dsb();
    isb();
}

unsafe fn nvic_get_active(irq: u32) -> bool {
    NVIC::is_active(RawInterrupt::new(irq))
}

unsafe fn nvic_set_priority(irq: u32, priority: u8) {
    // STM32WBA is ARMv8-M, which uses byte-accessible IPR registers
    let nvic = &*NVIC::PTR;
    nvic.ipr[irq as usize].write(priority);

    dsb();
    isb();
}

#[inline(always)]
fn set_basepri_max(value: u8) {
    unsafe {
        if basepri::read() < value {
            basepri::write(value);
        }
    }
}

fn prng_next() -> u32 {
    #[inline]
    fn xorshift(mut x: u32) -> u32 {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x
    }

    if !PRNG_INIT.load(Ordering::Acquire) {
        let seed = unsafe {
            let timer = link_layer::ll_intf_cmn_get_slptmr_value();
            let core_clock = read_system_core_clock();
            timer ^ core_clock ^ 0x6C8E_9CF5
        };
        PRNG_STATE.store(seed, Ordering::Relaxed);
        PRNG_INIT.store(true, Ordering::Release);
    }

    let mut current = PRNG_STATE.load(Ordering::Relaxed);
    loop {
        let next = xorshift(current);
        match PRNG_STATE.compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Relaxed) {
            Ok(_) => return next,
            Err(v) => current = v,
        }
    }
}

pub unsafe fn run_radio_high_isr() {
    if let Some(cb) = load_callback(&RADIO_CALLBACK) {
        cb();
    }
}

pub unsafe fn run_radio_sw_low_isr() {
    if let Some(cb) = load_callback(&LOW_ISR_CALLBACK) {
        cb();
    }

    if RADIO_SW_LOW_ISR_RUNNING_HIGH_PRIO.swap(false, Ordering::AcqRel) {
        nvic_set_priority(RADIO_SW_LOW_INTR_NUM, pack_priority(mac::RADIO_SW_LOW_INTR_PRIO));
    }
}

/// Initialize radio-related clock prerequisites.
///
/// Currently this touches the sleep timer to ensure the Link Layer common
/// interface is initialized. It does not actively reconfigure clocks.
///
/// # Safety
/// Called from the vendor Link Layer. Must run in a context where accessing
/// the LL common interface is safe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_ClockInit() {
    let _ = link_layer::ll_intf_cmn_get_slptmr_value();
}

/// Busy-wait for the requested duration in microseconds.
///
/// Blocks the current context until `delay` microseconds have elapsed.
///
/// # Safety
/// Must be called only in contexts where busy-waiting is acceptable (e.g. no
/// hard real-time deadlines are violated).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_DelayUs(delay: u32) {
    block_for(Duration::from_micros(u64::from(delay)));
}

/// Assert a condition and panic if it is false.
///
/// # Safety
/// None beyond general panic considerations; will abort/panic the program if
/// `condition == 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_Assert(condition: u8) {
    if condition == 0 {
        panic!("LINKLAYER_PLAT assertion failed");
    }
}

/// Wait for the AHB5 clock domain to be ready after low-power entry.
///
/// If the platform flagged AHB5 as switched off before WFI, this waits until
/// the sleep timer ticks, indicating the bus has resumed.
///
/// # Safety
/// Spins while waiting; must be safe to busy-wait in the calling context.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_WaitHclkRdy() {
    if AHB5_SWITCHED_OFF.swap(false, Ordering::AcqRel) {
        let reference = RADIO_SLEEP_TIMER_VAL.load(Ordering::Acquire);
        while reference == link_layer::ll_intf_cmn_get_slptmr_value() {
            spin_loop();
        }
    }
}

/// Notify that the system is entering WFI and AHB5 may be turned off depending
/// on radio state.
///
/// # Safety
/// None; this only updates internal state used to resynchronize after WFI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_NotifyWFIEnter() {
    AHB5_SWITCHED_OFF.store(true, Ordering::Release);
}

/// Notify that the system exited WFI and capture a reference sleep timer value.
///
/// If AHB5 was flagged as switched off on entry, records the current sleep
/// timer value for later synchronization in [`LINKLAYER_PLAT_WaitHclkRdy`].
///
/// # Safety
/// None; reads a monotonic timer from the LL common interface.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_NotifyWFIExit() {
    if AHB5_SWITCHED_OFF.load(Ordering::Acquire) {
        let value = link_layer::ll_intf_cmn_get_slptmr_value();
        RADIO_SLEEP_TIMER_VAL.store(value, Ordering::Release);
    }
}

/// Control the active clock (placeholder).
///
/// Currently a no-op. Present for API compatibility with vendor code.
///
/// # Safety
/// None; function does nothing.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_AclkCtrl(_enable: u8) {}

/// Fill a buffer with pseudo-random bytes.
///
/// This uses a xorshift32 PRNG seeded from the sleep timer and core clock.
/// It is not cryptographically secure and is intended only for non-security
/// purposes.
///
/// # Safety
/// - `ptr_rnd` must be valid for writes of `len` bytes.
/// - The memory region must not alias mutable references elsewhere.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_GetRNG(ptr_rnd: *mut u8, len: u32) {
    if ptr_rnd.is_null() || len == 0 {
        return;
    }

    for i in 0..len {
        let byte = (prng_next() >> ((i & 3) * 8)) as u8;
        ptr::write_volatile(ptr_rnd.add(i as usize), byte);
    }
}

/// Configure the radio high-priority interrupt callback and NVIC state.
///
/// When `intr_cb` is `Some`, sets the NVIC priority to
/// `RADIO_INTR_PRIO_HIGH` and unmasks the interrupt. Passing `None` disables
/// the interrupt.
///
/// # Safety
/// `intr_cb` must be an ISR-safe function. Alters NVIC state globally.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_SetupRadioIT(intr_cb: Option<Callback>) {
    store_callback(&RADIO_CALLBACK, intr_cb);

    if intr_cb.is_some() {
        nvic_set_priority(mac::RADIO_INTR_NUM, pack_priority(mac::RADIO_INTR_PRIO_HIGH));
        nvic_enable(mac::RADIO_INTR_NUM);
    } else {
        nvic_disable(mac::RADIO_INTR_NUM);
    }
}

/// Configure the software low-priority radio interrupt callback and NVIC state.
///
/// When `intr_cb` is `Some`, sets the NVIC priority to
/// `RADIO_SW_LOW_INTR_PRIO` and unmasks the interrupt. Passing `None`
/// disables the interrupt.
///
/// # Safety
/// `intr_cb` must be ISR-safe. Alters NVIC state globally.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_SetupSwLowIT(intr_cb: Option<Callback>) {
    store_callback(&LOW_ISR_CALLBACK, intr_cb);

    if intr_cb.is_some() {
        nvic_set_priority(RADIO_SW_LOW_INTR_NUM, pack_priority(mac::RADIO_SW_LOW_INTR_PRIO));
        nvic_enable(RADIO_SW_LOW_INTR_NUM);
    } else {
        nvic_disable(RADIO_SW_LOW_INTR_NUM);
    }
}

/// Trigger the software low-priority radio interrupt.
///
/// If `priority` is non-zero, elevates the interrupt to the low radio priority
/// for this trigger or the next run when already active.
///
/// # Safety
/// Alters NVIC pending and priority state; must be safe for the system.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_TriggerSwLowIT(priority: u8) {
    let active = nvic_get_active(RADIO_SW_LOW_INTR_NUM);

    if !active {
        let prio = if priority == 0 {
            pack_priority(mac::RADIO_SW_LOW_INTR_PRIO)
        } else {
            pack_priority(mac::RADIO_INTR_PRIO_LOW)
        };
        nvic_set_priority(RADIO_SW_LOW_INTR_NUM, prio);
    } else if priority != 0 {
        RADIO_SW_LOW_ISR_RUNNING_HIGH_PRIO.store(true, Ordering::Release);
    }

    nvic_set_pending(RADIO_SW_LOW_INTR_NUM);
}

/// Enable interrupts using a reference-counted scheme.
///
/// When the internal counter reaches zero, restores the previous PRIMASK
/// snapshot and enables or keeps interrupts disabled accordingly.
///
/// # Safety
/// Must be paired with prior calls to [`LINKLAYER_PLAT_DisableIRQ`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_EnableIRQ() {
    if counter_release(&IRQ_COUNTER) {
        let snapshot = PRIMASK_SNAPSHOT.swap(0, Ordering::Relaxed);
        if snapshot != 0 {
            cortex_m::interrupt::disable();
        } else {
            cortex_m::interrupt::enable();
        }
    }
}

/// Disable interrupts using a reference-counted scheme.
///
/// Captures the current PRIMASK state on the first disable and then disables
/// interrupts. Must be balanced with [`LINKLAYER_PLAT_EnableIRQ`].
///
/// # Safety
/// Affects global interrupt state; may impact system timing and ISRs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_DisableIRQ() {
    if counter_acquire(&IRQ_COUNTER) {
        let snapshot = if primask::read().is_active() { 1 } else { 0 };
        PRIMASK_SNAPSHOT.store(snapshot, Ordering::Relaxed);
    }
    cortex_m::interrupt::disable();
}

/// Enable specific Link Layer interrupt groups.
///
/// - `LL_HIGH_ISR_ONLY`: Unmask high-priority radio ISR.
/// - `LL_LOW_ISR_ONLY`: Unmask software low-priority radio ISR.
/// - `SYS_LOW_ISR`: Lower BASEPRI mask to re-enable lower-priority system ISRs.
///
/// Uses internal reference counters so multiple disables/enables can be nested.
///
/// # Safety
/// Alters NVIC and BASEPRI state globally.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_EnableSpecificIRQ(isr_type: u8) {
    if (isr_type & link_layer::LL_HIGH_ISR_ONLY as u8) != 0 {
        if counter_release(&PRIO_HIGH_ISR_COUNTER) {
            nvic_enable(mac::RADIO_INTR_NUM);
        }
    }

    if (isr_type & link_layer::LL_LOW_ISR_ONLY as u8) != 0 {
        if counter_release(&PRIO_LOW_ISR_COUNTER) {
            nvic_enable(RADIO_SW_LOW_INTR_NUM);
        }
    }

    if (isr_type & link_layer::SYS_LOW_ISR as u8) != 0 {
        if counter_release(&PRIO_SYS_ISR_COUNTER) {
            let stored = LOCAL_BASEPRI_VALUE.load(Ordering::Relaxed) as u8;
            basepri::write(stored);
        }
    }
}

/// Disable specific Link Layer interrupt groups.
///
/// - `LL_HIGH_ISR_ONLY`: Mask high-priority radio ISR.
/// - `LL_LOW_ISR_ONLY`: Mask software low-priority radio ISR.
/// - `SYS_LOW_ISR`: Raise BASEPRI to mask system ISRs lower than SW low-priority.
///
/// Uses internal reference counters so multiple disables/enables can be nested.
///
/// # Safety
/// Alters NVIC and BASEPRI state globally.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_DisableSpecificIRQ(isr_type: u8) {
    if (isr_type & link_layer::LL_HIGH_ISR_ONLY as u8) != 0 {
        if counter_acquire(&PRIO_HIGH_ISR_COUNTER) {
            nvic_disable(mac::RADIO_INTR_NUM);
        }
    }

    if (isr_type & link_layer::LL_LOW_ISR_ONLY as u8) != 0 {
        if counter_acquire(&PRIO_LOW_ISR_COUNTER) {
            nvic_disable(RADIO_SW_LOW_INTR_NUM);
        }
    }

    if (isr_type & link_layer::SYS_LOW_ISR as u8) != 0 {
        if counter_acquire(&PRIO_SYS_ISR_COUNTER) {
            let current = basepri::read();
            LOCAL_BASEPRI_VALUE.store(current.into(), Ordering::Relaxed);
            set_basepri_max(pack_priority(mac::RADIO_INTR_PRIO_LOW));
        }
    }
}

/// Unmask the radio high-priority interrupt.
///
/// # Safety
/// Alters NVIC state globally.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_EnableRadioIT() {
    nvic_enable(mac::RADIO_INTR_NUM);
}

/// Mask the radio high-priority interrupt.
///
/// # Safety
/// Alters NVIC state globally.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_DisableRadioIT() {
    nvic_disable(mac::RADIO_INTR_NUM);
}

/// Notify that a radio activity is starting.
///
/// Sets the radio interrupt priority to high and unmasks it.
///
/// # Safety
/// Alters NVIC state globally.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_StartRadioEvt() {
    nvic_set_priority(mac::RADIO_INTR_NUM, pack_priority(mac::RADIO_INTR_PRIO_HIGH));
    nvic_enable(mac::RADIO_INTR_NUM);
}

/// Notify that a radio activity ended.
///
/// Lowers the radio interrupt priority to its low setting.
///
/// # Safety
/// Alters NVIC state globally.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_StopRadioEvt() {
    nvic_set_priority(mac::RADIO_INTR_NUM, pack_priority(mac::RADIO_INTR_PRIO_LOW));
}

/// Notify that RCO calibration is starting (placeholder).
///
/// Currently a no-op.
///
/// # Safety
/// None.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_RCOStartClbr() {}

/// Notify that RCO calibration ended (placeholder).
///
/// Currently a no-op.
///
/// # Safety
/// None.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_RCOStopClbr() {}

/// Request a temperature measurement for radio calibration (placeholder).
///
/// Currently a no-op.
///
/// # Safety
/// None.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_RequestTemperature() {}

/// Notify that PHY calibration is starting (placeholder).
///
/// Currently a no-op.
///
/// # Safety
/// None.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_PhyStartClbr() {}

/// Notify that PHY calibration ended (placeholder).
///
/// Currently a no-op.
///
/// # Safety
/// None.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_PhyStopClbr() {}

/// Notify that new Link Layer scheduler timings have been applied (placeholder).
///
/// Currently a no-op.
///
/// # Safety
/// None.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_SCHLDR_TIMING_UPDATE_NOT(_timings: *const link_layer::Evnt_timing_t) {}

/// Return the STMicroelectronics Bluetooth SIG Company Identifier.
///
/// Value: `0x0030`.
///
/// # Safety
/// None.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_GetSTCompanyID() -> u32 {
    // STMicroelectronics Bluetooth SIG Company Identifier
    // TODO: Pull in update from latest stm32-generated-data
    0x0030
}

/// Return the lower 32 bits of the STM32 unique 96-bit device identifier.
///
/// Note: This may differ from the ST-defined UDN encoding found in some device
/// registers/documents; it returns the first word of the unique ID.
///
/// # Safety
/// None.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_GetUDN() -> u32 {
    // Read the first 32 bits of the STM32 unique 96-bit ID
    let uid = embassy_stm32::uid::uid();
    u32::from_le_bytes([uid[0], uid[1], uid[2], uid[3]])
}
