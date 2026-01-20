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

use core::cell::RefCell;
use core::hint::spin_loop;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU32, Ordering};

use cortex_m::asm::{dsb, isb};
use cortex_m::interrupt::InterruptNumber;
use cortex_m::peripheral::NVIC;
use cortex_m::register::basepri;
use critical_section;
#[cfg(feature = "defmt")]
use defmt::{error, trace};
use embassy_sync::blocking_mutex::Mutex;
#[cfg(not(feature = "defmt"))]
macro_rules! trace {
    ($($arg:tt)*) => {{}};
}
#[cfg(not(feature = "defmt"))]
macro_rules! error {
    ($($arg:tt)*) => {{}};
}
use embassy_stm32::NVIC_PRIO_BITS;
use embassy_stm32::peripherals::RNG;
use embassy_stm32::rng::Rng;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, block_for};

use super::bindings::{link_layer, mac};

// Missing constants from stm32-bindings - RADIO interrupt numbers
// For STM32WBA65RI, the RADIO interrupt is position 66 (between ADC4=65 and WKUP=67)
// Note: mac::RADIO_INTR_NUM is incorrectly set to 0 in stm32-bindings, so we override it here
const RADIO_INTR_NUM: u32 = 66; // 2.4 GHz RADIO global interrupt
const RADIO_SW_LOW_INTR_NUM: u32 = 67; // WKUP used as SW low interrupt

type Callback = unsafe extern "C" fn();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
struct RawInterrupt(u16);

impl RawInterrupt {
    fn new(irq: u32) -> Self {
        debug_assert!(irq <= u16::MAX as u32);
        Self(irq as u16)
    }
}

impl From<u32> for RawInterrupt {
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

static PRIO_HIGH_ISR_COUNTER: AtomicI32 = AtomicI32::new(0);
static PRIO_LOW_ISR_COUNTER: AtomicI32 = AtomicI32::new(0);
static PRIO_SYS_ISR_COUNTER: AtomicI32 = AtomicI32::new(0);
static LOCAL_BASEPRI_VALUE: AtomicU32 = AtomicU32::new(0);

static RADIO_SW_LOW_ISR_RUNNING_HIGH_PRIO: AtomicBool = AtomicBool::new(false);
static AHB5_SWITCHED_OFF: AtomicBool = AtomicBool::new(false);
static RADIO_SLEEP_TIMER_VAL: AtomicU32 = AtomicU32::new(0);

// Critical-section restore token for IRQ enable/disable pairing.
// Only written when the IRQ disable counter transitions 0->1, and consumed when it transitions 1->0.
static mut CS_RESTORE_STATE: Option<critical_section::RestoreState> = None;

// Optional hardware RNG instance for true random number generation.
// The RNG peripheral pointer is stored here to be used by LINKLAYER_PLAT_GetRNG.
// This must be set by the application using `set_rng_instance` before the link layer requests random numbers.
pub(crate) static mut HARDWARE_RNG: Option<&'static Mutex<CriticalSectionRawMutex, RefCell<Rng<'static, RNG>>>> = None;

fn store_callback(slot: &AtomicPtr<()>, cb: Option<Callback>) {
    let ptr = cb.map_or(ptr::null_mut(), |f| f as *mut ());
    slot.store(ptr, Ordering::Release);
}

fn load_callback(slot: &AtomicPtr<()>) -> Option<Callback> {
    let ptr = slot.load(Ordering::Acquire);
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { core::mem::transmute::<*mut (), Callback>(ptr) })
    }
}

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

fn counter_release(counter: &AtomicI32) -> bool {
    counter.fetch_sub(1, Ordering::SeqCst) <= 1
}

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

fn set_basepri_max(value: u8) {
    unsafe {
        if basepri::read() < value {
            basepri::write(value);
        }
    }
}

pub unsafe fn run_radio_high_isr() {
    if let Some(cb) = load_callback(&RADIO_CALLBACK) {
        cb();
    }
    // Wake the BLE runner task to process any resulting events
    super::runner::on_radio_interrupt();
}

