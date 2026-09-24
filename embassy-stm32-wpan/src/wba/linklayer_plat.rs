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

use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU32, AtomicUsize, Ordering, compiler_fence};

use cortex_m::interrupt::InterruptNumber;
use cortex_m::peripheral::NVIC;
use cortex_m::register::basepri;
use critical_section;
use embassy_crypto::{Aes128, Aes128Ccm, Aes128Cmac, p256};
use embassy_stm32::NVIC_PRIO_BITS;
use embassy_stm32::pac::{FLASH, PWR, RCC};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::zerocopy_channel;
use embassy_time::{Duration, Instant, block_for};
#[cfg(not(feature = "ble-stack-llo"))]
use stm32_bindings::ble::BLEPLATCB_TimerExpiry;

use crate::controller::ChannelPacket;
use crate::platform::Platform;
use crate::wba::bindings::{link_layer, mac};
use crate::wba::host_if::{TASK_BLE_HOST_MASK, TASK_PRIO_BLE_HOST};
use crate::wba::platform::P256Request;
use crate::wba::util_seq;

// RADIO interrupt numbers for STM32WBA
// RADIO interrupt is position 66
// SW low interrupt uses HASH peripheral interrupt (61) as per ST's implementation
const RADIO_INTR_NUM: u32 = 66; // 2.4 GHz RADIO global interrupt
const RADIO_SW_LOW_INTR_NUM: u32 = 61; // HASH interrupt used as SW low interrupt (per ST reference)

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

// Global platform reference, set by `Controller::new` and used by the BLE
// stack's platform callbacks (P-256 requests, event channel).
pub(crate) static mut PLATFORM: Option<&'static Platform> = None;

pub(crate) static mut EVENT_CHANNEL: Option<zerocopy_channel::Sender<'static, CriticalSectionRawMutex, ChannelPacket>> =
    None;

const fn get_platform() -> &'static Platform {
    unsafe { PLATFORM.as_ref().expect("PLATFORM not initialized") }
}

const fn get_channel() -> &'static mut zerocopy_channel::Sender<'static, CriticalSectionRawMutex, ChannelPacket> {
    unsafe { EVENT_CHANNEL.as_mut().expect("EVENT_CHANNEL not initialized") }
}

// ============================================================================
// AES-128 ECB (embassy-crypto driver)
// ============================================================================

/// Perform AES-128 ECB encryption using the `embassy-crypto` driver registered
/// for [`Aes128`] (selected by the final binary).
fn aes_ecb_encrypt(key: &[u8; 16], input: &[u8; 16], output: &mut [u8; 16]) {
    let cipher = Aes128::new(key);
    let mut block = *input;
    cipher.encrypt_block(&mut block);
    *output = block;
}

// ============================================================================
// AES-CMAC (RFC 4493), via the `embassy-crypto` CMAC driver
// ============================================================================

/// Stored CMAC key for multi-step CMAC operations
static mut CMAC_KEY: [u8; 16] = [0u8; 16];

// ============================================================================
// P-256 scalar multiplication (embassy-crypto driver)
// ============================================================================

/// Call BLEPLATCB_PkaComplete if a PKA callback was deferred.
/// Must be invoked from the embassy-task context (after seq_resume returns),
/// never from within the sequencer context.
pub fn dispatch_pka_callback() {
    unsafe { super::bindings::ble::BLEPLATCB_PkaComplete() };
}

/// Convert u32 LE word array (index 0 = LSW) to big-endian byte array.
/// This is needed because the BLE stack uses u32 LE words, while `embassy-crypto`
/// uses big-endian byte arrays.
fn words_le_to_be_bytes(words: &[u32; 8], bytes: &mut [u8; 32]) {
    for i in 0..8 {
        let be = words[7 - i].to_be_bytes();
        bytes[i * 4..i * 4 + 4].copy_from_slice(&be);
    }
}

/// Convert big-endian byte array to u32 LE word array (index 0 = LSW).
fn be_bytes_to_words_le(bytes: &[u8], words: &mut [u32; 8]) {
    for i in 0..8 {
        words[7 - i] = u32::from_be_bytes([bytes[i * 4], bytes[i * 4 + 1], bytes[i * 4 + 2], bytes[i * 4 + 3]]);
    }
}

/// Encode a scalar and affine point as BLE-stack u32 LE words.
/// All-zero words signal an error to `BLEPLAT_PkaReadP256Key`/`BLEPLAT_PkaReadDhKey`.
fn p256_point_to_words(p: &p256::Point) -> ([u32; 8], [u32; 8]) {
    let Some(affine) = p.to_affine() else {
        return ([0u32; 8], [0u32; 8]);
    };
    let mut x = [0u32; 8];
    let mut y = [0u32; 8];
    be_bytes_to_words_le(&affine.x, &mut x);
    be_bytes_to_words_le(&affine.y, &mut y);
    (x, y)
}

