//! BLE Rhai Playground for STM32WBA65I-DK1
//!
//! Receives Rhai scripts over BLE NUS (Nordic UART Service). After 500 ms of
//! idle time the accumulated input is dispatched to a dedicated eval task.
//! The eval task runs the Rhai engine and sends the result back over a channel;
//! the BLE task then notifies the client without ever blocking on eval.
//!
//! Board extras vs the WBA5MM module demo:
//! - 256 KB Rhai heap (512 KB SRAM on WBA65)
//! - RGB user LEDs (PD8/PD9/PB10)
//! - Joystick on ADC4/PA3 (`joy()`)
//! - SSD1306 OLED on SPI3 (`oled_line()`, `oled_clear()`; `print()` mirrors to line 7 live)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐   ┌──────────────────────────────┐
//! │  main / BLE task                    │   │  eval_task                   │
//! │                                     │   │                              │
//! │  select3(                           │   │  owns: Engine, RGB LEDs      │
//! │    ble.read_event(),         ──────────→│  SCRIPT_CHAN.receive()       │
//! │    RESULT_CHAN.receive(),    ←──────────│  engine.eval()               │
//! │    Timer::after_millis(500),    │   │   │  LED_STATE → set_level()     │
//! │  )                              │   │   │  RESULT_CHAN.send()          │
//! │                                 │   │   └──────────────────────────────┘
//! │  on timeout  → SCRIPT_CHAN.send()       (eval_pending flag prevents
//! │  on result   → gatt.notify()             double-dispatch)
//! │  on BLE event → handle connect /
//! │                 disconnect / RX data
//! └─────────────────────────────────────┘
//! ```
//!
//! BLE events are never blocked while Rhai evaluates. Both channels have
//! depth 1, so the eval task processes one script at a time.
//!
//! ## Build
//! cargo build --release --bin ble_rhai --features scripting

#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;

use embedded_alloc::LlffHeap as Heap;

use core::cell::RefCell;
// defmt::* would bring in a `panic_handler` proc-macro that conflicts with the
// built-in #[panic_handler] attribute used for our custom panic handler below.
// Import the macros we actually use explicitly instead of glob-importing.
use defmt::{debug, error, info, warn, unwrap};
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_futures::block_on;
use embassy_futures::select::{Either3, Either4, select3, select4};
use embassy_stm32::aes::{self, Aes};
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering, Ordering as AtomicOrdering};

use embassy_stm32::adc::{Adc, adc4};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::Peri;
use embassy_stm32::interrupt::{self, InterruptExt, Priority};
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rcc;
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_stm32wba65i_dk1_examples::board::{self, JoyDir, LedBank, LedId};
use embassy_stm32wba65i_dk1_examples::oled::{OledBus, PRINT_LINE};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_stm32_wpan::bluetooth::{HCI, Normal};
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gap::types::OwnAddressType;
use embassy_stm32_wpan::bluetooth::gatt::{
    CccdValue, CharProperties, GattEventMask, SecurityPermissions, ServiceType, Uuid,
    is_cccd_handle, is_value_handle,
};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use embassy_time::Timer;
use rhai::{Dynamic, Engine, packages::BasicIteratorPackage, packages::BasicMathPackage, packages::BasicStringPackage, packages::MoreStringPackage, packages::Package};
use static_cell::StaticCell;
use stm32wb_hci::Event;
use stm32wb_hci::vendor::event::{AttExchangeMtuResponse, VendorEvent};
use defmt_rtt as _; // RTT logging backend; panic handler is custom below (no panic_probe)
use cortex_m_rt::{exception, interrupt};

#[global_allocator]
static HEAP: Heap = Heap::empty();

// RAM layout (512 KB SRAM on STM32WBA65RI):
//   BSS baseline (BLE stack + task state) ≈ 33 KB  (excl. HEAP_MEM)
//   data section (BLE blob init data)     ≈ 35 KB
//   HEAP_MEM (below)                      = 256 KB
//   OLED framebuffer + stack              ≈ remainder
//
// Stack budget is still the critical constraint for Rhai user-defined functions.
// Three focused packages (Math+Iterator+String) with only_i32 keep stack usage low.
const HEAP_SIZE: usize = 256 * 1024;

// Max total bytes buffered in pending_output while disconnected.
// Caps heap use during long eval runs with many print() calls.
const MAX_PENDING_BYTES: usize = 16 * 1024;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
    AES => aes::InterruptHandler<AesPeriph>;
    PKA => pka::InterruptHandler<PkaPeriph>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

// Nordic UART Service (NUS) UUIDs
const NUS_SERVICE_UUID: [u8; 16] = [
    0x9E, 0xCA, 0xDC, 0x24, 0x0E, 0xE5, 0xA9, 0xE0, 0x93, 0xF3, 0xA3, 0xB5, 0x01, 0x00, 0x40, 0x6E,
];
const NUS_RX_CHAR_UUID: [u8; 16] = [
    0x9E, 0xCA, 0xDC, 0x24, 0x0E, 0xE5, 0xA9, 0xE0, 0x93, 0xF3, 0xA3, 0xB5, 0x02, 0x00, 0x40, 0x6E,
];
const NUS_TX_CHAR_UUID: [u8; 16] = [
    0x9E, 0xCA, 0xDC, 0x24, 0x0E, 0xE5, 0xA9, 0xE0, 0x93, 0xF3, 0xA3, 0xB5, 0x03, 0x00, 0x40, 0x6E,
];

// Per-packet BLE payload limit. BLE 5.0 allows ATT MTU up to 517 bytes;
// max payload = ATT_MTU - 3 = 514. We use 512 as a clean round number that
// fits within both the stack's negotiated MTU and the ATT event buffer.
const MAX_DATA_LEN: usize = 512;