pub unsafe fn run_radio_sw_low_isr() {
    if let Some(cb) = load_callback(&LOW_ISR_CALLBACK) {
        cb();
    }

    if RADIO_SW_LOW_ISR_RUNNING_HIGH_PRIO.swap(false, Ordering::AcqRel) {
        nvic_set_priority(RADIO_SW_LOW_INTR_NUM, pack_priority(mac::RADIO_SW_LOW_INTR_PRIO));
    }

    // Wake the BLE runner task to process any resulting events
    super::runner::on_radio_interrupt();
}

// /**
//   * @brief  Configure the necessary clock sources for the radio.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_ClockInit() {
    trace!("LINKLAYER_PLAT_ClockInit");

    // Enable AHB5ENR peripheral clock (bus CLK) for the radio
    // For STM32WBA65xx: RCC base = 0x4602_0C00, AHB5ENR offset = 0x098
    // RADIOEN bit = bit 0
    const RCC_AHB5ENR: *mut u32 = 0x4602_0C98 as *mut u32;
    const RADIOEN_BIT: u32 = 1 << 0;

    ptr::write_volatile(RCC_AHB5ENR, ptr::read_volatile(RCC_AHB5ENR) | RADIOEN_BIT);

    // Memory barrier to ensure clock is enabled before proceeding
    dsb();
    isb();

    trace!("LINKLAYER_PLAT_ClockInit: radio clock enabled");
}

// /**
//   * @brief  Link Layer active waiting loop.
//   * @param  delay: delay in us
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_DelayUs(delay: u32) {
    //   static uint8_t lock = 0;
    //   uint32_t t0;
    //   uint32_t primask_bit;
    //
    //   /* Enter critical section */
    //   primask_bit= __get_PRIMASK();
    //   __disable_irq();
    //
    //   if (lock == 0U)
    //   {
    //     /* Initialize counter */
    //     /* Reset cycle counter to prevent overflow
    //        As a us counter, it is assumed than even with re-entrancy,
    //        overflow will never happen before re-initializing this counter */
    //     DWT->CYCCNT = 0U;
    //     /* Enable DWT by safety but should be useless (as already set) */
    //     SET_BIT(DCB->DEMCR, DCB_DEMCR_TRCENA_Msk);
    //     /* Enable counter */
    //     SET_BIT(DWT->CTRL, DWT_CTRL_CYCCNTENA_Msk);
    //   }
    //   /* Increment 're-entrance' counter */
    //   lock++;
    //   /* Get starting time stamp */
    //   t0 = DWT->CYCCNT;
    //   /* Exit critical section */
    //  __set_PRIMASK(primask_bit);
    //
    //   /* Turn us into cycles */
    //   delay = delay * (SystemCoreClock / 1000000U);
    //   delay += t0;
    //
    //   /* Busy waiting loop */
    //   while (DWT->CYCCNT < delay)
    //   {
    //   };
    //
    //   /* Enter critical section */
    //   primask_bit= __get_PRIMASK();
    //   __disable_irq();
    //   if (lock == 1U)
    //   {
    //     /* Disable counter */
    //     CLEAR_BIT(DWT->CTRL, DWT_CTRL_CYCCNTENA_Msk);
    //   }
    //   /* Decrement 're-entrance' counter */
    //   lock--;
    //   /* Exit critical section */
    //  __set_PRIMASK(primask_bit);
    //
    trace!("LINKLAYER_PLAT_DelayUs: delay={}", delay);
    block_for(Duration::from_micros(u64::from(delay)));
}

// /**
//   * @brief  Link Layer assertion API
//   * @param  condition: conditional statement to be checked.
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_Assert(condition: u8) {
    if condition == 0 {
        panic!("LINKLAYER_PLAT assertion failed");
    }
}

// /**
//   * @brief  Enable/disable the Link Layer active clock (baseband clock).
//   * @param  enable: boolean value to enable (1) or disable (0) the clock.
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_WaitHclkRdy() {
    trace!("LINKLAYER_PLAT_WaitHclkRdy");
    if AHB5_SWITCHED_OFF.swap(false, Ordering::AcqRel) {
        let reference = RADIO_SLEEP_TIMER_VAL.load(Ordering::Acquire);
        trace!("LINKLAYER_PLAT_WaitHclkRdy: reference={}", reference);
        while reference == link_layer::ll_intf_cmn_get_slptmr_value() {
            spin_loop();
        }
    }
}