/// Compute a P-256 operation requested by the BLE stack, using the
/// `embassy-crypto` arithmetic driver registered for [`p256`].
///
/// `k` and the point coordinates are u32 arrays in LE word order (index 0 = LSW).
/// Returns all-zero coordinates on any error (invalid scalar, point off the
/// curve, or the point at infinity).
pub fn p256_compute(req: &P256Request) -> ([u32; 8], [u32; 8]) {
    fn scalar_from_words(words: &[u32; 8]) -> Option<p256::Scalar> {
        let mut be = [0u8; 32];
        words_le_to_be_bytes(words, &mut be);
        p256::Scalar::from_bytes(&be).ok()
    }

    match req {
        P256Request::PublicKey { k } => {
            let Some(scalar) = scalar_from_words(k) else {
                warn!("PKA P-256 key: invalid scalar");
                return ([0u32; 8], [0u32; 8]);
            };
            p256_point_to_words(&p256::Point::mul_base(&scalar))
        }
        P256Request::DhKey { k, peer_x, peer_y } => {
            let (Some(scalar), Some(point)) = (scalar_from_words(k), point_from_words(peer_x, peer_y)) else {
                warn!("PKA P-256 DH: invalid scalar or peer point");
                return ([0u32; 8], [0u32; 8]);
            };
            p256_point_to_words(&point.mul(&scalar))
        }
    }
}

fn point_from_words(x: &[u32; 8], y: &[u32; 8]) -> Option<p256::Point> {
    let mut x_be = [0u8; 32];
    let mut y_be = [0u8; 32];
    words_le_to_be_bytes(x, &mut x_be);
    words_le_to_be_bytes(y, &mut y_be);
    p256::Point::from_xy(&x_be, &y_be).ok()
}

// ============================================================================
// BLE Timer Support using embassy_time
// ============================================================================

/// Maximum number of concurrent BLE stack timers.
/// The BLE stack uses sparse timer IDs (up to 2048+), so we store (id, deadline) pairs.
const MAX_BLE_TIMERS: usize = 32;

/// Timer slots: (timer_id, deadline). id=0xFFFF means slot is free.
const TIMER_SLOT_FREE: u16 = 0xFFFF;
static mut TIMER_SLOTS: [(u16, Instant); MAX_BLE_TIMERS] = [(TIMER_SLOT_FREE, Instant::MAX); MAX_BLE_TIMERS];

/// Get the earliest active timer deadline, if any
pub fn earliest_timer_deadline() -> Instant {
    unsafe {
        TIMER_SLOTS
            .iter()
            .filter(|(id, _)| *id != TIMER_SLOT_FREE)
            .map(|(_, deadline)| *deadline)
            .min()
            .unwrap_or(Instant::MAX)
    }
}

#[cfg(feature = "ble-stack-llo")]
pub fn check_expired_timers() {}

#[cfg(not(feature = "ble-stack-llo"))]
/// Check and fire any expired timers. Called from the runner loop.
/// Calls BLEPLATCB_TimerExpiry(id) for each expired timer to notify the BLE stack.
pub fn check_expired_timers() {
    let now = Instant::now();
    let mut expired = false;
    let mut timer_id: u16;
    unsafe {
        for (id, deadline) in TIMER_SLOTS
            .iter_mut()
            .filter(|(id, deadline)| *id != TIMER_SLOT_FREE && now >= *deadline)
        {
            timer_id = *id;
            *id = TIMER_SLOT_FREE;
            *deadline = Instant::MAX;
            expired = true;

            BLEPLATCB_TimerExpiry(timer_id);
        }
    }

    if expired {
        super::util_seq::seq_pend();
    }
}

// ============================================================================
// NVM (Non-Volatile Memory) Storage using Internal Flash
// ============================================================================

// Flash parameters for STM32WBA
// WRITE_SIZE = 16 bytes (quad-word), ERASE_SIZE = 8KB per page
const NVM_WRITE_SIZE: usize = 16;
const NVM_PAGE_SIZE: usize = 8192; // 8KB

// We use the last page of flash for NVM storage.
// The application must ensure this page is not used for code.
// STM32WBA65 has 2MB flash at 0x0800_0000, so last page starts at 0x081F_E000.
// For smaller variants, this would be different.
// We use a configurable base address that defaults to the last 8KB.
//
// Layout within the NVM page:
//   [0..4]   : magic marker (0x424C_454E = "BLEN")
//   [4..8]   : data length in bytes
//   [16..]   : actual NVM data (aligned to 16-byte quad-word boundary)
const NVM_MAGIC: u32 = 0x424C_454E; // "BLEN"

/// NVM base address - set by the application before BLE init.
/// Defaults to 0 (disabled). Must be set to a valid flash page address.
static NVM_BASE_ADDRESS: AtomicU32 = AtomicU32::new(0);