// Max accumulated script size from NUS RX (multiple 512-byte writes).
const MAX_SCRIPT_BYTES: usize = 1024;

// BLE task → eval task: raw script bytes
static SCRIPT_CHAN: Channel<CriticalSectionRawMutex, Vec<u8>, 1> =
    Channel::new();

// eval task → BLE task: final eval result
static RESULT_CHAN: Channel<CriticalSectionRawMutex, Vec<u8>, 1> =
    Channel::new();

// print()/debug() output: each call sends one Vec<u8> immediately during eval.
// depth 8: scripts rarely print more than 8 lines between awaits.
static PRINT_CHAN: Channel<CriticalSectionRawMutex, Vec<u8>, 8> =
    Channel::new();

/// Live OLED framebuffer + SPI bus, flushed synchronously from eval (on_print /
/// oled_line) because engine.eval() monopolises the thread executor.
struct OledScreen {
    bus: OledBus,
    lines: [heapless::String<22>; 8],
}

static OLED_SCREEN: Mutex<CriticalSectionRawMutex, RefCell<Option<OledScreen>>> =
    Mutex::new(RefCell::new(None));

fn oled_apply_line(line: u8, text: &str) {
    OLED_SCREEN.lock(|cell| {
        let mut slot = cell.borrow_mut();
        let Some(screen) = slot.as_mut() else {
            return;
        };
        if line == 0xFF {
            for l in screen.lines.iter_mut() {
                l.clear();
            }
        } else if (line as usize) < screen.lines.len() {
            screen.lines[line as usize].clear();
            screen.lines[line as usize]
                .push_str(text.get(..21).unwrap_or(text))
                .ok();
        }
        let _ = screen.bus.render_lines(&screen.lines);
    });
}

/// Forward one line to BLE notify (and OLED line 7 when `mirror_oled`).
fn script_print(line: &str, mirror_oled: bool) {
    if HEAP.free() < line.len() + 16 + MIN_HEAP_RESERVE {
        return;
    }
    if mirror_oled {
        oled_apply_line(PRINT_LINE, line);
    }
    let mut v: Vec<u8> = Vec::with_capacity(line.len() + 2);
    v.extend_from_slice(line.as_bytes());
    v.push(b'\r');
    v.push(b'\n');
    let _ = PRINT_CHAN.try_send(v);
}

const HELP_LINES: &[&str] = &[
    "=== RhaiPlay STM32WBA65I-DK1 ===",
    "-- board --",
    "led(on)           green LED (same as led(0,on))",
    "led(n, on)        LED 0=gr 1=rd 2=bl",
    "led_toggle(n)     toggle LED, ret 0/1",
    "rgb(r,g,b)        all RGB LEDs (0/1 or bool)",
    "joy()             0 none 1 sel 2 L 3 D 4 U 5 R",
    "oled_line(n,text) OLED row 0..7",
    "oled_clear()      clear OLED rows",
    "ts()              uptime ticks (32768 Hz)",
    "sleep(ms)         delay ms (use instead of spin)",
    "heap_free()       free heap bytes",
    "diag()            joy poll count (debug)",
    "help()            this listing",
    "-- I/O --",
    "print(\"…\")        BLE + OLED line 7",
    "debug(\"…\")        same, debug channel",
    "-- Rhai packages --",
    "Math: + - * / % ** abs min max",
    "Iterator: len, for-in, while, if",
    "String: +, ==, !=, <, >, substr",
    "  .len .contains .trim .replace",
    "  .split .sub_string .to_upper",
    "Arrays: [], +=, [i], max 4096 el",
    "Send idle 500ms or newline to eval",
];

// Latest joystick direction, updated by input_task (read by joy() in eval).
static JOY_STATE: AtomicU8 = AtomicU8::new(JoyDir::None as u8);

// #region agent log
// Incremented by input_task each ADC poll; exposed via diag() for concurrency checks.
static JOY_POLL_COUNT: AtomicU32 = AtomicU32::new(0);
// #endregion

// Set while engine.eval() is running; lets the BLE task detect an in-progress eval on disconnect.
static EVAL_RUNNING: AtomicBool = AtomicBool::new(false);

// Set by the on_progress heap-guard when free heap drops below MIN_HEAP_RESERVE.
// Cleared atomically in the error handler so the eval loop can report a clean
// "out of memory" message instead of the opaque Rhai termination error.
static OOM_TERMINATED: AtomicBool = AtomicBool::new(false);

// Set by the on_progress stack-guard when MSP falls within STACK_GUARD_MARGIN of
// _stack_end (the linker-defined bottom of the thread stack).  Cleared in the eval
// error handler so a clean "stack overflow" message is forwarded to the BLE client.
static STACK_OVERFLOW_TERMINATED: AtomicBool = AtomicBool::new(false);
// MSP value captured by on_progress at the moment the overflow is detected.
// Used to report the exact stack depth to the BLE client.
static STACK_OVERFLOW_MSP: AtomicU32 = AtomicU32::new(0);

// Computed once in main(): addr_of!(_stack_end) + STACK_GUARD_MARGIN.
// on_progress reads this and aborts the script when MSP < limit.
static STACK_GUARD_LIMIT: AtomicU32 = AtomicU32::new(0);

// Safety margin above the hard stack bottom (_stack_end).  2 KB leaves enough
// room for on_progress itself, the return path through Rhai, and any interrupt
// frames (ICACHE/RADIO/HASH) that may arrive during the unwind.
const STACK_GUARD_MARGIN: u32 = 2 * 1024;