// /**
//   * @brief  Notify the Link Layer platform layer the system will enter in WFI
//   *         and AHB5 clock may be turned of regarding the 2.4Ghz radio state.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_NotifyWFIEnter() {
    //   /* Check if Radio state will allow the AHB5 clock to be cut */
    //
    //   /* AHB5 clock will be cut in the following cases:
    //    * - 2.4GHz radio is not in ACTIVE mode (in SLEEP or DEEPSLEEP mode).
    //    * - RADIOSMEN and STRADIOCLKON bits are at 0.
    //    */
    //   if((LL_PWR_GetRadioMode() != LL_PWR_RADIO_ACTIVE_MODE) ||
    //      ((__HAL_RCC_RADIO_IS_CLK_SLEEP_ENABLED() == 0) && (LL_RCC_RADIO_IsEnabledSleepTimerClock() == 0)))
    //   {
    //     AHB5_SwitchedOff = 1;
    //   }
    trace!("LINKLAYER_PLAT_NotifyWFIEnter");
    AHB5_SWITCHED_OFF.store(true, Ordering::Release);
}

// /**
//   * @brief  Notify the Link Layer platform layer the system exited WFI and AHB5
//   *         clock may be resynchronized as is may have been turned of during
//   *         low power mode entry.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_NotifyWFIExit() {
    trace!("LINKLAYER_PLAT_NotifyWFIExit");
    //   /* Check if AHB5 clock has been turned of and needs resynchronisation */
    if AHB5_SWITCHED_OFF.load(Ordering::Acquire) {
        //     /* Read sleep register as earlier as possible */
        let value = link_layer::ll_intf_cmn_get_slptmr_value();
        RADIO_SLEEP_TIMER_VAL.store(value, Ordering::Release);
    }
}

// /**
//   * @brief  Enable/disable the Link Layer active clock (baseband clock).
//   * @param  enable: boolean value to enable (1) or disable (0) the clock.
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_AclkCtrl(enable: u8) {
    trace!("LINKLAYER_PLAT_AclkCtrl: enable={}", enable);

    if enable != 0 {
        // Wait for HSE to be ready before enabling radio baseband clock
        // HSE (High-Speed External) oscillator is required for radio operation
        // For STM32WBA65xx: RCC base = 0x4602_0C00, CR offset = 0x000
        // RCC_CR register, bit 17 (HSERDY) indicates HSE ready status
        const RCC_CR: *const u32 = 0x4602_0C00 as *const u32;
        const HSERDY_BIT: u32 = 1 << 17;

        while (ptr::read_volatile(RCC_CR) & HSERDY_BIT) == 0 {
            spin_loop();
        }

        // Enable RADIO baseband clock (active clock)
        // For STM32WBA65xx: RCC base = 0x4602_0C00, RADIOENR offset = 0x208
        // RCC_RADIOENR register, bit 1 (BBCLKEN) enables the baseband clock
        const RCC_RADIOENR: *mut u32 = 0x4602_0E08 as *mut u32;
        const BBCLKEN_BIT: u32 = 1 << 1;

        ptr::write_volatile(RCC_RADIOENR, ptr::read_volatile(RCC_RADIOENR) | BBCLKEN_BIT);

        // Memory barrier to ensure clock is enabled before proceeding
        dsb();
        isb();

        trace!("LINKLAYER_PLAT_AclkCtrl: radio baseband clock enabled");
    } else {
        // Disable RADIO baseband clock (active clock)
        // For STM32WBA65xx: RCC base = 0x4602_0C00, RADIOENR offset = 0x208
        const RCC_RADIOENR: *mut u32 = 0x4602_0E08 as *mut u32;
        const BBCLKEN_BIT: u32 = 1 << 1;

        ptr::write_volatile(RCC_RADIOENR, ptr::read_volatile(RCC_RADIOENR) & !BBCLKEN_BIT);

        // Memory barrier
        dsb();
        isb();

        trace!("LINKLAYER_PLAT_AclkCtrl: radio baseband clock disabled");
    }
}