/// Set the NVM base address. Must be called before BLE init.
/// The address must be page-aligned (8KB boundary) and within flash.
/// For STM32WBA65RI (2MB flash): use 0x081F_E000 (last 8KB page of BANK_2).
pub fn set_nvm_base_address(addr: u32) {
    NVM_BASE_ADDRESS.store(addr, Ordering::Release);
}

/// Erase the bond NVM flash page previously configured with [`set_nvm_base_address`].
///
/// Must be called **after** [`set_nvm_base_address`] and **before** BLE stack init
/// (`new_platform!`). Reflashing the application does not erase this page; use this
/// when clearing stale bonds. `aci_gap_clear_security_db` only clears RAM — the next
/// boot would reload bonds from flash without this erase.
pub fn erase_bond_nvm_flash() -> bool {
    let base = NVM_BASE_ADDRESS.load(Ordering::Acquire);
    if base == 0 {
        return false;
    }
    unsafe { flash_erase_page(base) }
}

/// Pointer to the NVM cache buffer allocated by the BLE stack init.
static NVM_CACHE_PTR: AtomicUsize = AtomicUsize::new(0);
/// Size of the NVM cache buffer in bytes.
static NVM_CACHE_LEN_BYTES: AtomicUsize = AtomicUsize::new(0);

/// Register the NVM cache buffer. Called once during `init_ble_stack` so that
/// `BLEPLAT_NvmStore` can write the entire buffer (matching ST reference behaviour).
pub fn register_nvm_cache(ptr: *mut u64, len_u64s: usize) {
    NVM_CACHE_PTR.store(ptr as usize, Ordering::Release);
    NVM_CACHE_LEN_BYTES.store(len_u64s * 8, Ordering::Release);
}

/// Load previously stored NVM data from flash into `buf`.
///
/// Call this before `BleStack_Init` so the stack can restore any bonds that
/// were persisted in a previous session. Returns the number of valid bytes
/// loaded into `buf`, or 0 if flash has no valid NVM data (first boot after
/// flash erase, or NVM base address not configured).
pub fn load_nvm_from_flash(buf: &mut [u8]) -> usize {
    let base = NVM_BASE_ADDRESS.load(Ordering::Acquire);
    if base == 0 {
        return 0;
    }

    // Read and verify magic marker
    let magic = unsafe { core::ptr::read_volatile(base as *const u32) };
    if magic != NVM_MAGIC {
        return 0;
    }

    // Read stored data length
    let size = unsafe { core::ptr::read_volatile((base + 4) as *const u16) } as usize;
    if size == 0 || size > buf.len() {
        return 0;
    }

    // Copy data payload (starts at offset NVM_WRITE_SIZE = 16)
    let data_ptr = (base + NVM_WRITE_SIZE as u32) as *const u8;
    let data = unsafe { core::slice::from_raw_parts(data_ptr, size) };
    buf[..size].copy_from_slice(data);

    trace!("load_nvm_from_flash: loaded {} bytes", size);
    size
}

/// Unlock flash for programming
unsafe fn flash_unlock() {
    if FLASH.nscr().read().lock() {
        FLASH.nskeyr().write_value(0x4567_0123);
        FLASH.nskeyr().write_value(0xCDEF_89AB);
    }
}

/// Lock flash after programming
unsafe fn flash_lock() {
    FLASH.nscr().modify(|w| w.set_lock(true));
}

/// Wait for flash operation to complete
unsafe fn flash_wait_ready() -> bool {
    loop {
        let sr = FLASH.nssr().read();
        if !sr.wdw() && !sr.bsy() {
            // Check for errors
            if sr.pgserr() || sr.sizerr() || sr.pgaerr() || sr.wrperr() || sr.progerr() || sr.operr() {
                // Clear errors
                FLASH.nssr().modify(|w| {
                    w.set_eop(true);
                    w.set_operr(true);
                    w.set_progerr(true);
                    w.set_wrperr(true);
                    w.set_pgaerr(true);
                    w.set_sizerr(true);
                    w.set_pgserr(true);
                    w.set_optwerr(true);
                });
                return false;
            }
            return true;
        }
    }
}

/// Erase a flash page by its base address
unsafe fn flash_erase_page(page_addr: u32) -> bool {
    const FLASH_BASE: u32 = 0x0800_0000;
    const BANK2_BASE: u32 = 0x0810_0000; // BANK_2 starts at +1MB

    let (bank2, page_index) = if page_addr >= BANK2_BASE {
        (true, (page_addr - BANK2_BASE) / NVM_PAGE_SIZE as u32)
    } else {
        (false, (page_addr - FLASH_BASE) / NVM_PAGE_SIZE as u32)
    };

    flash_unlock();

    FLASH.nscr().modify(|w| {
        w.set_per(true);
        w.set_pnb(page_index as u8);
        w.set_bker(bank2);
    });
    FLASH.nscr().modify(|w| {
        w.set_strt(true);
    });

    let ok = flash_wait_ready();

    FLASH.nscr().modify(|w| w.set_per(false));
    flash_lock();

    ok
}