// Linker-defined thread-stack bounds (cortex-m-rt symbols).
unsafe extern "C" {
    static _stack_end: u8;   // bottom (lowest address, must not be written)
    static _stack_start: u8; // top    (initial SP value = 0x20020000)
}

// Persists across a SYSRESET (software reset) — placed in .uninit so startup code
// never zeroes it. The panic handler writes RESET_CAUSE_OOM before resetting;
// the BLE task reads and clears it on the next CCCD subscribe to notify the client.
//
// On a fresh power-up RAM is random, so false-positive probability is 1/2^32.
// After reading, the value is always cleared to 0.
#[unsafe(link_section = ".uninit")]
static mut RESET_CAUSE: u32 = 0;
const RESET_CAUSE_OOM: u32 = 0xDEAD_0001;
const RESET_CAUSE_STACKOVERFLOW: u32 = 0xDEAD_0002;
const RESET_CAUSE_PANIC: u32 = 0xDEAD_0003; // any panic outside of eval (e.g. BLE init, alloc)

/// Custom panic handler: logs via defmt, then either halts (debugger attached)
/// or performs a clean SYSRESET (deployed, no debugger).
///
/// **Debugger attached** (probe-rs / cargo run):
///   Disables interrupts and spins in a `bkpt` loop. probe-rs stays connected,
///   the session is not terminated, and the error is visible in the RTT log.
///   Disconnect/reflash to recover.
///
/// **No debugger** (deployed device):
///   Writes RESET_CAUSE (survives SYSRESET in .uninit RAM), waits briefly for
///   RTT to flush, then calls SCB::sys_reset(). The firmware reboots, the BLE
///   stack re-initialises, and the next CCCD subscribe delivers the error
///   message to the BLE client automatically.
///
/// Debugger detection uses the Cortex-M DHCSR register (0xE000_EDF0), bit 0
/// = C_DEBUGEN. This is a read-only status bit set by the debug port.
#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    if EVAL_RUNNING.load(Ordering::Relaxed) {
        // Most likely OOM from a large array/string allocation inside Rhai.
        defmt::error!("panic during script eval (OOM?) — check Rhai array/string limits");
        // Safety: single-core, interrupts disabled.
        unsafe { RESET_CAUSE = RESET_CAUSE_OOM; }
    } else {
        defmt::error!("panic");
        // Safety: single-core, interrupts disabled.
        unsafe { RESET_CAUSE = RESET_CAUSE_PANIC; }
    }
    // Brief busy-wait to give RTT a chance to flush.
    cortex_m::asm::delay(100_000);
    // DHCSR C_DEBUGEN bit: 1 = debugger connected.
    const DHCSR: *const u32 = 0xE000_EDF0 as *const u32;
    if unsafe { core::ptr::read_volatile(DHCSR) } & 1 != 0 {
        // Debugger attached: halt here so probe-rs keeps its session.
        // User can inspect logs, then disconnect to trigger a reset.
        defmt::error!("halting (debugger attached) — disconnect probe to reset");
        loop { cortex_m::asm::bkpt(); }
    }
    // No debugger: reset and auto-recover.
    cortex_m::peripheral::SCB::sys_reset()
}

/// Hardware stack overflow guard (Cortex-M33 MSPLIM).
///
/// Fires as a UsageFault (CFSR STKOF, bit 20) when MSP crosses the limit set
/// in main(). The limit is placed 2 KB above the end of static data so the
/// handler always has clean stack to report the error before RAM is corrupted.
///
/// Same reset/halt logic as the panic handler: SYSRESET on deployed device,
/// bkpt loop when debugger is attached.
#[exception]
unsafe fn UsageFault() {
    cortex_m::interrupt::disable();
    // Read and clear the Configurable Fault Status Register (sticky bits).
    const SCB_CFSR: *mut u32 = 0xE000_ED28 as *mut u32;
    let cfsr = unsafe { core::ptr::read_volatile(SCB_CFSR) };
    unsafe { core::ptr::write_volatile(SCB_CFSR, cfsr) };
    if (cfsr >> 20) & 1 != 0 {
        // STKOF: stack pointer crossed MSPLIM.
        if EVAL_RUNNING.load(Ordering::Relaxed) {
            defmt::error!("stack overflow during script eval (MSPLIM) — resetting");
        } else {
            defmt::error!("stack overflow (MSPLIM) — resetting");
        }
        unsafe { RESET_CAUSE = RESET_CAUSE_STACKOVERFLOW; }
    } else {
        defmt::error!("UsageFault CFSR=0x{:08X} — resetting", cfsr);
    }
    cortex_m::asm::delay(100_000);
    const DHCSR: *const u32 = 0xE000_EDF0 as *const u32;
    if unsafe { core::ptr::read_volatile(DHCSR) } & 1 != 0 {
        defmt::error!("halting (debugger attached) — disconnect probe to reset");
        loop { cortex_m::asm::bkpt(); }
    }
    cortex_m::peripheral::SCB::sys_reset()
}

// Minimum free heap to maintain during script execution.
// on_progress checks this between Rhai steps and terminates the script if
// free heap drops below this threshold. Used only for post-eval cleanup:
// alloc::format!() and result_buf.extend_from_slice() need a few hundred bytes
// after OOM termination.
//
// In-step print()/debug() allocations are now guarded at the call site:
// on_print/on_debug check HEAP.free() before allocating and silently skip the
// Vec copy when free heap would drop below s.len() + 16 + MIN_HEAP_RESERVE.
// This replaces the old 6 KB worst-case heuristic with an exact per-call check,
// freeing ~4 KB more for script use without weakening post-eval cleanup safety.
const MIN_HEAP_RESERVE: usize = 2 * 1024;

