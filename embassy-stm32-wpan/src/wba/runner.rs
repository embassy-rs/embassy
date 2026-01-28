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
//! 2. The sequencer processes pending tasks
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

use core::future::poll_fn;
use core::sync::atomic::{AtomicBool, Ordering, compiler_fence};
use core::task::Poll;

use embassy_futures::join;
use embassy_sync::waitqueue::AtomicWaker;

// Note: complete_ble_link_layer_init is now called as part of init_ble_stack()
// in Ble::init(), so we no longer need to call it from the runner.
use super::util_seq;

// BleStack_Process return values
const BLE_SLEEPMODE_RUNNING: u8 = 0;

// External BLE stack process function
#[link(name = "stm32wba_ble_stack_basic")]
unsafe extern "C" {
    /// BLE stack process function - must be called to process BLE events
    fn BleStack_Process() -> u8;
}

/// Call BleStack_Process until it returns CPU_HALT
/// Per ST docs: "When BleStack_Process returns BLE_SLEEPMODE_RUNNING, it shall be re-called"
fn process_ble_stack() {
    unsafe {
        let mut iterations = 0;
        loop {
            let result = BleStack_Process();

            #[cfg(feature = "defmt")]
            if iterations == 0 {
                defmt::trace!("BleStack_Process called, result={}", result);
            }

            if result != BLE_SLEEPMODE_RUNNING {
                // CPU can halt, no more work to do
                break;
            }

            iterations += 1;

            // Safety limit to prevent infinite loop
            if iterations > 1000 {
                #[cfg(feature = "defmt")]
                defmt::warn!("BleStack_Process called {} times, breaking to prevent hang", iterations);
                break;
            }
        }

        #[cfg(feature = "defmt")]
        if iterations > 10 {
            defmt::debug!("BleStack_Process completed after {} iterations", iterations);
        }
    }
}

/// Whether the link layer init has been completed
static LL_INIT_COMPLETED: AtomicBool = AtomicBool::new(false);

/// Signal to trigger BleStack_Process (equivalent to Sidewalk SDK's BleHostSemaphore)
pub(crate) static BLE_WAKER: AtomicWaker = AtomicWaker::new();

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

    join::join(
        async {
            loop {
                util_seq::wait_for_event().await;

                // Resume the sequencer context
                // This will run BLE stack tasks until the sequencer yields
                util_seq::seq_resume();
                BLE_WAKER.wake();
            }
        },
        poll_fn(|cx| {
            BLE_WAKER.register(cx.waker());
            compiler_fence(Ordering::Release);

            process_ble_stack();

            Poll::<()>::Pending
        }),
    )
    .await;

    loop {}
}

/// Integrate with the link layer ISR to wake the runner
///
/// This should be called from the radio interrupt handler.
pub fn on_radio_interrupt() {
    util_seq::seq_pend();
}