/// Write a 16-byte quad-word to flash
unsafe fn flash_write_quadword(addr: u32, data: &[u8; 16]) -> bool {
    flash_unlock();

    // Wait for any previous operation
    flash_wait_ready();

    // Enable programming
    FLASH.nscr().write(|w| w.set_pg(true));

    // Write 4 x u32 words
    for i in 0..4 {
        let word = u32::from_le_bytes([data[i * 4], data[i * 4 + 1], data[i * 4 + 2], data[i * 4 + 3]]);
        core::ptr::write_volatile((addr + (i as u32) * 4) as *mut u32, word);
        core::sync::atomic::fence(Ordering::SeqCst);
    }

    let ok = flash_wait_ready();

    // Disable programming
    FLASH.nscr().write(|w| w.set_pg(false));
    flash_lock();

    ok
}

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
    compiler_fence(Ordering::SeqCst);
}

unsafe fn nvic_disable(irq: u32) {
    NVIC::mask(RawInterrupt::new(irq));
    compiler_fence(Ordering::SeqCst);
}

unsafe fn nvic_set_pending(irq: u32) {
    NVIC::pend(RawInterrupt::new(irq));
    compiler_fence(Ordering::SeqCst);
}

unsafe fn nvic_get_active(irq: u32) -> bool {
    NVIC::is_active(RawInterrupt::new(irq))
}

unsafe fn nvic_set_priority(irq: u32, priority: u8) {
    // STM32WBA is ARMv8-M, which uses byte-accessible IPR registers
    let nvic = &*NVIC::PTR;
    nvic.ipr[irq as usize].write(priority);

    compiler_fence(Ordering::SeqCst);
}

/// Set BASEPRI to at least `value` (ARM BASEPRI_MAX semantics).
/// Lower numeric value = higher priority = more restrictive mask.
/// Only writes if `value` would make the mask MORE restrictive
/// (i.e., block more interrupts) than the current BASEPRI.
fn set_basepri_max(value: u8) {
    unsafe {
        let current = basepri::read();
        // BASEPRI=0 means "no masking". Any non-zero value is more restrictive.
        // Among non-zero values, a lower value masks more interrupts.
        if value != 0 && (current == 0 || value < current) {
            basepri::write(value);
        }
    }
}

pub(crate) unsafe fn run_radio_high_isr() {
    trace!("RADIO ISR: callback={:?}", load_callback(&RADIO_CALLBACK).is_some());
    if let Some(cb) = load_callback(&RADIO_CALLBACK) {
        cb();
    }
    // Wake the BLE runner task to process any resulting events
    util_seq::seq_pend();
}

pub(crate) unsafe fn run_radio_sw_low_isr() {
    trace!(
        "HASH ISR (sw low): callback={:?}",
        load_callback(&LOW_ISR_CALLBACK).is_some()
    );
    if let Some(cb) = load_callback(&LOW_ISR_CALLBACK) {
        cb();
    }

    if RADIO_SW_LOW_ISR_RUNNING_HIGH_PRIO.swap(false, Ordering::AcqRel) {
        nvic_set_priority(RADIO_SW_LOW_INTR_NUM, pack_priority(mac::RADIO_SW_LOW_INTR_PRIO));
    }

    // Wake the BLE runner task to process any resulting events
    util_seq::seq_pend();
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
    RCC.ahb5enr().modify(|w| w.set_radioen(true));

    // Memory barrier to ensure clock is enabled before proceeding
    compiler_fence(Ordering::SeqCst);

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
// Some of these sanity checks fire on production silicon — e.g. an IP-version
// probe inside the PHY init path can return the silicon-default response on
// early WBA5x cuts, which doesn't match the value the type-7 PHY code
// expects. ST's reference HAL project templates define `assert_failed()` with
// an empty body, so the link-layer archive is designed to continue past a
// failed assert.
//
// We mirror that here: log the failed assert (with the return address so it
// can be looked up in the archive's disassembly) but do not halt.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_Assert(condition: u8) {
    if condition == 0 {
        let lr: u32;
        core::arch::asm!("mov {}, lr", out(reg) lr);
        warn!("LINKLAYER_PLAT_Assert(0) at LR=0x{:08x}", lr);
    }
}