// /**
//   * @brief  Link Layer RNG request.
//   * @param  ptr_rnd: pointer to the variable that hosts the number.
//   * @param  len: number of byte of anthropy to get.
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_GetRNG(ptr_rnd: *mut u8, len: u32) {
    trace!("LINKLAYER_PLAT_GetRNG: ptr_rnd={:?}, len={}", ptr_rnd, len);
    if ptr_rnd.is_null() || len == 0 {
        return;
    }

    critical_section::with(|cs| {
        HARDWARE_RNG
            .as_ref()
            .unwrap()
            .borrow(cs)
            .borrow_mut()
            .fill_bytes(core::slice::from_raw_parts_mut(ptr_rnd, len as usize))
    });

    trace!("LINKLAYER_PLAT_GetRNG: generated {} random bytes", len);
}

// /**
//   * @brief  Initialize Link Layer radio high priority interrupt.
//   * @param  intr_cb: function pointer to assign for the radio high priority ISR routine.
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_SetupRadioIT(intr_cb: Option<Callback>) {
    trace!("LINKLAYER_PLAT_SetupRadioIT: intr_cb={:?}", intr_cb);
    store_callback(&RADIO_CALLBACK, intr_cb);

    if intr_cb.is_some() {
        nvic_set_priority(RADIO_INTR_NUM, pack_priority(mac::RADIO_INTR_PRIO_HIGH));
        nvic_enable(RADIO_INTR_NUM);
    } else {
        nvic_disable(RADIO_INTR_NUM);
    }
}

// /**
//   * @brief  Initialize Link Layer SW low priority interrupt.
//   * @param  intr_cb: function pointer to assign for the SW low priority ISR routine.
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_SetupSwLowIT(intr_cb: Option<Callback>) {
    trace!("LINKLAYER_PLAT_SetupSwLowIT: intr_cb={:?}", intr_cb);
    store_callback(&LOW_ISR_CALLBACK, intr_cb);

    if intr_cb.is_some() {
        nvic_set_priority(RADIO_SW_LOW_INTR_NUM, pack_priority(mac::RADIO_SW_LOW_INTR_PRIO));
        nvic_enable(RADIO_SW_LOW_INTR_NUM);
    } else {
        nvic_disable(RADIO_SW_LOW_INTR_NUM);
    }
}

// /**
//   * @brief  Trigger the link layer SW low interrupt.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_TriggerSwLowIT(priority: u8) {
    trace!("LINKLAYER_PLAT_TriggerSwLowIT: priority={}", priority);
    let active = nvic_get_active(RADIO_SW_LOW_INTR_NUM);

    //   /* Check if a SW low interrupt as already been raised.
    //    * Nested call far radio low isr are not supported
    //    **/
    if !active {
        let prio = if priority == 0 {
            //     /* No nested SW low ISR, default behavior */
            pack_priority(mac::RADIO_SW_LOW_INTR_PRIO)
        } else {
            pack_priority(mac::RADIO_INTR_PRIO_LOW)
        };
        nvic_set_priority(RADIO_SW_LOW_INTR_NUM, prio);
    } else if priority != 0 {
        //     /* Nested call detected */
        //     /* No change for SW radio low interrupt priority for the moment */
        //
        //     if(priority != 0)
        //     {
        //       /* At the end of current SW radio low ISR, this pending SW low interrupt
        //        * will run with RADIO_INTR_PRIO_LOW priority
        //        **/
        //       radio_sw_low_isr_is_running_high_prio = 1;
        //     }
        RADIO_SW_LOW_ISR_RUNNING_HIGH_PRIO.store(true, Ordering::Release);
    }

    nvic_set_pending(RADIO_SW_LOW_INTR_NUM);
}

// /**
//   * @brief  Enable interrupts.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_EnableIRQ() {
    trace!("LINKLAYER_PLAT_EnableIRQ");
    //   irq_counter = max(0,irq_counter-1);
    //
    //   if(irq_counter == 0)
    //   {
    //     /* When irq_counter reaches 0, restore primask bit */
    //     __set_PRIMASK(primask_bit);
    //   }
    if counter_release(&IRQ_COUNTER) {
        // When the counter reaches zero, restore prior interrupt state using the captured token.
        if let Some(token) = CS_RESTORE_STATE.take() {
            critical_section::release(token);
        }
    }
}

// /**
//   * @brief  Disable interrupts.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_DisableIRQ() {
    trace!("LINKLAYER_PLAT_DisableIRQ");
    //   if(irq_counter == 0)
    //   {
    //     /* Save primask bit at first interrupt disablement */
    //     primask_bit= __get_PRIMASK();
    //   }
    //   __disable_irq();
    //   irq_counter ++;
    if counter_acquire(&IRQ_COUNTER) {
        // Capture and disable using critical-section API on first disable.
        CS_RESTORE_STATE = Some(critical_section::acquire());
    }
}

