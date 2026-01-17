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

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};

use super::context;
use super::util_seq;

/// Minimum time to wait between sequencer runs when idle (microseconds)
const MIN_IDLE_PERIOD_US: u64 = 100;

/// Maximum time to wait when idle before checking again (milliseconds)
const MAX_IDLE_PERIOD_MS: u64 = 10;

/// Signal used to wake the runner when there's BLE work to do
static RUNNER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

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

    loop {
        // Check if there's work to do
        let has_work = util_seq::has_pending_tasks()
            || util_seq::has_pending_events()
            || context::sequencer_has_pending_work();

        if has_work {
            // Resume the sequencer context
            // This will run BLE stack tasks until the sequencer yields
            context::sequencer_resume();

            // Small yield to let other high-priority tasks run
            embassy_futures::yield_now().await;
        } else {
            // No immediate work - wait a bit before checking again
            // This prevents busy-looping while still being responsive
            Timer::after(Duration::from_micros(MIN_IDLE_PERIOD_US)).await;
        }
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
        let timeout = embassy_time::with_timeout(
            Duration::from_millis(MAX_IDLE_PERIOD_MS),
            RUNNER_SIGNAL.wait(),
        )
        .await;

        // Clear the signal (reset for next time)
        RUNNER_SIGNAL.reset();

        // Check if there's work (either we were signaled or timed out)
        let has_work = util_seq::has_pending_tasks()
            || util_seq::has_pending_events()
            || context::sequencer_has_pending_work();

        if has_work || timeout.is_ok() {
            // Resume the sequencer
            context::sequencer_resume();

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