// /**
//   * @brief  Enable/disable the Link Layer active clock (baseband clock).
//   * @param  enable: boolean value to enable (1) or disable (0) the clock.
//   * @retval None
//   */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LINKLAYER_PLAT_WaitHclkRdy() {
    // trace!("LINKLAYER_PLAT_WaitHclkRdy"); // Too frequent, disabled
    if AHB5_SWITCHED_OFF.swap(false, Ordering::AcqRel) {
        let reference = RADIO_SLEEP_TIMER_VAL.load(Ordering::Acquire);
        trace!("LINKLAYER_PLAT_WaitHclkRdy: reference={}", reference);
        while reference == link_layer::ll_intf_cmn_get_slptmr_value() {}
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
    // Check if radio state will allow the AHB5 clock to be cut.
    // AHB5 clock will be cut in the following cases:
    //  - 2.4GHz radio is NOT in ACTIVE mode (in SLEEP or DEEPSLEEP), OR
    //  - Both RADIOSMEN and STRADIOCLKON bits are at 0.
    //
    // Radio mode: 0x0=DeepSleep, 0x1=Sleep, 0x2/0x3=Active (1x pattern)
    let radio_mode = PWR.radioscr().read().mode().to_bits();
    let radio_active = radio_mode >= 2; // 1x = active mode

    let radiosmen = RCC.ahb5smenr().read().radiosmen();
    let stradioclkon = RCC.radioenr().read().stradioclkon();

    if !radio_active || (!radiosmen && !stradioclkon) {
        AHB5_SWITCHED_OFF.store(true, Ordering::Release);
    }
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
        while !RCC.cr().read().hserdy() {}

        // Enable RADIO baseband clock (active clock)
        RCC.radioenr().modify(|w| w.set_bbclken(true));

        // Memory barrier to ensure clock is enabled before proceeding
        compiler_fence(Ordering::SeqCst);

        trace!("LINKLAYER_PLAT_AclkCtrl: radio baseband clock enabled");
    } else {
        // Disable RADIO baseband clock (active clock)
        RCC.radioenr().modify(|w| w.set_bbclken(false));

        // Memory barrier
        compiler_fence(Ordering::SeqCst);

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

    // Blocking driver registered by the final binary (e.g. the
    // `embassy-crypto-rng` feature of `embassy-stm32`).
    embassy_crypto::rng_fill_bytes(core::slice::from_raw_parts_mut(ptr_rnd, len as usize));

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
    // trace!("LINKLAYER_PLAT_EnableIRQ"); // Too frequent, disabled
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
    // trace!("LINKLAYER_PLAT_DisableIRQ"); // Too frequent, disabled
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

    // Enable radio bus clock during Sleep/Stop modes (RADIOSMEN bit)
    // This keeps the radio clock alive while the radio is active
    RCC.ahb5smenr().modify(|w| w.set_radiosmen(true));

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

    // Disable radio bus clock during Sleep/Stop modes (RADIOSMEN bit)
    // Radio is no longer active, so the clock can be gated
    RCC.ahb5smenr().modify(|w| w.set_radiosmen(false));

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
    // AES and P-256 operations go through `embassy-crypto`'s link-time drivers,
    // which take care of their own peripheral setup. Nothing to do here.
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

    embassy_crypto::rng_fill_bytes(core::slice::from_raw_parts_mut(val as *mut u8, n as usize * 4));
}

/// AES ECB encrypt function using the `embassy-crypto` driver registered
/// for [`Aes128`].
///
/// Used by the BLE stack for random address hash calculation and
/// other cryptographic operations.
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

    let key_slice: &[u8; 16] = &*(key as *const [u8; 16]);
    let input_slice: &[u8; 16] = &*(input as *const [u8; 16]);
    let output_slice: &mut [u8; 16] = &mut *(output as *mut [u8; 16]);

    aes_ecb_encrypt(key_slice, input_slice, output_slice);
}

/// AES CMAC set key function.
///
/// Stores the key for subsequent CMAC compute operations.
///
/// # Arguments
/// * `key` - 16-byte AES key for CMAC
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_AesCmacSetKey(key: *const u8) {
    trace!("BLEPLAT_AesCmacSetKey");

    if key.is_null() {
        error!("BLEPLAT_AesCmacSetKey: null key");
        return;
    }

    core::ptr::copy_nonoverlapping(key, CMAC_KEY.as_mut_ptr(), 16);
}

/// AES CMAC compute function (RFC 4493).
///
/// Computes a 16-byte CMAC tag over the input data using the key
/// previously set by BLEPLAT_AesCmacSetKey, via the `embassy-crypto`
/// CMAC driver registered for [`Aes128Cmac`].
///
/// # Arguments
/// * `input` - Input data
/// * `input_length` - Length of input data in bytes
/// * `output_tag` - 16-byte output CMAC tag
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_AesCmacCompute(input: *const u8, input_length: u32, output_tag: *mut u8) {
    trace!("BLEPLAT_AesCmacCompute: length={}", input_length);

    if output_tag.is_null() {
        error!("BLEPLAT_AesCmacCompute: null output");
        return;
    }

    let input_slice = if input.is_null() || input_length == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(input, input_length as usize)
    };

    let output_slice: &mut [u8; 16] = &mut *(output_tag as *mut [u8; 16]);

    *output_slice = Aes128Cmac::mac(&CMAC_KEY, input_slice);
}