// /**
//   * @brief  Enable specific interrupt group.
//   * @param  isr_type: mask for interrupt group to enable.
//   *         This parameter can be one of the following:
//   *         @arg LL_HIGH_ISR_ONLY: enable link layer high priority ISR.
//   *         @arg LL_LOW_ISR_ONLY: enable link layer SW low priority ISR.
//   *         @arg SYS_LOW_ISR: mask interrupts for all the other system ISR with
//   *              lower priority that link layer SW low interrupt.
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_EnableSpecificIRQ(isr_type: u8) {
    trace!("LINKLAYER_PLAT_EnableSpecificIRQ: isr_type={}", isr_type);
    //   if( (isr_type & LL_HIGH_ISR_ONLY) != 0 )
    //   {
    //     prio_high_isr_counter--;
    //     if(prio_high_isr_counter == 0)
    //     {
    //       /* When specific counter for link layer high ISR reaches 0, interrupt is enabled */
    //       HAL_NVIC_EnableIRQ(RADIO_INTR_NUM);
    //       /* USER CODE BEGIN LINKLAYER_PLAT_EnableSpecificIRQ_1 */
    //
    //       /* USER CODE END LINKLAYER_PLAT_EnableSpecificIRQ_1 */
    //     }
    //   }
    //
    //   if( (isr_type & LL_LOW_ISR_ONLY) != 0 )
    //   {
    //     prio_low_isr_counter--;
    //     if(prio_low_isr_counter == 0)
    //     {
    //       /* When specific counter for link layer SW low ISR reaches 0, interrupt is enabled */
    //       HAL_NVIC_EnableIRQ(RADIO_SW_LOW_INTR_NUM);
    //     }
    //
    //   }
    //
    //   if( (isr_type & SYS_LOW_ISR) != 0 )
    //   {
    //     prio_sys_isr_counter--;
    //     if(prio_sys_isr_counter == 0)
    //     {
    //       /* Restore basepri value */
    //       __set_BASEPRI(local_basepri_value);
    //     }
    //   }
    if (isr_type & link_layer::LL_HIGH_ISR_ONLY as u8) != 0 {
        if counter_release(&PRIO_HIGH_ISR_COUNTER) {
            nvic_enable(RADIO_INTR_NUM);
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

// /**
//   * @brief  Disable specific interrupt group.
//   * @param  isr_type: mask for interrupt group to disable.
//   *         This parameter can be one of the following:
//   *         @arg LL_HIGH_ISR_ONLY: disable link layer high priority ISR.
//   *         @arg LL_LOW_ISR_ONLY: disable link layer SW low priority ISR.
//   *         @arg SYS_LOW_ISR: unmask interrupts for all the other system ISR with
//   *              lower priority that link layer SW low interrupt.
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_DisableSpecificIRQ(isr_type: u8) {
    //   if( (isr_type & LL_HIGH_ISR_ONLY) != 0 )
    //   {
    //     prio_high_isr_counter++;
    //     if(prio_high_isr_counter == 1)
    //     {
    //       /* USER CODE BEGIN LINKLAYER_PLAT_DisableSpecificIRQ_1 */
    //
    //       /* USER CODE END LINKLAYER_PLAT_DisableSpecificIRQ_1 */
    //       /* When specific counter for link layer high ISR value is 1, interrupt is disabled */
    //       HAL_NVIC_DisableIRQ(RADIO_INTR_NUM);
    //     }
    //   }
    //
    //   if( (isr_type & LL_LOW_ISR_ONLY) != 0 )
    //   {
    //     prio_low_isr_counter++;
    //     if(prio_low_isr_counter == 1)
    //     {
    //       /* When specific counter for link layer SW low ISR value is 1, interrupt is disabled */
    //       HAL_NVIC_DisableIRQ(RADIO_SW_LOW_INTR_NUM);
    //     }
    //   }
    //
    //   if( (isr_type & SYS_LOW_ISR) != 0 )
    //   {
    //     prio_sys_isr_counter++;
    //     if(prio_sys_isr_counter == 1)
    //     {
    //       /* Save basepri register value */
    //       local_basepri_value = __get_BASEPRI();
    //
    //       /* Mask all other interrupts with lower priority that link layer SW low ISR */
    //       __set_BASEPRI_MAX(RADIO_INTR_PRIO_LOW<<4);
    //     }
    //   }
    trace!("LINKLAYER_PLAT_DisableSpecificIRQ: isr_type={}", isr_type);
    if (isr_type & link_layer::LL_HIGH_ISR_ONLY as u8) != 0 {
        if counter_acquire(&PRIO_HIGH_ISR_COUNTER) {
            nvic_disable(RADIO_INTR_NUM);
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

// /**
//   * @brief  Enable link layer high priority ISR only.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_EnableRadioIT() {
    trace!("LINKLAYER_PLAT_EnableRadioIT");
    nvic_enable(RADIO_INTR_NUM);
}

// /**
//   * @brief  Disable link layer high priority ISR only.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_DisableRadioIT() {
    trace!("LINKLAYER_PLAT_DisableRadioIT");
    nvic_disable(RADIO_INTR_NUM);
}

// /**
//   * @brief  Link Layer notification for radio activity start.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_StartRadioEvt() {
    trace!("LINKLAYER_PLAT_StartRadioEvt");
    //   __HAL_RCC_RADIO_CLK_SLEEP_ENABLE();
    //   NVIC_SetPriority(RADIO_INTR_NUM, RADIO_INTR_PRIO_HIGH);
    // #if (CFG_SCM_SUPPORTED == 1)
    //   scm_notifyradiostate(SCM_RADIO_ACTIVE);
    // #endif /* CFG_SCM_SUPPORTED */
    nvic_set_priority(RADIO_INTR_NUM, pack_priority(mac::RADIO_INTR_PRIO_HIGH));
    nvic_enable(RADIO_INTR_NUM);
}

// /**
//   * @brief  Link Layer notification for radio activity end.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_StopRadioEvt() {
    trace!("LINKLAYER_PLAT_StopRadioEvt");
    // {
    //   __HAL_RCC_RADIO_CLK_SLEEP_DISABLE();
    //   NVIC_SetPriority(RADIO_INTR_NUM, RADIO_INTR_PRIO_LOW);
    // #if (CFG_SCM_SUPPORTED == 1)
    //   scm_notifyradiostate(SCM_RADIO_NOT_ACTIVE);
    // #endif /* CFG_SCM_SUPPORTED */
    nvic_set_priority(RADIO_INTR_NUM, pack_priority(mac::RADIO_INTR_PRIO_LOW));
}

// /**
//   * @brief  Link Layer notification for RCO calibration start.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_RCOStartClbr() {
    trace!("LINKLAYER_PLAT_RCOStartClbr");
    // #if (CFG_LPM_LEVEL != 0)
    //   PWR_DisableSleepMode();
    //   /* Disabling stop mode prevents also from entering in standby */
    //   UTIL_LPM_SetStopMode(1U << CFG_LPM_LL_HW_RCO_CLBR, UTIL_LPM_DISABLE);
    // #endif /* (CFG_LPM_LEVEL != 0) */
    // #if (CFG_SCM_SUPPORTED == 1)
    //   scm_setsystemclock(SCM_USER_LL_HW_RCO_CLBR, HSE_32MHZ);
    //   while (LL_PWR_IsActiveFlag_VOS() == 0);
    // #endif /* (CFG_SCM_SUPPORTED == 1) */
}

// /**
//   * @brief  Link Layer notification for RCO calibration end.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_RCOStopClbr() {
    trace!("LINKLAYER_PLAT_RCOStopClbr");
    // #if (CFG_LPM_LEVEL != 0)
    //   PWR_EnableSleepMode();
    //   UTIL_LPM_SetStopMode(1U << CFG_LPM_LL_HW_RCO_CLBR, UTIL_LPM_ENABLE);
    // #endif /* (CFG_LPM_LEVEL != 0) */
    // #if (CFG_SCM_SUPPORTED == 1)
    //   scm_setsystemclock(SCM_USER_LL_HW_RCO_CLBR, HSE_16MHZ);
    // #endif /* (CFG_SCM_SUPPORTED == 1) */
}