// RGB LED bank shared between Rhai closures and input_task.
static LED_BANK: Mutex<CriticalSectionRawMutex, RefCell<Option<LedBank>>> =
    Mutex::new(RefCell::new(None));

static LED_CELL: StaticCell<LedBank> = StaticCell::new();

// FullRuntime is stored inside new_platform!'s internal StaticCell — already 'static.
// EXECUTOR_BLE: high-priority interrupt executor for ble_runner_task, ble_task,
// and input_task. All run at Priority::P4 so they preempt eval_task (thread
// executor) during engine.eval(), keeping BLE alive and joy() up to date.
//
// ICACHE (IRQ 64) is used as a pure software-trigger interrupt.
// On Cortex-M there is no dedicated software IRQ; borrowing a peripheral IRQ
// is the standard pattern. ICACHE is chosen because it only fires on cache ECC
// errors and is never enabled for hardware use in this firmware, so there is
// zero risk of accidental triggering.
static EXECUTOR_BLE: InterruptExecutor = InterruptExecutor::new();

#[interrupt]
unsafe fn ICACHE() {
    unsafe { EXECUTOR_BLE.on_interrupt() }
}

// ---------------------------------------------------------------------------
// BLE platform tasks
// ---------------------------------------------------------------------------

#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

// ---------------------------------------------------------------------------
// Eval task — owns the Rhai Engine and the LED pin (thread executor)
// ---------------------------------------------------------------------------

