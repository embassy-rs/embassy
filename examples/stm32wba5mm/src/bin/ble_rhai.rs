//! BLE Rhai Interpreter Demo
//!
//! Receives Rhai scripts over BLE NUS (Nordic UART Service). After 500 ms of
//! idle time the accumulated input is dispatched to a dedicated eval task.
//! The eval task runs the Rhai engine and sends the result back over a channel;
//! the BLE task then notifies the client without ever blocking on eval.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐   ┌──────────────────────────────┐
//! │  main / BLE task                    │   │  eval_task                   │
//! │                                     │   │                              │
//! │  select3(                           │   │  owns: Engine, LED pin (PA1) │
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
use core::sync::atomic::{AtomicBool, Ordering};
// defmt::* would bring in a `panic_handler` proc-macro that conflicts with the
// built-in #[panic_handler] attribute used for our custom panic handler below.
// Import the macros we actually use explicitly instead of glob-importing.
use defmt::{debug, error, info, warn, unwrap};
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_futures::select::{Either3, Either4, select3, select4};
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::interrupt::{self, InterruptExt, Priority};
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rcc;
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts, peripherals};
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
use rhai::{Dynamic, Engine, packages::BasicIteratorPackage, packages::BasicMathPackage, packages::BasicStringPackage, packages::Package};
use static_cell::StaticCell;
use stm32wb_hci::Event;
use stm32wb_hci::vendor::event::{AttExchangeMtuResponse, VendorEvent};
use defmt_rtt as _; // RTT logging backend; panic handler is custom below (no panic_probe)
use cortex_m_rt::{exception, interrupt};

#[global_allocator]
static HEAP: Heap = Heap::empty();

// RAM layout (128 KB total):
//   BSS baseline (BLE stack + task state) ≈ 33 KB  (excl. HEAP_MEM)
//   data section (BLE blob init data)     ≈ 35 KB
//   HEAP_MEM (below)                      = 46 KB  (reduced from 50 KB to give ~14 KB stack)
//   ── total used ──────────────────────────── 114 KB
//   Stack (grows down from 0x20020000)    ≈ 14 KB
//
// Stack budget is the critical constraint for Rhai user-defined functions.
// Each Rhai fn call consumes ~2-3 KB Rust stack (interpreter eval structs).
// With 14 KB stack: depth 5 is safe, depth 6 likely OK, depth ≥7 risky.
// Heap: engine init ~18 KB → ~28 KB free for eval (arrays, strings, results).
// CorePackage was tried but its init uses too much stack (overflows into BLE BSS).
// Three focused packages (Math+Iterator+String) with only_i32 keep stack usage low.
const HEAP_SIZE: usize = 46 * 1024;

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

// Set while engine.eval() is running; lets the BLE task detect an in-progress eval on disconnect.
static EVAL_RUNNING: AtomicBool = AtomicBool::new(false);

// Set by the on_progress heap-guard when free heap drops below MIN_HEAP_RESERVE.
// Cleared atomically in the error handler so the eval loop can report a clean
// "out of memory" message instead of the opaque Rhai termination error.
static OOM_TERMINATED: AtomicBool = AtomicBool::new(false);

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
// free heap drops below this threshold. The reserve serves two purposes:
//
//  1. Post-eval cleanup: after OOM termination, alloc::format!() and
//     result_buf.extend_from_slice() still need a few hundred bytes.
//
//  2. In-step print() buffer: on_print allocates Vec::with_capacity(s.len()+2)
//     inside a Rhai step (not between steps), so on_progress cannot intercept it.
//     Keeping 6 KB in reserve ensures there is room for a reasonably-sized
//     print() call even in the step immediately after the last on_progress check.
//     This is a heuristic, not a hard guarantee — extremely large print strings
//     could still exhaust the reserve within a single step.
const MIN_HEAP_RESERVE: usize = 6 * 1024;

