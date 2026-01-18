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

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Duration;

use super::context;
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

/// Maximum time to wait when idle before checking again (milliseconds)
const MAX_IDLE_PERIOD_MS: u64 = 10;

/// Signal used to wake the runner when there's BLE work to do
static RUNNER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// Signal to trigger BleStack_Process (equivalent to Sidewalk SDK's BleHostSemaphore)
static BLE_PROCESS_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// Wake the BLE process (call this after HCI commands, radio events, etc.)
pub fn wake_ble_process() {
    BLE_PROCESS_SIGNAL.signal(());
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
        context::sequencer_resume();

        LL_INIT_COMPLETED.store(true, Ordering::Release);

        #[cfg(feature = "defmt")]
        defmt::trace!("BLE runner: sequencer context initialized");
    }

    loop {
        // Check if there's sequencer work to do
        let has_work =
            util_seq::has_pending_tasks() || util_seq::has_pending_events() || context::sequencer_has_pending_work();

        if has_work {
            // Resume the sequencer context
            // This will run BLE stack tasks until the sequencer yields
            context::sequencer_resume();

            // After link layer scheduling, trigger BleStack_Process
            // Per ST docs: "BleStack_Process shall be called after Link Layer has been scheduled"
            wake_ble_process();
        }

        // Wait for BleStack_Process signal with timeout
        // This is equivalent to the Sidewalk SDK's osSemaphoreAcquire(BleHostSemaphore)
        let signaled = embassy_time::with_timeout(Duration::from_millis(100), BLE_PROCESS_SIGNAL.wait()).await;

        // Only process BLE host stack events when signaled, not on timeout
        // This prevents interfering with advertising timing
        if signaled.is_ok() {
            // Process BLE host stack events
            // This generates HCI events which call BLECB_Indication
            process_ble_stack();
        }

        // Yield to embassy executor
        embassy_futures::yield_now().await;
    }
}

/// Signal-based BLE runner function
///
/// This version is more power-efficient as it waits on a signal rather than
/// polling. The radio interrupt handlers automatically signal this runner.
///
/// # Example
///
/// ```no_run
/// use embassy_executor::Spawner;
/// use embassy_stm32_wpan::wba::runner::ble_runner_signaled;
///
/// #[embassy_executor::task]
/// async fn ble_runner_task() {
///     ble_runner_signaled().await
/// }
/// ```
pub async fn ble_runner_signaled() -> ! {
    #[cfg(feature = "defmt")]
    defmt::info!("BLE signaled runner started");

    loop {
        // Wait for signal or timeout
        let timeout = embassy_time::with_timeout(Duration::from_millis(MAX_IDLE_PERIOD_MS), RUNNER_SIGNAL.wait()).await;

        // Clear the signal (reset for next time)
        RUNNER_SIGNAL.reset();

        // Check if there's work (either we were signaled or timed out)
        let has_work =
            util_seq::has_pending_tasks() || util_seq::has_pending_events() || context::sequencer_has_pending_work();

        if has_work || timeout.is_ok() {
            // Resume the sequencer
            context::sequencer_resume();

            // Call BleStack_Process to process BLE host stack events
            unsafe {
                let mut iterations = 0;
                while BleStack_Process() == BLE_SLEEPMODE_RUNNING {
                    iterations += 1;
                    if iterations > 100 {
                        break;
                    }
                }
            }

            // Yield to let other tasks run
            embassy_futures::yield_now().await;
        }
    }
}

/// Wake the BLE runner
///
/// Call this from interrupt handlers when BLE events occur.
/// This is automatically called by the radio interrupt handlers.
pub fn wake_runner() {
    RUNNER_SIGNAL.signal(());
}

/// Integrate with the link layer ISR to wake the runner
///
/// This should be called from the radio interrupt handler.
pub fn on_radio_interrupt() {
    context::sequencer_wake();
    wake_runner();
}