#[embassy_executor::task]
async fn eval_task() {

    let mut engine = Engine::new_raw();
    // Three focused packages (no CorePackage — its init uses too much stack on
    // this device and overflows into BLE BSS). With only_i32 each package
    // registers i32+f32 variants only → roughly half the function-table entries.
    BasicMathPackage::new().register_into_engine(&mut engine);
    BasicIteratorPackage::new().register_into_engine(&mut engine);
    BasicStringPackage::new().register_into_engine(&mut engine);
    MoreStringPackage::new().register_into_engine(&mut engine);

    engine.register_fn("led", |state: bool| -> bool {
        LED_BANK.lock(|cell| {
            if let Some(bank) = cell.borrow_mut().as_mut() {
                bank.set(LedId::Green, state);
            }
        });
        state
    });

    engine.register_fn("led", |state: i32| -> i32 {
        let on = state != 0;
        LED_BANK.lock(|cell| {
            if let Some(bank) = cell.borrow_mut().as_mut() {
                bank.set(LedId::Green, on);
            }
        });
        if on { 1 } else { 0 }
    });

    engine.register_fn("led", |index: i32, state: bool| -> bool {
        if let Some(id) = LedId::from_i32(index) {
            LED_BANK.lock(|cell| {
                if let Some(bank) = cell.borrow_mut().as_mut() {
                    bank.set(id, state);
                }
            });
        }
        state
    });

    engine.register_fn("led", |index: i32, state: i32| -> i32 {
        let on = state != 0;
        if let Some(id) = LedId::from_i32(index) {
            LED_BANK.lock(|cell| {
                if let Some(bank) = cell.borrow_mut().as_mut() {
                    bank.set(id, on);
                }
            });
        }
        if on { 1 } else { 0 }
    });

    engine.register_fn("led_toggle", |index: i32| -> i32 {
        if let Some(id) = LedId::from_i32(index) {
            let mut on = false;
            LED_BANK.lock(|cell| {
                if let Some(bank) = cell.borrow_mut().as_mut() {
                    on = bank.toggle(id);
                }
            });
            return if on { 1 } else { 0 };
        }
        -1
    });

    engine.register_fn("rgb", |r: bool, g: bool, b: bool| -> i32 {
        LED_BANK.lock(|cell| {
            if let Some(bank) = cell.borrow_mut().as_mut() {
                bank.set_rgb(r, g, b);
            }
        });
        0
    });

    engine.register_fn("rgb", |r: i32, g: i32, b: i32| -> i32 {
        LED_BANK.lock(|cell| {
            if let Some(bank) = cell.borrow_mut().as_mut() {
                bank.set_rgb(r != 0, g != 0, b != 0);
            }
        });
        0
    });

    engine.register_fn("joy", || -> i32 {
        JOY_STATE.load(AtomicOrdering::Relaxed) as i32
    });

    engine.register_fn("sleep", |ms: i32| -> i32 {
        let ms = ms.clamp(0, 60_000) as u64;
        block_on(Timer::after_millis(ms));
        0
    });

    // #region agent log
    engine.register_fn("diag", || -> i32 {
        JOY_POLL_COUNT.load(AtomicOrdering::Relaxed) as i32
    });
    // #endregion

    engine.register_fn("oled_line", |line: i32, text: &str| -> i32 {
        if !(0..8).contains(&line) {
            return -1;
        }
        oled_apply_line(line as u8, text);
        line
    });

    engine.register_fn("oled_clear", || -> i32 {
        oled_apply_line(0xFF, "");
        0
    });

    engine.register_fn("ts", || { // timestamp in ticks (32768 Hz); i32 with only_i32
        embassy_time::Instant::now().as_ticks() as i32
    });

    engine.register_fn("heap_free", || { // free heap in bytes (i32); scripts can query memory
        HEAP.free() as i32
    });

    engine.register_fn("help", || -> i32 {
        for line in HELP_LINES {
            script_print(line, false);
        }
        oled_apply_line(6, "help sent (BLE)");
        HELP_LINES.len() as i32
    });

    // Each print()/debug() call sends immediately to PRINT_CHAN.
    // ble_task runs at interrupt priority and will preempt eval_task to forward
    // the notification to the BLE client without waiting for eval to finish.
    engine.on_print(|s| {
        info!("print: {}", s);
        script_print(s, true);
    });
    engine.on_debug(|s, _src, _pos| {
        info!("debug: {}", s);
        script_print(s, false);
    });

    // Rhai safety limits — enforced BEFORE the allocation attempt, so violations
    // return a clean EvalAltResult error instead of an allocator panic.
    //
    // set_max_array_size: effective cap of 1024 elements.
    // Heap budget analysis (48 KB total, ~18 KB engine init = ~30 KB free; ~28 KB effective):
    //   A Rhai Dynamic is 16 bytes on 32-bit. 1024 elements = 16 KB in the array.
    //   Rhai's `+=` operator checks `old_size <= limit` BEFORE appending (bug: it
    //   should check `old_size + new_len <= limit`).  With limit=1024 the check
    //   passes when old_size=1024, append is called, and Vec tries to double its
    //   capacity 1024→2048 (32 KB) — causing OOM.
    //   Workaround: set limit to desired_max-1 = 1023.  Then the check fires when
    //   old_size=1024 (1024 > 1023), returning a clean error before any growth.
    //   Vec capacity never exceeds 1024 (16 KB), which fits the heap comfortably.
    //
    // set_max_string_size: 1 KB cap against runaway string building.
    //
    // set_max_operations + on_progress: belt-and-suspenders for non-array OOM
    // and infinite loops. on_progress fires between Rhai steps so it cannot
    // stop an in-step Vec realloc — the size limits above are the primary guard.
    engine.set_max_array_size(4095); // effective limit 4096 elements (~64 KB Dynamic array)
    engine.set_max_string_size(4095);
    engine.set_max_operations(0); // no fixed limit; rely on on_progress heap check for OOM and user-friendly error reporting
    // Soft call-stack depth limit (secondary guard).
    // The primary guard is the software MSP check in on_progress: it fires a
    // clean Rhai termination before MSP crosses _stack_end (the thread stack
    // bottom).  This soft limit acts as a belt-and-suspenders check that catches
    // recursive scripts early enough for the MSP check to remain meaningful.
    //
    // With ~14 KB thread stack and ~1-2 KB per Rhai call level, the MSP check
    // fires around depth 8-10.  Setting max_call_levels to 12 means well-behaved
    // recursive scripts get a clean "too many levels" error rather than the
    // on_progress abort message.
    engine.set_max_call_levels(12);
    engine.on_progress(|_ops| {
        // Software stack guard: read MSP and abort before it crosses _stack_end.
        // MSPLIM cannot be used here because the sequencer context-switch swaps
        // MSP directly (see context.rs); on_progress is our only safe check point.
        let msp: u32;
        unsafe { core::arch::asm!("mrs {0}, msp", out(reg) msp, options(nomem, nostack, preserves_flags)) }
        let limit = STACK_GUARD_LIMIT.load(Ordering::Relaxed);
        if limit != 0 && msp < limit {
            STACK_OVERFLOW_MSP.store(msp, Ordering::Relaxed);
            STACK_OVERFLOW_TERMINATED.store(true, Ordering::Relaxed);
            return Some(Dynamic::UNIT);
        }
        if HEAP.free() < MIN_HEAP_RESERVE {
            OOM_TERMINATED.store(true, Ordering::Relaxed);
            Some(Dynamic::UNIT) // signal Rhai to stop immediately
        } else {
            None
        }
    });

    info!("eval_task ready");

    loop {
        let script_bytes = SCRIPT_CHAN.receive().await;

        let mut result_buf: Vec<u8> = Vec::new();

        if let Ok(script) = core::str::from_utf8(&script_bytes) {
            info!("eval: {} bytes  heap_free={} KB\n{}", script_bytes.len(), HEAP.free() / 1024, script);

            EVAL_RUNNING.store(true, Ordering::Relaxed);
            let eval_result = match engine.eval::<Dynamic>(script) {
                Ok(result) => {
                    let value = if result.is_string() {
                        result.into_string().unwrap_or_default()
                    } else {
                        alloc::format!("{}", result)
                    };
                    alloc::format!("{}\r\n", value)
                }
                Err(e) => {
                    if STACK_OVERFLOW_TERMINATED.swap(false, Ordering::Relaxed) {
                        let msp = STACK_OVERFLOW_MSP.load(Ordering::Relaxed);
                        let stack_top = core::ptr::addr_of!(_stack_start) as u32;
                        let stack_bot = core::ptr::addr_of!(_stack_end)   as u32;
                        let used  = stack_top.saturating_sub(msp);
                        let total = stack_top.saturating_sub(stack_bot);
                        let free  = total.saturating_sub(used);
                        error!("eval stack overflow: used={} KB, free={} KB, total={} KB (MSP=0x{:08X})",
                               used / 1024, free / 1024, total / 1024, msp);
                        alloc::format!("err: stack overflow (used {} KB, {} KB free of {} KB total)\r\n",
                                       used / 1024, free / 1024, total / 1024)
                    } else if OOM_TERMINATED.swap(false, Ordering::Relaxed) {
                        let free = HEAP.free();
                        error!("eval OOM: {} KB heap free (reserve {} KB)", free / 1024, MIN_HEAP_RESERVE / 1024);
                        alloc::format!("err: out of memory ({} KB heap free)\r\n", free / 1024)
                    } else {
                        warn!("eval err: {}", alloc::format!("{}", e).as_str());
                        alloc::format!("err: {}\r\n", e)
                    }
                }
            };
            EVAL_RUNNING.store(false, Ordering::Relaxed);
            info!("eval done: heap_free={} KB", HEAP.free() / 1024);

            result_buf.extend_from_slice(eval_result.as_bytes());
        }

        RESULT_CHAN.send(result_buf).await;
    }
}

