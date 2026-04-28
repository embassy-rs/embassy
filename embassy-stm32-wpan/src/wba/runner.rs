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

use core::future::poll_fn;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_futures::select::{Either, select};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_time::Timer;

use super::{linklayer_plat, util_seq};

// BleStack_Process return values
pub(crate) const BLE_SLEEPMODE_RUNNING: u8 = 0;

/// Ble runner task initialized
pub(crate) static BLE_INIT: AtomicBool = AtomicBool::new(false);

/// Ble init waker
pub(crate) static BLE_INIT_WAKER: AtomicWaker = AtomicWaker::new();

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
///     // Spawn the runner
///     spawner.spawn(ble_runner_task()).unwrap();
///     // Initialize the BLE controller...
///     
///     let controller = Controller::new(..).await.unwrap();
///     
///     // Your BLE application logic...
/// }
/// ```
pub async fn ble_runner() -> ! {
    info!("BLE runner started; waiting for BLE init");
    poll_fn(|cx| {
        BLE_INIT_WAKER.register(cx.waker());

        if BLE_INIT.load(Ordering::Acquire) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await;

    info!("BLE runner execution started");

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