// LED pin shared between the Rhai led() closure (needs 'static) and eval_task.
static LED_PIN: Mutex<CriticalSectionRawMutex, RefCell<Option<Output<'static>>>> =
    Mutex::new(RefCell::new(None));

// Used to give PA1 a 'static lifetime so it can live in LED_PIN.
static LED_CELL: StaticCell<Output<'static>> = StaticCell::new();

// FullRuntime is stored inside new_platform!'s internal StaticCell — already 'static.
// EXECUTOR_BLE: high-priority interrupt executor for ble_runner_task + ble_task.
// Both run at Priority::P4 so they preempt eval_task (thread executor) during
// engine.eval(), keeping the BLE connection and HCI runner alive.
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
async fn rng_runner_task(platform: &'static Platform) {
    platform.run_rng().await
}

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

    engine.register_fn("led", |state: bool| -> bool {
        LED_PIN.lock(|cell| {
            if let Some(pin) = cell.borrow_mut().as_mut() {
                pin.set_level(if state { Level::High } else { Level::Low });
            }
        });
        state
    });

    engine.register_fn("ts", || { // timestamp in ticks (32768 Hz); i32 with only_i32
        embassy_time::Instant::now().as_ticks() as i32
    });

    // Each print()/debug() call sends immediately to PRINT_CHAN.
    // ble_task runs at interrupt priority and will preempt eval_task to forward
    // the notification to the BLE client without waiting for eval to finish.
    engine.on_print(|s| {
        let mut v: Vec<u8> = Vec::with_capacity(s.len() + 2);
        v.extend_from_slice(s.as_bytes());
        v.push(b'\r');
        v.push(b'\n');
        info!("print: {}", s);
        let _ = PRINT_CHAN.try_send(v); // non-blocking; drops if channel full

    });
    engine.on_debug(|s, _src, _pos| {
        let mut v: Vec<u8> = Vec::with_capacity(s.len() + 2);
        v.extend_from_slice(s.as_bytes());
        v.push(b'\r');
        v.push(b'\n');
        info!("debug: {}", s);
        let _ = PRINT_CHAN.try_send(v);
    });

    // Rhai safety limits — enforced BEFORE the allocation attempt, so violations
    // return a clean EvalAltResult error instead of an allocator panic.
    //
    // set_max_array_size: 512 elements maximum.
    // Heap budget analysis (50 KB total, ~18 KB engine init = ~32 KB eval room):
    //   A Rhai Dynamic is 16 bytes on 32-bit. 512 elements = 8 KB in the array.
    //   Rhai checks the size limit BEFORE calling Vec::push, so when mask has
    //   512 elements (capacity 512), the 513th push is intercepted first —
    //   no Vec growth (512→1024 = 16 KB) is ever attempted.
    //   Setting it to 1024 leaves one doubling step (1024→2048 = 32 KB) within
    //   a single Rhai step where the check cannot intervene — that causes OOM.
    //
    // set_max_string_size: 8 KB cap against runaway string building.
    //
    // set_max_operations + on_progress: belt-and-suspenders for non-array OOM
    // and infinite loops. on_progress fires between Rhai steps so it cannot
    // stop an in-step Vec realloc — the size limits above are the primary guard.
    engine.set_max_array_size(512);
    engine.set_max_string_size(8192);
    engine.set_max_operations(500_000);
    // Soft call-stack depth limit (secondary guard).
    // MSPLIM is the primary hardware guard: it fires a UsageFault the moment
    // MSP crosses the protected address, BEFORE RAM is corrupted. This soft
    // limit exists as a belt-and-suspenders check that delivers a clean Rhai
    // error message for moderate recursion, without involving a firmware reset.
    //
    // With 14 KB stack and ~2-3 KB per Rhai call level, MSPLIM fires around
    // depth 4-5 anyway. Set this to 12 so normal scripts aren't capped early;
    // deeply recursive scripts hit MSPLIM first and get the "firmware reset"
    // error message via RESET_CAUSE_STACKOVERFLOW.
    engine.set_max_call_levels(12);
    engine.on_progress(|_ops| {
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
            info!("eval: {} bytes  heap_free={} B\n{}", script_bytes.len(), HEAP.free(), script);

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
                    if OOM_TERMINATED.swap(false, Ordering::Relaxed) {
                        let free = HEAP.free();
                        error!("eval OOM: {} B heap free (reserve {} B)", free, MIN_HEAP_RESERVE);
                        alloc::format!("err: out of memory ({} B heap free)\r\n", free)
                    } else {
                        warn!("eval err: {}", alloc::format!("{}", e).as_str());
                        alloc::format!("err: {}\r\n", e)
                    }
                }
            };
            EVAL_RUNNING.store(false, Ordering::Relaxed);

            result_buf.extend_from_slice(eval_result.as_bytes());
        }

        RESULT_CHAN.send(result_buf).await;
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
    adv_data.add_name("RhaiShell").unwrap();

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

    info!("Advertising as 'RhaiShell' — connect and send Rhai expressions");

    let mut input_buf: Vec<u8> = Vec::new();
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
                        pending_output.push(data);
                    }
                } else {
                    // No connection — buffer for replay on next reconnect.
                    pending_output.push(data);
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
                                    }
                                    let _ = gatt.notify(conn, service_handle, tx_char_handle, b"> ");
                                }
                            }
                            info!("TX notifications {}", if tx_notifications { "on" } else { "off" });
                        } else if is_value_handle(rx_char_handle.0, attr.attr_handle.0) {
                            input_buf.extend_from_slice(attr.data());
                            debug!("buffered {} bytes total", input_buf.len());
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

    // ---- Cortex-M33 hardware stack guard (MSPLIM) --------------------------------
    // Enable UsageFault (SHCSR bit 18 = USGFAULTENA) so a stack overflow triggers
    // our UsageFault handler instead of escalating silently to HardFault.
    // Then set MSPLIM to _ebss + .uninit size (~520 B) + 2 KB safety runway:
    //   - fault fires 2 KB BEFORE the stack pointer would overwrite static data
    //   - those 2 KB remain available for the UsageFault handler itself
    // With 46 KB heap, _ebss is ~0x2001_BD00; MSPLIM lands near 0x2001_C700,
    // leaving ~10 KB of safe stack for eval_task.
    unsafe {
        const SCB_SHCSR: *mut u32 = 0xE000_ED24 as *mut u32;
        core::ptr::write_volatile(
            SCB_SHCSR,
            core::ptr::read_volatile(SCB_SHCSR) | (1 << 18), // USGFAULTENA
        );
        unsafe extern "C" { static __ebss: u8; }
        // __ebss = end of .bss (cortex-m-rt symbol); .uninit follows (~520 B: RTT buf 512 + RESET_CAUSE 4 + pad)
        let limit = core::ptr::addr_of!(__ebss) as u32 + 520 + 2048;
        cortex_m::register::msplim::write(limit);
        info!("stack guard: MSPLIM=0x{:08X}", limit);
    }
    // -------------------------------------------------------------------------------
    let led: &'static mut Output<'static> = LED_CELL.init(Output::new(p.PA1, Level::Low, Speed::Low));
    LED_PIN.lock(|cell| { *cell.borrow_mut() = unsafe { Some(core::ptr::read(led)) }; });

    info!("BLE Rhai interpreter starting");

    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        Pka::new_blocking(p.PKA, Irqs),
        4  // HCI channel packet slots; 4 is enough for NUS, saves ~1.1 KB BSS vs 8
    );

    // Make runtime 'static so HCI (and therefore ble_task) can be 'static.
    // new_platform! already stores runtime in a StaticCell internally, so
    // runtime is &'static mut FullRuntime — no extra cell needed here.

    spawner.spawn(rng_runner_task(platform).expect("spawn rng"));

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

    spawner.spawn(eval_task().expect("spawn eval_task"));

    // Thread executor is now only needed for eval_task; park main here.
    loop { core::future::pending::<()>().await; }
}
