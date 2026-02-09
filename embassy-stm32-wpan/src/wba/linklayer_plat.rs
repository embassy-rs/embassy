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
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU32, Ordering, compiler_fence};

use cortex_m::interrupt::InterruptNumber;
use cortex_m::peripheral::NVIC;
use cortex_m::register::basepri;
use critical_section;
#[cfg(feature = "defmt")]
use defmt::{error, trace, warn};
use embassy_sync::blocking_mutex::Mutex;
#[cfg(not(feature = "defmt"))]
macro_rules! trace {
    ($($arg:tt)*) => {{}};
}
#[cfg(not(feature = "defmt"))]
macro_rules! error {
    ($($arg:tt)*) => {{}};
}
#[cfg(not(feature = "defmt"))]
macro_rules! warn {
    ($($arg:tt)*) => {{}};
}
use embassy_stm32::NVIC_PRIO_BITS;
use embassy_stm32::pac::{FLASH, RCC};
use embassy_stm32::peripherals::RNG;
use embassy_stm32::rng::Rng;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Instant, block_for};

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

// ============================================================================
// AES-128 ECB Hardware Acceleration (PAC-level)
// ============================================================================

/// Perform AES-128 ECB encryption using the hardware AES peripheral.
///
/// Uses PAC registers directly since this is called from extern "C" callbacks.
fn aes_ecb_encrypt(key: &[u8; 16], input: &[u8; 16], output: &mut [u8; 16]) {
    let aes = embassy_stm32::pac::AES;

    // Disable AES peripheral
    aes.cr().modify(|w| w.set_en(false));

    // Configure: ECB mode (CHMOD=0), encrypt (MODE=0), 128-bit key, NO_SWAP datatype
    aes.cr().write(|w| {
        w.set_chmod(embassy_stm32::pac::aes::vals::Chmod::from_bits(0)); // ECB
        w.set_mode(embassy_stm32::pac::aes::vals::Mode::from_bits(0)); // Encrypt
        w.set_keysize(false); // 128-bit
        w.set_datatype(embassy_stm32::pac::aes::vals::Datatype::from_bits(0)); // NO_SWAP
    });

    // Load key: big-endian words in reverse register order (keyr(3)=first word, keyr(0)=last word)
    for i in 0..4 {
        let word = u32::from_be_bytes([key[i * 4], key[i * 4 + 1], key[i * 4 + 2], key[i * 4 + 3]]);
        aes.keyr(3 - i).write_value(word);
    }

    // Enable AES peripheral
    aes.cr().modify(|w| w.set_en(true));

    // Write 4 input words via DINR (big-endian conversion)
    for i in 0..4 {
        let word = u32::from_be_bytes([input[i * 4], input[i * 4 + 1], input[i * 4 + 2], input[i * 4 + 3]]);
        aes.dinr().write_value(word);
    }

    // Wait for Computation Complete Flag (CCF)
    while !aes.sr().read().ccf() {}

    // Read 4 output words from DOUTR (big-endian conversion)
    for i in 0..4 {
        let word = aes.doutr().read();
        output[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }

    // Clear CCF flag
    aes.icr().write(|w| w.0 = 0xFFFF_FFFF);

    // Disable AES peripheral
    aes.cr().modify(|w| w.set_en(false));
}

// ============================================================================
// AES-CMAC (RFC 4493) Implementation
// ============================================================================

/// Stored CMAC key for multi-step CMAC operations
static mut CMAC_KEY: [u8; 16] = [0u8; 16];

/// Left-shift a 16-byte block by 1 bit and conditionally XOR with Rb (0x87)
fn cmac_shift_and_xor(input: &[u8; 16]) -> [u8; 16] {
    let mut output = [0u8; 16];
    let mut carry: u8 = 0;
    for i in (0..16).rev() {
        output[i] = (input[i] << 1) | carry;
        carry = input[i] >> 7;
    }
    // If MSB of input was set, XOR last byte with 0x87 (Rb constant for AES-128)
    if input[0] & 0x80 != 0 {
        output[15] ^= 0x87;
    }
    output
}

/// Generate CMAC subkeys K1 and K2 from the cipher key
fn cmac_generate_subkeys(key: &[u8; 16]) -> ([u8; 16], [u8; 16]) {
    let zero_block = [0u8; 16];
    let mut l = [0u8; 16];
    aes_ecb_encrypt(key, &zero_block, &mut l);

    let k1 = cmac_shift_and_xor(&l);
    let k2 = cmac_shift_and_xor(&k1);

    (k1, k2)
}

/// Compute AES-CMAC tag per RFC 4493
fn cmac_compute(key: &[u8; 16], input: &[u8], output: &mut [u8; 16]) {
    let (k1, k2) = cmac_generate_subkeys(key);

    let n = input.len();
    let n_blocks = if n == 0 { 1 } else { (n + 15) / 16 };
    let complete = n != 0 && (n % 16 == 0);

    // Prepare the last block
    let mut last_block = [0u8; 16];
    if complete {
        // Complete block: XOR with K1
        let start = (n_blocks - 1) * 16;
        for i in 0..16 {
            last_block[i] = input[start + i] ^ k1[i];
        }
    } else {
        // Incomplete block: pad with 10...0, XOR with K2
        let start = (n_blocks - 1) * 16;
        let remaining = n - start;
        for i in 0..remaining {
            last_block[i] = input[start + i];
        }
        last_block[remaining] = 0x80; // padding bit
        // rest is already 0
        for i in 0..16 {
            last_block[i] ^= k2[i];
        }
    }

    // CBC-MAC chain: X starts as zero, then X = AES(K, X ^ M_i)
    let mut x = [0u8; 16];
    for i in 0..n_blocks - 1 {
        let start = i * 16;
        let mut y = [0u8; 16];
        for j in 0..16 {
            y[j] = x[j] ^ input[start + j];
        }
        aes_ecb_encrypt(key, &y, &mut x);
    }

    // Final block
    let mut y = [0u8; 16];
    for j in 0..16 {
        y[j] = x[j] ^ last_block[j];
    }
    aes_ecb_encrypt(key, &y, output);
}

// ============================================================================
// PKA P-256 Hardware Acceleration (PAC-level)
// ============================================================================

/// PKA ECC scalar multiplication RAM offsets (from embassy-stm32 pka driver)
mod pka_offsets {
    pub const IN_EXP_NB_BITS: usize = 0x00;
    pub const IN_OP_NB_BITS: usize = 0x08;
    pub const IN_A_COEFF_SIGN: usize = 0x10;
    pub const IN_A_COEFF: usize = 0x18;
    pub const IN_B_COEFF: usize = 0x120;
    pub const IN_MOD_GF: usize = 0xC88;
    pub const IN_K: usize = 0xEA0;
    pub const IN_INITIAL_POINT_X: usize = 0x178;
    pub const IN_INITIAL_POINT_Y: usize = 0x70;
    pub const IN_N_PRIME_ORDER: usize = 0xB88;
    pub const OUT_RESULT_X: usize = 0x178;
    pub const OUT_RESULT_Y: usize = 0x1D0;
    pub const OUT_ERROR: usize = 0x280;
}

/// P-256 curve parameters as u32 words (little-endian word order, index 0 = LSW)
/// This matches the format the BLE stack uses for its u32 arrays.
mod p256 {
    /// Prime p = FFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFF
    pub const P: [u32; 8] = [
        0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF, 0x0000_0000,
        0x0000_0000, 0x0000_0000, 0x0000_0001, 0xFFFF_FFFF,
    ];

    /// |a| = 3 (a = -3 mod p, sign stored separately)
    pub const A: [u32; 8] = [
        0x0000_0003, 0x0000_0000, 0x0000_0000, 0x0000_0000,
        0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000,
    ];

    /// b coefficient
    pub const B: [u32; 8] = [
        0x27D2_604B, 0x3BCE_3C3E, 0xCC53_B0F6, 0x651D_06B0,
        0x7698_86BC, 0xB3EB_BD55, 0xAA3A_93E7, 0x5AC6_35D8,
    ];

    /// Generator point Gx
    pub const GX: [u32; 8] = [
        0xD898_C296, 0xF4A1_3945, 0x2DEB_33A0, 0x7703_7D81,
        0x63A4_40F2, 0xF8BC_E6E5, 0xE12C_4247, 0x6B17_D1F2,
    ];

    /// Generator point Gy
    pub const GY: [u32; 8] = [
        0x37BF_51F5, 0xCBB6_4068, 0x6B31_5ECE, 0x2BCE_3357,
        0x7C0F_9E16, 0x8EE7_EB4A, 0xFE1A_7F9B, 0x4FE3_42E2,
    ];

    /// Order n
    pub const N: [u32; 8] = [
        0xFC63_2551, 0xF3B9_CAC2, 0xA717_9E84, 0xBCE6_FAAD,
        0xFFFF_FFFF, 0xFFFF_FFFF, 0x0000_0000, 0xFFFF_FFFF,
    ];
}

/// Cached PKA result for async Start/Read pattern
/// The BLE stack calls Start (begin computation), then later calls Read (get result).
static mut PKA_RESULT_X: [u32; 8] = [0u32; 8];
static mut PKA_RESULT_Y: [u32; 8] = [0u32; 8];
static PKA_RESULT_READY: AtomicBool = AtomicBool::new(false);

fn pka_write_ram_word(offset: usize, value: u32) {
    let pka = embassy_stm32::pac::PKA;
    let word_index = offset / 4;
    unsafe {
        let ram_ptr = pka.ram(word_index).as_ptr() as *mut u32;
        ram_ptr.write_volatile(value);
    }
}

fn pka_read_ram_word(offset: usize) -> u32 {
    let pka = embassy_stm32::pac::PKA;
    let word_index = offset / 4;
    unsafe {
        let ram_ptr = pka.ram(word_index).as_ptr() as *const u32;
        ram_ptr.read_volatile()
    }
}

/// Write a u32 array (LE word order) to PKA RAM at a given offset.
/// Also writes two zero-terminator words after the data (per ST HAL convention).
fn pka_write_operand_words(offset: usize, words: &[u32]) {
    for (i, &word) in words.iter().enumerate() {
        pka_write_ram_word(offset + i * 4, word);
    }
    // Two zero terminators (matching __PKA_RAM_PARAM_END)
    pka_write_ram_word(offset + words.len() * 4, 0);
    pka_write_ram_word(offset + (words.len() + 1) * 4, 0);
}

/// Read u32 words from PKA RAM into a u32 array
fn pka_read_result_words(offset: usize, words: &mut [u32]) {
    for (i, word) in words.iter_mut().enumerate() {
        *word = pka_read_ram_word(offset + i * 4);
    }
}

/// Perform P-256 ECC scalar multiplication: result = k * P
/// k and point coordinates are u32 arrays in LE word order (index 0 = LSW).
/// Returns 0 on success, non-zero on error.
fn pka_p256_mul(k: &[u32; 8], px: &[u32; 8], py: &[u32; 8], rx: &mut [u32; 8], ry: &mut [u32; 8]) -> i32 {
    let pka = embassy_stm32::pac::PKA;

    // Enable PKA clock
    RCC.ahb2enr().modify(|w| w.set_pkaen(true));
    compiler_fence(Ordering::SeqCst);

    // Enable PKA peripheral
    pka.cr().write(|w| w.set_en(true));

    // Wait for INITOK (bit 0 of SR) - RAM initialization complete
    let sr_ptr = pka.sr().as_ptr() as *const u32;
    let mut timeout: u32 = 0;
    loop {
        let sr_raw = unsafe { sr_ptr.read_volatile() };
        if sr_raw & 0x01 != 0 {
            break;
        }
        timeout += 1;
        if timeout > 1_000_000 {
            warn!("PKA INITOK timeout");
            return -1;
        }
    }

    // Clear any pending flags
    pka.clrfr().write(|w| {
        w.set_procendfc(true);
        w.set_ramerrfc(true);
        w.set_addrerrfc(true);
        w.set_operrfc(true);
    });

    // Write bit counts
    pka_write_ram_word(pka_offsets::IN_EXP_NB_BITS, 256); // scalar bits
    pka_write_ram_word(pka_offsets::IN_OP_NB_BITS, 256); // modulus bits
    pka_write_ram_word(pka_offsets::IN_A_COEFF_SIGN, 1); // a is negative

    // Write P-256 curve parameters
    pka_write_operand_words(pka_offsets::IN_A_COEFF, &p256::A);
    pka_write_operand_words(pka_offsets::IN_B_COEFF, &p256::B);
    pka_write_operand_words(pka_offsets::IN_MOD_GF, &p256::P);
    pka_write_operand_words(pka_offsets::IN_N_PRIME_ORDER, &p256::N);

    // Write scalar and point
    pka_write_operand_words(pka_offsets::IN_K, k);
    pka_write_operand_words(pka_offsets::IN_INITIAL_POINT_X, px);
    pka_write_operand_words(pka_offsets::IN_INITIAL_POINT_Y, py);

    // Set mode to ECC scalar multiplication (0x20) and start
    pka.cr().modify(|w| {
        w.set_mode(0x20);
        w.set_procendie(false);
        w.set_ramerrie(false);
        w.set_addrerrie(false);
        w.set_operrie(false);
    });
    pka.cr().modify(|w| w.set_start(true));

    // Wait for completion
    timeout = 0;
    loop {
        let sr = pka.sr().read();
        if sr.ramerrf() || sr.addrerrf() || sr.operrf() {
            pka.clrfr().write(|w| {
                w.set_ramerrfc(true);
                w.set_addrerrfc(true);
                w.set_operrfc(true);
            });
            pka.cr().modify(|w| w.set_en(false));
            warn!("PKA error during ECC mul");
            return -1;
        }
        if sr.procendf() {
            pka.clrfr().write(|w| w.set_procendfc(true));
            break;
        }
        timeout += 1;
        if timeout > 10_000_000 {
            pka.cr().modify(|w| w.set_en(false));
            warn!("PKA timeout during ECC mul");
            return -1;
        }
    }

    // Check result status
    let status = pka_read_ram_word(pka_offsets::OUT_ERROR);
    if status != 0xD60D {
        pka.cr().modify(|w| w.set_en(false));
        warn!("PKA ECC mul failed, status=0x{:08X}", status);
        return -1;
    }

    // Read result
    pka_read_result_words(pka_offsets::OUT_RESULT_X, rx);
    pka_read_result_words(pka_offsets::OUT_RESULT_Y, ry);

    // Disable PKA
    pka.cr().modify(|w| w.set_en(false));

    0 // success
}

// ============================================================================
// BLE Timer Support using embassy_time
// ============================================================================

/// Maximum number of BLE stack timers
const MAX_BLE_TIMERS: usize = 8;

/// Timer deadlines stored as Option<Instant>. None means timer is not active.
static mut TIMER_DEADLINES: [Option<Instant>; MAX_BLE_TIMERS] = [None; MAX_BLE_TIMERS];

/// Flag indicating a timer has expired and sequencer should be woken
static TIMER_EXPIRED: AtomicBool = AtomicBool::new(false);

/// Get the earliest active timer deadline, if any
pub fn earliest_timer_deadline() -> Option<Instant> {
    let mut earliest: Option<Instant> = None;
    unsafe {
        for deadline in TIMER_DEADLINES.iter() {
            if let Some(d) = deadline {
                match earliest {
                    None => earliest = Some(*d),
                    Some(e) if *d < e => earliest = Some(*d),
                    _ => {}
                }
            }
        }
    }
    earliest
}

/// Check and fire any expired timers. Called from the runner loop.
pub fn check_expired_timers() {
    let now = Instant::now();
    unsafe {
        for deadline in TIMER_DEADLINES.iter_mut() {
            if let Some(d) = deadline {
                if now >= *d {
                    *deadline = None;
                    TIMER_EXPIRED.store(true, Ordering::Release);
                }
            }
        }
    }
    if TIMER_EXPIRED.swap(false, Ordering::AcqRel) {
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
pub fn set_nvm_base_address(addr: u32) {
    NVM_BASE_ADDRESS.store(addr, Ordering::Release);
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
    // Calculate page index: (addr - FLASH_BASE) / PAGE_SIZE
    let flash_base = 0x0800_0000u32;
    let page_index = (page_addr - flash_base) / NVM_PAGE_SIZE as u32;

    flash_unlock();

    FLASH.nscr().modify(|w| {
        w.set_per(true);
        w.set_pnb(page_index as u8);
        w.set_bker(false); // Bank 1
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
        let word = u32::from_le_bytes([
            data[i * 4],
            data[i * 4 + 1],
            data[i * 4 + 2],
            data[i * 4 + 3],
        ]);
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

    // Enable AES hardware clock for crypto operations
    RCC.ahb2enr().modify(|w| w.set_aesen(true));
    compiler_fence(Ordering::SeqCst);

    // Enable PKA hardware clock for ECC operations
    RCC.ahb2enr().modify(|w| w.set_pkaen(true));
    compiler_fence(Ordering::SeqCst);

    trace!("BLEPLAT_Init: AES and PKA clocks enabled");
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

/// AES ECB encrypt function using hardware AES peripheral.
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
/// previously set by BLEPLAT_AesCmacSetKey.
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

    cmac_compute(&CMAC_KEY, input_slice, output_slice);
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

    let idx = id as usize;
    if idx >= MAX_BLE_TIMERS {
        warn!("BLEPLAT_TimerStart: invalid timer id {}", id);
        return 1;
    }

    let deadline = Instant::now() + Duration::from_millis(timeout as u64);
    TIMER_DEADLINES[idx] = Some(deadline);

    // Wake the runner so it can update its select/timer
    super::util_seq::seq_pend();

    0 // Success
}

/// Stop a BLE stack timer.
///
/// # Arguments
/// * `id` - Timer ID to stop
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_TimerStop(id: u16) {
    trace!("BLEPLAT_TimerStop: id={}", id);

    let idx = id as usize;
    if idx >= MAX_BLE_TIMERS {
        return;
    }

    unsafe {
        TIMER_DEADLINES[idx] = None;
    }
}

/// NVM store function for BLE stack.
///
/// Stores BLE bonding/configuration data to internal flash.
/// The NVM base address must be set via `set_nvm_base_address()` before use.
///
/// # Arguments
/// * `ptr` - Pointer to data to store
/// * `size` - Size of data in bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn BLEPLAT_NvmStore(ptr: *const u64, size: u16) {
    trace!("BLEPLAT_NvmStore: size={}", size);

    let base = NVM_BASE_ADDRESS.load(Ordering::Acquire);
    if base == 0 {
        trace!("BLEPLAT_NvmStore: NVM not configured, skipping");
        return;
    }

    if ptr.is_null() || size == 0 {
        return;
    }

    let data = core::slice::from_raw_parts(ptr as *const u8, size as usize);

    // Erase the NVM page first
    if !flash_erase_page(base) {
        error!("BLEPLAT_NvmStore: flash erase failed");
        return;
    }

    // Write header: magic + length (fits in first quad-word with padding)
    let mut header = [0u8; NVM_WRITE_SIZE];
    header[0..4].copy_from_slice(&NVM_MAGIC.to_le_bytes());
    header[4..6].copy_from_slice(&size.to_le_bytes());
    // bytes 6..16 are zero padding

    if !flash_write_quadword(base, &header) {
        error!("BLEPLAT_NvmStore: flash write header failed");
        return;
    }

    // Write data starting at offset 16 (second quad-word)
    let data_addr = base + NVM_WRITE_SIZE as u32;
    let mut offset: usize = 0;
    while offset < data.len() {
        let mut quad = [0u8; NVM_WRITE_SIZE];
        let remaining = data.len() - offset;
        let chunk = if remaining >= NVM_WRITE_SIZE { NVM_WRITE_SIZE } else { remaining };
        quad[..chunk].copy_from_slice(&data[offset..offset + chunk]);

        if !flash_write_quadword(data_addr + offset as u32, &quad) {
            error!("BLEPLAT_NvmStore: flash write data failed at offset {}", offset);
            return;
        }
        offset += NVM_WRITE_SIZE;
    }

    trace!("BLEPLAT_NvmStore: stored {} bytes", size);
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

    PKA_RESULT_READY.store(false, Ordering::Release);

    let k: &[u32; 8] = &*(local_private_key as *const [u32; 8]);

    let result = pka_p256_mul(k, &p256::GX, &p256::GY, &mut PKA_RESULT_X, &mut PKA_RESULT_Y);

    if result == 0 {
        PKA_RESULT_READY.store(true, Ordering::Release);
    }

    result
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

    if !PKA_RESULT_READY.load(Ordering::Acquire) {
        warn!("BLEPLAT_PkaReadP256Key: result not ready");
        return -1;
    }

    let out = core::slice::from_raw_parts_mut(local_public_key, 16);
    out[0..8].copy_from_slice(&PKA_RESULT_X);
    out[8..16].copy_from_slice(&PKA_RESULT_Y);

    PKA_RESULT_READY.store(false, Ordering::Release);

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

    PKA_RESULT_READY.store(false, Ordering::Release);

    let k: &[u32; 8] = &*(local_private_key as *const [u32; 8]);
    let remote = core::slice::from_raw_parts(remote_public_key, 16);

    let mut px = [0u32; 8];
    let mut py = [0u32; 8];
    px.copy_from_slice(&remote[0..8]);
    py.copy_from_slice(&remote[8..16]);

    let result = pka_p256_mul(k, &px, &py, &mut PKA_RESULT_X, &mut PKA_RESULT_Y);

    if result == 0 {
        PKA_RESULT_READY.store(true, Ordering::Release);
    }

    result
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

    if !PKA_RESULT_READY.load(Ordering::Acquire) {
        warn!("BLEPLAT_PkaReadDhKey: result not ready");
        return -1;
    }

    // DH key is just the X coordinate of the shared point
    let out = core::slice::from_raw_parts_mut(dh_key, 8);
    out.copy_from_slice(&PKA_RESULT_X);

    PKA_RESULT_READY.store(false, Ordering::Release);

    BLEPLAT_OK
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