// /**
//   * @brief  Link Layer requests temperature.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_RequestTemperature() {
    trace!("LINKLAYER_PLAT_RequestTemperature");
    // #if (USE_TEMPERATURE_BASED_RADIO_CALIBRATION == 1)
    //   ll_sys_bg_temperature_measurement();
    // #endif /* USE_TEMPERATURE_BASED_RADIO_CALIBRATION */
}

// /**
//   * @brief  PHY Start calibration.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_PhyStartClbr() {
    trace!("LINKLAYER_PLAT_PhyStartClbr");
}

// /**
//   * @brief  PHY Stop calibration.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_PhyStopClbr() {
    trace!("LINKLAYER_PLAT_PhyStopClbr");
}

// /**
//  * @brief Notify the upper layer that new Link Layer timings have been applied.
//  * @param evnt_timing[in]: Evnt_timing_t pointer to structure contains drift time , execution time and scheduling time
//  * @retval None.
//  */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_SCHLDR_TIMING_UPDATE_NOT(_timings: *const link_layer::Evnt_timing_t) {
    trace!("LINKLAYER_PLAT_SCHLDR_TIMING_UPDATE_NOT: timings={:?}", _timings);
}

// /**
//   * @brief  Get the ST company ID.
//   * @param  None
//   * @retval Company ID
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_GetSTCompanyID() -> u32 {
    trace!("LINKLAYER_PLAT_GetSTCompanyID");
    // STMicroelectronics Bluetooth SIG Company Identifier
    // TODO: Pull in update from latest stm32-generated-data
    0x0030
}