// ============================================================================
// AES-CCM (RFC 3610 / NIST SP 800-38C), via the `embassy-crypto` CCM driver
// ============================================================================

/// AES-CCM encryption/decryption for the BLE stack, via the `embassy-crypto`
/// driver registered for [`Aes128Ccm`].
///
/// # Arguments
/// * `mode` - 0 for encryption, 1 for decryption
/// * `key` - 16-byte AES key
/// * `iv_length` - IV length in bytes (7-13)
/// * `iv` - IV data
/// * `add_length` - Additional Authenticated Data length
/// * `add` - AAD data
/// * `input_length` - Input data length
/// * `input` - Data to encrypt/decrypt
/// * `tag_length` - CCM tag length (4, 6, 8, 10, 12, 14 or 16 bytes)
/// * `tag` - CCM tag (written on encrypt, verified on decrypt)
/// * `output` - Result data
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_AesCcmCrypt(
    mode: u8,
    key: *const u8,
    iv_length: u8,
    iv: *const u8,
    add_length: u16,
    add: *const u8,
    input_length: u32,
    input: *const u8,
    tag_length: u8,
    tag: *mut u8,
    output: *mut u8,
) -> core::ffi::c_int {
    trace!(
        "BLEPLAT_AesCcmCrypt: mode={}, iv_len={}, add_len={}, input_len={}, tag_len={}",
        mode, iv_length, add_length, input_length, tag_length
    );

    const BLEPLAT_ERROR: core::ffi::c_int = -5;

    if key.is_null() || iv.is_null() || tag.is_null() || output.is_null() {
        error!("BLEPLAT_AesCcmCrypt: null pointer");
        return BLEPLAT_ERROR;
    }
    if input.is_null() && input_length > 0 {
        error!("BLEPLAT_AesCcmCrypt: null input with non-zero length");
        return BLEPLAT_ERROR;
    }

    let key_slice: &[u8; 16] = &*(key as *const [u8; 16]);
    let iv_slice = core::slice::from_raw_parts(iv, iv_length as usize);
    let aad_slice = if add.is_null() || add_length == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(add, add_length as usize)
    };
    let input_slice = if input_length == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(input, input_length as usize)
    };
    let tag_slice = core::slice::from_raw_parts_mut(tag, tag_length as usize);
    let output_slice = core::slice::from_raw_parts_mut(output, input_length as usize);

    let cipher = Aes128Ccm::new(key_slice);

    let result = if mode == 0 {
        output_slice.copy_from_slice(input_slice);
        cipher.encrypt(iv_slice, aad_slice, output_slice, tag_slice)
    } else {
        output_slice.copy_from_slice(input_slice);
        cipher.decrypt(iv_slice, aad_slice, output_slice, tag_slice)
    };

    match result {
        Ok(()) => 0, // BLEPLAT_OK
        // Includes `Error::InvalidSignature` on tag mismatch when decrypting.
        Err(e) => {
            warn!("BLEPLAT_AesCcmCrypt failed: {:?}", e);
            BLEPLAT_ERROR
        }
    }
}