// ---------------------------------------------------------------------------
// OLED task — init SPI3 + SSD1306, publish shared state for eval-time flush
// ---------------------------------------------------------------------------

#[embassy_executor::task]
async fn display_task(mut oled: OledBus) {
    loop {
        if oled.try_init().is_ok() {
            break;
        }
        warn!("OLED init failed, retrying…");
        Timer::after_millis(500).await;
    }

    let mut lines: [heapless::String<22>; 8] = Default::default();
    lines[0].push_str(board::BOARD_NAME).ok();
    lines[2].push_str("BLE Rhai Playground").ok();
    lines[4].push_str(board::BLE_ADV_NAME).ok();
    let _ = oled.render_lines(&lines);

    OLED_SCREEN.lock(|cell| {
        *cell.borrow_mut() = Some(OledScreen { bus: oled, lines });
    });

    loop {
        Timer::after_secs(3600).await;
    }
}

// ---------------------------------------------------------------------------
// Input task — polls joystick ADC on the interrupt executor so it keeps running
// while eval_task monopolises the thread executor during engine.eval().
// ---------------------------------------------------------------------------

#[embassy_executor::task]
async fn input_task(
    mut adc: Adc<'static, peripherals::ADC4>,
    mut joy_pin: Peri<'static, peripherals::PA3>,
) {
    adc.set_resolution_adc4(adc4::Resolution::Bits12);
    let max = adc4::resolution_to_max_count(adc4::Resolution::Bits12) as u16;

    loop {
        let raw = adc.blocking_read(&mut joy_pin, adc4::SampleTime::Cycles15);
        let dir = JoyDir::from_raw(raw, max);
        JOY_STATE.store(dir as u8, AtomicOrdering::Relaxed);
        // #region agent log
        JOY_POLL_COUNT.fetch_add(1, AtomicOrdering::Relaxed);
        // #endregion
        // Script owns RGB while eval runs; live joy hints only when idle.
        if !EVAL_RUNNING.load(AtomicOrdering::Relaxed) {
            LED_BANK.lock(|cell| {
                if let Some(bank) = cell.borrow_mut().as_mut() {
                    bank.show_joy(dir);
                }
            });
        }
        Timer::after_millis(80).await;
    }
}

// ---------------------------------------------------------------------------
// BLE task — runs on interrupt executor (Priority::P4) so it can preempt
// eval_task (thread executor) and forward print() output in real time.
// ---------------------------------------------------------------------------