// /**
//   * @brief  Get the Unique Device Number (UDN).
//   * @param  None
//   * @retval UDN
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_GetUDN() -> u32 {
    trace!("LINKLAYER_PLAT_GetUDN");
    // Read the first 32 bits of the STM32 unique 96-bit ID
    let uid = embassy_stm32::uid::uid();
    u32::from_le_bytes([uid[0], uid[1], uid[2], uid[3]])
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_DEBUG_SIGNAL_SET() {
    // Debug signal - no-op in release builds
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_DEBUG_SIGNAL_RESET() {
    // Debug signal - no-op in release builds
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_DEBUG_SIGNAL_TOGGLE() {
    // Debug signal - no-op in release builds
}

// BLE Platform functions required by BLE stack

/// Initialize BLE platform layer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_Init() {
    trace!("BLEPLAT_Init");
    // Platform initialization is already done in linklayer_plat_init()
    // This function is called by BLE stack init
}

/// Get random numbers from RNG
///
/// # Arguments
/// * `n` - Number of 32-bit random values to generate (1-4)
/// * `val` - Pointer to array where random values will be stored
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_RngGet(n: u8, val: *mut u32) {
    trace!("BLEPLAT_RngGet: n={}", n);

    if val.is_null() || n == 0 {
        return;
    }

    critical_section::with(|cs| {
        HARDWARE_RNG
            .as_ref()
            .unwrap()
            .borrow(cs)
            .borrow_mut()
            .fill_bytes(core::slice::from_raw_parts_mut(val as *mut u8, n as usize * 4));
    });
}

/// AES ECB encrypt function
///
/// Used by the BLE stack for random address hash calculation.
///
/// # Arguments
/// * `key` - 16-byte AES key
/// * `input` - 16-byte input plaintext
/// * `output` - 16-byte output ciphertext
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_AesEcbEncrypt(key: *const u8, input: *const u8, output: *mut u8) {
    trace!("BLEPLAT_AesEcbEncrypt");

    if key.is_null() || input.is_null() || output.is_null() {
        error!("BLEPLAT_AesEcbEncrypt: null pointer");
        return;
    }

    // Use the STM32 AES hardware peripheral
    // For now, use software AES as a fallback since we don't have async context
    // In a production implementation, you'd want to use the hardware AES peripheral

    // Simple software AES-128 ECB encryption using the AES peripheral in blocking mode
    // Note: This is a simplified implementation. A proper implementation would use
    // the STM32 AES hardware peripheral.

    // Copy input to output as placeholder (real impl would do actual AES)
    // For security-sensitive operations, implement proper AES here
    core::ptr::copy_nonoverlapping(input, output, 16);

    // TODO: Implement proper AES-128 ECB encryption using hardware AES peripheral
    // For now, we use a stub that just copies data
    // This is NOT secure and needs to be replaced with actual AES encryption
    trace!("BLEPLAT_AesEcbEncrypt: WARNING - using stub implementation");
}

/// AES CMAC set key function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_AesCmacSetKey(_key: *const u8) {
    trace!("BLEPLAT_AesCmacSetKey");
    // TODO: Implement CMAC key setup
}

/// AES CMAC compute function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_AesCmacCompute(_input: *const u8, _input_length: u32, _output_tag: *mut u8) {
    trace!("BLEPLAT_AesCmacCompute");
    // TODO: Implement CMAC computation
}