/// Start a BLE stack timer using embassy_time.
///
/// Sets a deadline for the specified timer ID. The BLE runner checks
/// these deadlines and wakes the sequencer when they expire.
///
/// # Arguments
/// * `id` - Timer ID (0-based, max MAX_BLE_TIMERS-1)
/// * `timeout` - Timeout in milliseconds
///
/// # Returns
/// 0 on success, 1 on error (invalid ID)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_TimerStart(id: u16, timeout: u32) -> u8 {
    trace!("BLEPLAT_TimerStart: id={}, timeout={}ms", id, timeout);

    let deadline = Instant::now() + Duration::from_millis(timeout as u64);

    // Find existing slot for this ID, or a free slot
    let mut free_slot: Option<usize> = None;
    for (i, (slot_id, slot_deadline)) in TIMER_SLOTS.iter_mut().enumerate() {
        if *slot_id == id {
            // Update existing timer
            *slot_deadline = deadline;
            super::util_seq::seq_pend();
            return 0;
        }

        if *slot_id == TIMER_SLOT_FREE && free_slot.is_none() {
            free_slot = Some(i);
        }
    }

    // Use a free slot
    if let Some(i) = free_slot {
        TIMER_SLOTS[i] = (id, deadline);
        super::util_seq::seq_pend();
        0
    } else {
        warn!("BLEPLAT_TimerStart: no free timer slots for id {}", id);
        1
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_TimerStop(id: u16) {
    trace!("BLEPLAT_TimerStop: id={}", id);

    for slot in TIMER_SLOTS.iter_mut() {
        if slot.0 == id {
            slot.0 = TIMER_SLOT_FREE;
            slot.1 = Instant::MAX;
            return;
        }
    }
}

/// NVM store function for BLE stack.
///
/// The ST BLE stack ignores the `ptr`/`size` arguments in its reference implementation
/// and always writes the entire NVM cache buffer. We do the same: `ptr` and `size` are
/// ignored; the buffer registered via `register_nvm_cache` is written to flash wholesale.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_NvmStore(_ptr: *const u64, _size: u16) {
    let base = NVM_BASE_ADDRESS.load(Ordering::Acquire);
    if base == 0 {
        trace!("BLEPLAT_NvmStore: NVM not configured, skipping");
        return;
    }

    let cache_ptr = NVM_CACHE_PTR.load(Ordering::Acquire);
    let cache_len = NVM_CACHE_LEN_BYTES.load(Ordering::Acquire);
    if cache_ptr == 0 || cache_len == 0 {
        error!("BLEPLAT_NvmStore: NVM cache not registered");
        return;
    }

    trace!("BLEPLAT_NvmStore: writing {} bytes", cache_len);

    let data = core::slice::from_raw_parts(cache_ptr as *const u8, cache_len);

    if !flash_erase_page(base) {
        error!("BLEPLAT_NvmStore: flash erase failed");
        return;
    }

    // Write header: magic + length in bytes (fits in first quad-word with padding)
    let mut header = [0u8; NVM_WRITE_SIZE];
    header[0..4].copy_from_slice(&NVM_MAGIC.to_le_bytes());
    header[4..6].copy_from_slice(&(cache_len as u16).to_le_bytes());

    if !flash_write_quadword(base, &header) {
        error!("BLEPLAT_NvmStore: flash write header failed");
        return;
    }

    // Write data starting at offset 16 (second quad-word)
    let data_addr = base + NVM_WRITE_SIZE as u32;
    let mut offset: usize = 0;
    while offset < data.len() {
        let mut quad = [0u8; NVM_WRITE_SIZE];
        let chunk = (data.len() - offset).min(NVM_WRITE_SIZE);
        quad[..chunk].copy_from_slice(&data[offset..offset + chunk]);

        if !flash_write_quadword(data_addr + offset as u32, &quad) {
            error!("BLEPLAT_NvmStore: flash write data failed at offset {}", offset);
            return;
        }
        offset += NVM_WRITE_SIZE;
    }

    trace!("BLEPLAT_NvmStore: stored {} bytes", cache_len);
}

// BLEPLAT return codes
const BLEPLAT_OK: i32 = 0;

/// Start P-256 public key generation using hardware PKA.
///
/// Computes public_key = private_key * G (generator point).
/// The BLE stack provides the private key as 8 x u32 in LE word order.
/// Result is cached and retrieved via BLEPLAT_PkaReadP256Key.
///
/// # Arguments
/// * `local_private_key` - Pointer to 8 x u32 array (256-bit private key, LE word order)
///
/// # Returns
/// 0 on success, negative on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_PkaStartP256Key(local_private_key: *const u32) -> i32 {
    trace!("BLEPLAT_PkaStartP256Key");

    if local_private_key.is_null() {
        error!("BLEPLAT_PkaStartP256Key: null private key");
        return -1;
    }

    let k: &[u32; 8] = &*(local_private_key as *const [u32; 8]);

    if get_platform().get_p256_req().signaled() {
        error!("BLEPLAT_PkaStartP256Key: signal not empty");
        return -1;
    }

    get_platform().get_p256_resp().reset();
    get_platform().get_p256_req().signal(P256Request::PublicKey { k: *k });

    BLEPLAT_OK
}

/// Read result of P-256 public key generation.
///
/// Returns the public key computed by BLEPLAT_PkaStartP256Key.
/// The output is 16 x u32: [X0..X7, Y0..Y7] in LE word order.
///
/// # Arguments
/// * `local_public_key` - Pointer to 16 x u32 array for output
///
/// # Returns
/// 0 on success, negative on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_PkaReadP256Key(local_public_key: *mut u32) -> i32 {
    trace!("BLEPLAT_PkaReadP256Key");

    if local_public_key.is_null() {
        error!("BLEPLAT_PkaReadP256Key: null output");
        return -1;
    }

    let Some((pka_x, pka_y)) = get_platform().get_p256_resp().try_take() else {
        warn!("BLEPLAT_PkaReadP256Key: result not ready");
        return -1;
    };

    if pka_x == [0u32; 8] && pka_y == [0u32; 8] {
        warn!("BLEPLAT_PkaReadP256Key: invalid result");
        return -1;
    }

    let out = core::slice::from_raw_parts_mut(local_public_key, 16);
    out[0..8].copy_from_slice(&pka_x);
    out[8..16].copy_from_slice(&pka_y);

    BLEPLAT_OK
}

