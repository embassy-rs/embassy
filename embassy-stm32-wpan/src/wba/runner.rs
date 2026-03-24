//! BLE Stack Runner for Embassy Integration
//!
//! This module provides the runner that drives the BLE sequencer while
//! integrating properly with the embassy async executor.
//!
//! # Architecture
//!
//! The BLE stack runs in a separate context (with its own stack) managed by
//! the context switching module. The runner:
//!
//! 1. Resumes the sequencer context
//! 2. The sequencer processes pending tasks (including BleStack_Process_BG)
//! 3. When idle, the sequencer yields back
//! 4. The runner yields to the embassy executor
//! 5. When woken (by interrupt), repeats from step 1
//!
//! # Usage
//!
//! The runner must be spawned as a separate embassy task:
//!
//! ```no_run
//! use embassy_executor::Spawner;
//! use embassy_stm32_wpan::wba::ble_runner;
//!
//! #[embassy_executor::task]
//! async fn ble_task() {
//!     ble_runner().await
//! }
//!
//! #[embassy_executor::main]
//! async fn main(spawner: Spawner) {
//!     // Initialize BLE stack first...
//!
//!     // Spawn the BLE runner task
//!     spawner.spawn(ble_task()).unwrap();
//!
//!     // Your application logic...
//! }
//! ```

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_futures::select::{Either, select};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_time::Timer;

use super::bindings::mac;
use super::{linklayer_plat, util_seq};

// BleStack_Process return values
const BLE_SLEEPMODE_RUNNING: u8 = 0;

// Task ID for BLE Host processing (next available after CFG_TASK_NBR=9)
const CFG_TASK_BLE_HOST: u32 = 9;
const TASK_BLE_HOST_MASK: u32 = 1 << CFG_TASK_BLE_HOST;
const TASK_PRIO_BLE_HOST: u32 = 0; // CFG_SEQ_PRIO_0

// Link Layer background task
const TASK_LINK_LAYER_MASK: u32 = 1 << mac::CFG_TASK_ID_T_CFG_TASK_LINK_LAYER;

// External BLE stack process function
#[link(name = "stm32wba_ble_stack_basic")]
unsafe extern "C" {
    /// BLE stack process function - must be called to process BLE events
    fn BleStack_Process() -> u8;
}

/// BLE stack background processing task, registered as a sequencer task.
///
/// Matches ST's BleStack_Process_BG:
///   - Calls BleStack_Process() once
///   - If it returns 0 (more work pending), re-queues via BleStackCB_Process
///   - If non-zero (idle/can sleep), does NOT re-queue
///
/// IMPORTANT: This runs on the sequencer's stack context, matching the
/// C reference implementation where BleStack_Process is a UTIL_SEQ task.
unsafe extern "C" fn ble_stack_process_bg() {
    let result = BleStack_Process();

    #[cfg(feature = "defmt")]
    defmt::trace!("BleStack_Process called, result={}", result);

    if result == BLE_SLEEPMODE_RUNNING {
        // More work to do - re-queue
        ble_stack_cb_process();
    }
}

/// Matches ST's BleStackCB_Process: re-queues BleStack_Process_BG via the sequencer.
fn ble_stack_cb_process() {
    util_seq::UTIL_SEQ_SetTask(TASK_BLE_HOST_MASK, TASK_PRIO_BLE_HOST);
}

/// Whether the link layer init has been completed
static LL_INIT_COMPLETED: AtomicBool = AtomicBool::new(false);

/// Signal to wake the runner loop (set by radio ISR and event callbacks)
pub(crate) static BLE_WAKER: AtomicWaker = AtomicWaker::new();