#[embassy_executor::task]
async fn ble_task(mut ble: HCI<'static, Normal>) {
    let mut gatt = ble.gatt_server();

    let service_handle = gatt
        .add_service(Uuid::from_u128_le(NUS_SERVICE_UUID), ServiceType::Primary, 10)
        .expect("add NUS service");

    let rx_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u128_le(NUS_RX_CHAR_UUID),
            MAX_DATA_LEN as u16,
            CharProperties::WRITE | CharProperties::WRITE_WITHOUT_RESPONSE,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            true,
        )
        .expect("add RX char");

    let tx_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u128_le(NUS_TX_CHAR_UUID),
            MAX_DATA_LEN as u16,
            CharProperties::NOTIFY,
            SecurityPermissions::NONE,
            GattEventMask::empty(),
            0,
            true,
        )
        .expect("add TX char");

    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).unwrap();
    adv_data.add_name(board::BLE_ADV_NAME).unwrap();

    let mut scan_rsp = AdvData::new();
    scan_rsp.add_service_uuid_128(&NUS_SERVICE_UUID).unwrap();

    let adv_params = AdvParams {
        interval_min: 0x0050,
        interval_max: 0x0064,
        adv_type: AdvType::ConnectableUndirected,
        own_addr_type: OwnAddressType::Random,
        ..AdvParams::default()
    };

    ble.start_advertising(adv_params.clone(), adv_data.clone(), Some(scan_rsp.clone()))
        .await
        .expect("start advertising");

    info!("Advertising as '{}' — connect and send Rhai expressions", board::BLE_ADV_NAME);

    let mut input_buf: Vec<u8> = Vec::with_capacity(MAX_SCRIPT_BYTES);
    let mut tx_notifications = false;
    let mut conn_handle: Option<u16> = None;
    let mut eval_pending = false;
    // Notifications buffered while disconnected (no connection / CCCD not subscribed yet).
    // Drained as soon as the client re-subscribes. Capped at MAX_PENDING_BYTES total
    // to prevent heap exhaustion during long disconnected eval runs.
    let mut pending_output: Vec<Vec<u8>> = Vec::new();
    let mut pending_bytes: usize = 0;

    loop {
        // 4-way select when buffer is pending dispatch (500 ms timeout), else 3-way.
        enum Msg<E> { Ble(E), Result(Vec<u8>), Print(Vec<u8>), Timeout }

        let msg = if !input_buf.is_empty() {
            match select4(
                ble.read_event(),
                RESULT_CHAN.receive(),
                PRINT_CHAN.receive(),
                embassy_time::Timer::after_millis(500),
            ).await {
                Either4::First(ev)  => Msg::Ble(ev),
                Either4::Second(r)  => Msg::Result(r),
                Either4::Third(p)   => Msg::Print(p),
                Either4::Fourth(_)  => Msg::Timeout,
            }
        } else {
            match select3(
                ble.read_event(),
                RESULT_CHAN.receive(),
                PRINT_CHAN.receive(),
            ).await {
                Either3::First(ev)  => Msg::Ble(ev),
                Either3::Second(r)  => Msg::Result(r),
                Either3::Third(p)   => Msg::Print(p),
            }
        };

        match msg {
            // ----------------------------------------------------------------
            // print()/debug() output — forward immediately to BLE client
            // ----------------------------------------------------------------
            Msg::Print(data) => {
                if let Some(conn) = conn_handle {
                    if tx_notifications {
                        for chunk in data.chunks(MAX_DATA_LEN) {
                            if let Err(e) = gatt.notify(conn, service_handle, tx_char_handle, chunk) {
                                warn!("print notify failed: {:?}", defmt::Debug2Format(&e));
                            }
                        }
                    } else {
                        // Connected but CCCD not yet subscribed — buffer for drain.
                        if pending_bytes + data.len() <= MAX_PENDING_BYTES {
                            pending_bytes += data.len();
                            pending_output.push(data);
                        } else {
                            warn!("pending_output full, print dropped");
                        }
                    }
                } else {
                    // No connection — buffer for replay on next reconnect.
                    if pending_bytes + data.len() <= MAX_PENDING_BYTES {
                        pending_bytes += data.len();
                        pending_output.push(data);
                    } else {
                        warn!("pending_output full, print dropped");
                    }
                }
            }

            // ----------------------------------------------------------------
            // 500 ms idle — dispatch buffer to eval task
            // ----------------------------------------------------------------
            Msg::Timeout => {
                if !eval_pending {
                    // mem::take moves the buffer into the channel without cloning,
                    // freeing the script bytes from the BLE task's heap as soon as
                    // eval_task takes ownership.
                    SCRIPT_CHAN.send(core::mem::take(&mut input_buf)).await;
                    eval_pending = true;
                } else {
                    input_buf.clear();
                }
            }

            // ----------------------------------------------------------------
            // Eval result arrived — notify client
            // ----------------------------------------------------------------
            Msg::Result(result) => {
                eval_pending = false;
                if let Some(conn) = conn_handle {
                    for chunk in result.chunks(MAX_DATA_LEN) {
                        let _ = gatt.notify(conn, service_handle, tx_char_handle, chunk);
                    }
                    let _ = gatt.notify(conn, service_handle, tx_char_handle, b"> ");
                } else {
                    // Connection lost before result arrived — buffer for replay.
                    if !result.is_empty() {
                        if pending_bytes + result.len() <= MAX_PENDING_BYTES {
                            info!("result buffered for replay ({} bytes)", result.len());
                            pending_bytes += result.len();
                            pending_output.push(result);
                        } else {
                            warn!("pending_output full, result dropped");
                        }
                    }
                }
            }

            // ----------------------------------------------------------------
            // BLE event
            // ----------------------------------------------------------------
            Msg::Ble(event) => {
                if let Some(gap_event) = ble.process_event(&event) {
                    match gap_event {
                        GapEvent::Connected(conn) => {
                            info!("Connected: 0x{:04X} (pending_output={})", conn.handle.0, pending_output.len());
                            conn_handle = Some(conn.handle.0);
                            tx_notifications = false;
                            input_buf.clear();
                            eval_pending = false;
                            // pending_output intentionally kept — drained on CCCD subscribe.
                        }
                        GapEvent::Disconnected { handle, reason } => {
                            if EVAL_RUNNING.load(Ordering::Relaxed) {
                                warn!("Disconnected during eval — BLE runner starved by engine.eval(): handle=0x{:04X} reason=0x{:02X}", handle.0, reason);
                            } else {
                                info!("Disconnected: 0x{:04X} reason=0x{:02X}", handle.0, reason);
                            }
                            if !input_buf.is_empty() && !eval_pending {
                                SCRIPT_CHAN.send(core::mem::take(&mut input_buf)).await;
                                eval_pending = true;
                            }
                            conn_handle = None;
                            tx_notifications = false;
                            ble.start_advertising(
                                adv_params.clone(), adv_data.clone(), Some(scan_rsp.clone()),
                            ).await.expect("restart advertising");
                        }
                        _ => {}
                    }
                }

                match &event {
                    Event::Vendor(VendorEvent::GattAttributeModified(attr)) => {
                        if is_cccd_handle(tx_char_handle.0, attr.attr_handle.0) {
                            tx_notifications = CccdValue::from_bytes(attr.data()).notifications;
                            if tx_notifications {
                                if let Some(conn) = conn_handle {
                                    // Replay any output buffered while disconnected.
                                    if !pending_output.is_empty() {
                                        info!("replaying {} buffered item(s) ({} bytes)", pending_output.len(), pending_bytes);
                                        pending_bytes = 0;
                                        for item in pending_output.drain(..) {
                                            for chunk in item.chunks(MAX_DATA_LEN) {
                                                let _ = gatt.notify(conn, service_handle, tx_char_handle, chunk);
                                            }
                                        }
                                    }
                                    // Check if the firmware was reset due to OOM in the previous eval.
                                    // Safety: single-core; only written by panic handler before reset.
                                    let prev_cause = unsafe { RESET_CAUSE };
                                    if prev_cause == RESET_CAUSE_OOM {
                                        unsafe { RESET_CAUSE = 0; }
                                        warn!("OOM reset in previous eval — notifying client");
                                        let _ = gatt.notify(conn, service_handle, tx_char_handle,
                                            b"err: out of memory (script aborted, firmware reset)\r\n");
                                    } else if prev_cause == RESET_CAUSE_STACKOVERFLOW {
                                        unsafe { RESET_CAUSE = 0; }
                                        warn!("stack overflow reset in previous eval — notifying client");
                                        let _ = gatt.notify(conn, service_handle, tx_char_handle,
                                            b"err: stack overflow (too many recursive calls, firmware reset)\r\n");
                                    } else if prev_cause == RESET_CAUSE_PANIC {
                                        unsafe { RESET_CAUSE = 0; }
                                        warn!("firmware panic reset — notifying client");
                                        let _ = gatt.notify(conn, service_handle, tx_char_handle,
                                            b"err: firmware panic (reset)\r\n");
                                    }
                                    let _ = gatt.notify(conn, service_handle, tx_char_handle,
                                        b"Rhai Playground ready\r\n> help() for API list\r\n> ");
                                }
                            }
                            info!("TX notifications {}", if tx_notifications { "on" } else { "off" });
                        } else if is_value_handle(rx_char_handle.0, attr.attr_handle.0) {
                            let data = attr.data();
                            let room = MAX_SCRIPT_BYTES.saturating_sub(input_buf.len());
                            if room == 0 {
                                warn!("script buffer full ({} bytes)", MAX_SCRIPT_BYTES);
                            } else {
                                input_buf.extend_from_slice(&data[..data.len().min(room)]);
                                debug!("buffered {} bytes total", input_buf.len());
                            }
                        }
                    }

                    Event::Vendor(VendorEvent::AttExchangeMtuResponse(AttExchangeMtuResponse {
                        conn_handle: ch,
                        server_rx_mtu,
                    })) => {
                        if let Some(conn) = ble.get_connection_mut(*ch) {
                            conn.update_mtu(*server_rx_mtu as u16);
                        }
                        info!("ATT MTU exchanged: server_rx_mtu={} → max write payload={}", server_rx_mtu, server_rx_mtu.saturating_sub(3));
                    }

                    _ => {}
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Main — hardware init, spawn platform runners + tasks, then idle
// ---------------------------------------------------------------------------

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(core::ptr::addr_of!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();
    let p = embassy_stm32::init(config);

    // NOTE: MSPLIM is intentionally NOT set here.
    //
    // The BLE sequencer context-switch (context.rs) loads MSP directly from BSS
    // (the 32 KB sequencer stack lives below __ebss).  Any MSPLIM value high
    // enough to guard the thread stack would be above task_sp.  When an interrupt
    // fires while MSP == task_sp the hardware's exception-entry push violates
    // MSPLIM → STKOF fault → HardFault with stacked PC = 0x00000000 (the
    // exception frame was never written to the zeroed BSS, so the debugger reads
    // back zero).  This is exactly the crash that appeared once MSPLIM was added.
    //
    // Stack-overflow detection is instead done via a software MSP check in the
    // Rhai on_progress hook: on_progress reads the MSP register and aborts the
    // script before the stack reaches _stack_end (bottom of thread stack).
    // STACK_GUARD_LIMIT is computed once here from the linker symbol.
    {
        let limit = core::ptr::addr_of!(_stack_end) as u32 + STACK_GUARD_MARGIN;
        STACK_GUARD_LIMIT.store(limit, Ordering::Relaxed);
        info!("stack guard: limit=0x{:08X} (_stack_end+{} KB)", limit, STACK_GUARD_MARGIN / 1024);
    }

    let led_bank: &'static mut LedBank = LED_CELL.init(LedBank::new(
        Output::new(p.PD8, Level::High, Speed::Low),
        Output::new(p.PD9, Level::High, Speed::Low),
        Output::new(p.PB10, Level::High, Speed::Low),
    ));
    LED_BANK.lock(|cell| {
        *cell.borrow_mut() = Some(unsafe { core::ptr::read(led_bank) });
    });

    info!("{} BLE Rhai playground starting", board::BOARD_NAME);

    let oled = OledBus::new(p.SPI3, p.PA0, p.PB8, p.PE1, p.PE0, p.PE3);
    let adc = Adc::new_adc4(p.ADC4);
    let joy_pin = p.PA3;

    spawner.spawn(display_task(oled).expect("spawn display_task"));

    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Pka::new(p.PKA, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        4  // HCI channel packet slots; 4 is enough for NUS, saves ~1.1 KB BSS vs 8
    );

    // Make runtime 'static so HCI (and therefore ble_task) can be 'static.
    // new_platform! already stores runtime in a StaticCell internally, so
    // runtime is &'static mut FullRuntime — no extra cell needed here.

    // ble_runner_task must run at interrupt priority so it can preempt eval_task
    // during engine.eval(). Without this, the thread executor is monopolised by
    // eval and the BLE M0+ supervision timer fires → client disconnects mid-script.
    // Start the interrupt executor now (before HCI::new) so ble_runner_task is
    // already live when HCI::new() awaits BLE stack init events.
    interrupt::ICACHE.set_priority(Priority::P4);
    let ble_spawner = EXECUTOR_BLE.start(interrupt::ICACHE);
    ble_spawner.spawn(unwrap!(ble_runner_task(platform)));

    // BLE stack init (blocking-async) — must complete before spawning eval_task
    // to avoid concurrent heap allocation corrupting BLE internal state.
    let ble: HCI<'static, Normal> = HCI::new(platform, runtime, Irqs)
        .await
        .expect("BLE init failed");
    embassy_futures::yield_now().await;

    ble_spawner.spawn(unwrap!(ble_task(ble)));
    ble_spawner.spawn(unwrap!(input_task(adc, joy_pin)));

    spawner.spawn(eval_task().expect("spawn eval_task"));

    // Thread executor is now only needed for eval_task; park main here.
    loop { core::future::pending::<()>().await; }
}