/// Start DH key computation using hardware PKA.
///
/// Computes shared_secret = private_key * remote_public_key (ECC scalar multiplication).
/// Result is cached and retrieved via BLEPLAT_PkaReadDhKey.
///
/// # Arguments
/// * `local_private_key` - Pointer to 8 x u32 array (256-bit private key, LE word order)
/// * `remote_public_key` - Pointer to 16 x u32 array [X0..X7, Y0..Y7] (LE word order)
///
/// # Returns
/// 0 on success, negative on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_PkaStartDhKey(local_private_key: *const u32, remote_public_key: *const u32) -> i32 {
    trace!("BLEPLAT_PkaStartDhKey");

    if local_private_key.is_null() || remote_public_key.is_null() {
        error!("BLEPLAT_PkaStartDhKey: null pointer");
        return -1;
    }

    let k: &[u32; 8] = &*(local_private_key as *const [u32; 8]);
    let remote = core::slice::from_raw_parts(remote_public_key, 16);

    let mut px = [0u32; 8];
    let mut py = [0u32; 8];
    px.copy_from_slice(&remote[0..8]);
    py.copy_from_slice(&remote[8..16]);

    if get_platform().get_p256_req().signaled() {
        error!("BLEPLAT_PkaStartDhKey: signal not empty");
        return -1;
    }

    get_platform().get_p256_resp().reset();
    get_platform().get_p256_req().signal(P256Request::DhKey {
        k: *k,
        peer_x: px,
        peer_y: py,
    });

    BLEPLAT_OK
}

/// Read result of DH key computation.
///
/// Returns the X coordinate of the shared secret computed by BLEPLAT_PkaStartDhKey.
/// The output is 8 x u32 (256-bit X coordinate, LE word order).
///
/// # Arguments
/// * `dh_key` - Pointer to 8 x u32 array for output (X coordinate only)
///
/// # Returns
/// 0 on success, negative on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_PkaReadDhKey(dh_key: *mut u32) -> i32 {
    trace!("BLEPLAT_PkaReadDhKey");

    if dh_key.is_null() {
        error!("BLEPLAT_PkaReadDhKey: null output");
        return -1;
    }

    let Some((pka_x, pka_y)) = get_platform().get_p256_resp().try_take() else {
        warn!("BLEPLAT_PkaReadDhKey: result not ready");
        return -1;
    };

    if pka_x == [0u32; 8] && pka_y == [0u32; 8] {
        warn!("BLEPLAT_PkaReadDhKey: invalid result");
        return -1;
    }

    // DH key is just the X coordinate of the shared point
    let out = core::slice::from_raw_parts_mut(dh_key, 8);
    out.copy_from_slice(&pka_x);

    BLEPLAT_OK
}

/// BLE stack HCI event indication callback
/// This is called by the BLE stack when HCI events arrive
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLECB_Indication(data: *const u8, length: u16, ext_data: *const u8, ext_length: u16) -> u8 {
    if data.is_null() || length == 0 {
        return 1; // Error
    }

    // Convert to slice
    let event_data = core::slice::from_raw_parts(data, length as usize);
    let ext_data: &[u8] = if ext_data.is_null() || ext_length == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(ext_data, ext_length as usize)
    };

    trace!(
        "BLECB_Indication: event_code=0x{:02X}, length={}",
        event_data[0], length
    );

    // HCI event packet format:
    // Byte 0: 0x04 (HCI Event packet indicator)
    // Byte 1: Event code (0x05=Disconnect, 0x3E=LE Meta, 0xFF=Vendor)
    // Byte 2: Parameter total length
    // Byte 3+: Event parameters

    if event_data.len() > 0 && event_data[0] == 4 {
        let evt_code = if length >= 2 { event_data[1] } else { event_data[0] };

        if evt_code == 0x05 {
            let status = if length >= 4 { event_data[3] } else { 0 };
            let handle = if length >= 6 {
                u16::from_le_bytes([event_data[4], event_data[5]])
            } else {
                0
            };
            let reason = if length >= 7 { event_data[6] } else { 0 };
            debug!(
                "HCI Event: Disconnection Complete (status=0x{:02X}, handle=0x{:04X}, reason=0x{:02X})",
                status, handle, reason
            );
        } else if evt_code == 0x3E {
            let sub_code = if length >= 4 { event_data[3] } else { 0 };
            debug!("HCI Event: LE Meta (sub=0x{:02X}, len={})", sub_code, length);
        } else {
            debug!("HCI Event: code=0x{:02X}, len={}", evt_code, length);
        }
    } else {
        debug!("Other Event: {:x}", event_data[..10.min(event_data.len())]);
    }

    debug!("Raw Event: {:x} {:x}", event_data, ext_data);

    let Some(mut slot) = get_channel().try_send() else {
        return 0;
    };

    slot.copy_from(event_data, ext_data);
    slot.send_done();

    util_seq::UTIL_SEQ_SetTask(TASK_BLE_HOST_MASK, TASK_PRIO_BLE_HOST);

    0 // Success
}