/// Register BLE stack tasks with the sequencer.
///
/// Registers BleStack_Process_BG as a sequencer task, matching the C pattern:
///   UTIL_SEQ_RegTask(1U << CFG_TASK_BLE_HOST, UTIL_SEQ_RFU, BleStack_Process_BG);
pub fn register_ble_tasks() {
    util_seq::UTIL_SEQ_RegTask(TASK_BLE_HOST_MASK, 0, Some(ble_stack_process_bg));

    #[cfg(feature = "defmt")]
    defmt::trace!(
        "Registered BleStack_Process_BG as sequencer task (mask=0x{:08X})",
        TASK_BLE_HOST_MASK
    );
}

/// Schedule the BLE Host task to run.
///
/// Queues the BLE Host sequencer task and wakes the runner.
/// Call this after HCI events arrive or whenever BLE stack processing is needed.
pub fn schedule_ble_host_task() {
    ble_stack_cb_process();
    BLE_WAKER.wake();

    #[cfg(feature = "defmt")]
    defmt::trace!("BLE Host task scheduled");
}

/// BLE stack runner function
///
/// This async function drives the BLE stack. It must be spawned as a task
/// to enable proper BLE operation.
///
/// # Example
///
/// ```no_run
/// use embassy_executor::Spawner;
/// use embassy_stm32_wpan::wba::ble_runner;
///
/// #[embassy_executor::task]
/// async fn ble_runner_task() {
///     ble_runner().await
/// }
///
/// #[embassy_executor::main]
/// async fn main(spawner: Spawner) {
///     // Initialize BLE...
///
///     // Spawn the runner
///     spawner.spawn(ble_runner_task()).unwrap();
///
///     // Your BLE application logic...
/// }
/// ```
pub async fn ble_runner() -> ! {
    #[cfg(feature = "defmt")]
    defmt::info!("BLE runner started");

    // Mark that the runner has started (BLE init is now done via init_ble_stack())
    if !LL_INIT_COMPLETED.load(Ordering::Acquire) {
        #[cfg(feature = "defmt")]
        defmt::trace!("BLE runner: first run, initializing sequencer context");

        // Do one context switch to initialize the sequencer
        util_seq::seq_resume();

        LL_INIT_COMPLETED.store(true, Ordering::Release);

        #[cfg(feature = "defmt")]
        defmt::trace!("BLE runner: sequencer context initialized");
    }

    // Schedule the initial tasks and kick the BLE stack.
    // BLE init and GAP setup happened before the runner started, so there may be
    // pending HCI commands that need BleStack_Process to deliver them to the LL.
    schedule_ble_host_task();
    util_seq::UTIL_SEQ_SetTask(TASK_LINK_LAYER_MASK, 0);
    util_seq::seq_resume();

    // Flush pending HCI commands through BleStack_Process.
    // This delivers scan enable, connection parameters, etc. to the LL.
    loop {
        let result = unsafe { BleStack_Process() };
        if result != BLE_SLEEPMODE_RUNNING {
            break;
        }
    }

    // Re-issue LE advertising enable to the LL.
    // ACI_GAP_SET_DISCOVERABLE configures parameters but does not enable
    // advertising in the LL on WBA6. This is safe to call even if advertising
    // wasn't set up (it will just return an error which we ignore).
    let _ = super::hci::command::le_set_advertising_enable(true);

    // Run the sequencer once more to process any LL events from the enable
    util_seq::UTIL_SEQ_SetTask(TASK_LINK_LAYER_MASK, 0);
    util_seq::seq_resume();

    loop {
        // Wait for either a sequencer event or a timer expiry
        match linklayer_plat::earliest_timer_deadline() {
            Some(deadline) => match select(util_seq::wait_for_event(), Timer::at(deadline)).await {
                Either::First(()) => {}
                Either::Second(()) => {
                    linklayer_plat::check_expired_timers();
                }
            },
            None => {
                util_seq::wait_for_event().await;
            }
        }

        // Check for any expired timers on each iteration
        linklayer_plat::check_expired_timers();

        // Resume the sequencer context
        util_seq::seq_resume();
    }
}

/// Integrate with the link layer ISR to wake the runner
///
/// This should be called from the radio interrupt handler.
pub fn on_radio_interrupt() {
    util_seq::seq_pend();
}