/// Start a BLE stack timer
///
/// # Arguments
/// * `id` - Timer ID
/// * `timeout` - Timeout in milliseconds
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_TimerStart(_id: u16, _timeout: u32) -> u8 {
    trace!("BLEPLAT_TimerStart: id={}, timeout={}", _id, _timeout);
    // BLE timer implementation
    // The BLE stack uses timers for various protocol timeouts
    // For embassy integration, these would typically be handled by the async executor
    // For now, we return success and let the BLE stack handle timeouts via polling
    0 // Success
}

/// Stop a BLE stack timer
///
/// # Arguments
/// * `id` - Timer ID to stop
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_TimerStop(_id: u16) {
    trace!("BLEPLAT_TimerStop: id={}", _id);
    // Stop the specified timer
    // For embassy integration, this would cancel any pending timer
}

/// NVM store function for BLE stack
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_NvmStore(_ptr: *const u64, _size: u16) {
    trace!("BLEPLAT_NvmStore: size={}", _size);
    // NVM storage for BLE bonding data, etc.
    // TODO: Implement persistent storage if needed
}

// BLEPLAT return codes
const BLEPLAT_BUSY: i32 = -2;

/// Start P-256 public key generation
/// This is used for BLE secure connections
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_PkaStartP256Key(_local_private_key: *const u32) -> i32 {
    trace!("BLEPLAT_PkaStartP256Key");
    // PKA (Public Key Accelerator) not implemented yet
    // Return BUSY to indicate operation not supported
    BLEPLAT_BUSY
}

/// Read result of P-256 public key generation
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_PkaReadP256Key(_local_public_key: *mut u32) -> i32 {
    trace!("BLEPLAT_PkaReadP256Key");
    // PKA not implemented
    BLEPLAT_BUSY
}

/// Start DH key computation
/// This is used for BLE secure connections
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_PkaStartDhKey(_local_private_key: *const u32, _remote_public_key: *const u32) -> i32 {
    trace!("BLEPLAT_PkaStartDhKey");
    // PKA not implemented
    BLEPLAT_BUSY
}

/// Read result of DH key computation
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_PkaReadDhKey(_dh_key: *mut u32) -> i32 {
    trace!("BLEPLAT_PkaReadDhKey");
    // PKA not implemented
    BLEPLAT_BUSY
}

/// BLE stack HCI event indication callback
/// This is called by the BLE stack when HCI events arrive
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLECB_Indication(data: *const u8, length: u16, _ext_data: *const u8, _ext_length: u16) -> u8 {
    if data.is_null() || length == 0 {
        return 1; // Error
    }

    // Convert to slice
    let event_data = core::slice::from_raw_parts(data, length as usize);

    #[cfg(feature = "defmt")]
    defmt::trace!(
        "BLECB_Indication: event_code=0x{:02X}, length={}",
        event_data[0],
        length
    );

    // Parse and queue the event for processing
    if let Some(event) = super::hci::event::Event::parse(event_data) {
        match super::hci::event::try_send_event(event) {
            Ok(_) => {
                #[cfg(feature = "defmt")]
                defmt::trace!("Event queued successfully");

                // Signal BleStack_Process to run again
                // This is equivalent to Sidewalk SDK's osSemaphoreRelease(BleHostSemaphore)
                super::runner::BLE_WAKER.wake();
            }
            Err(_) => {
                #[cfg(feature = "defmt")]
                defmt::warn!("Event queue full, dropping event");
            }
        }
    } else {
        #[cfg(feature = "defmt")]
        defmt::warn!("Failed to parse HCI event");
    }

    0 // Success
}
