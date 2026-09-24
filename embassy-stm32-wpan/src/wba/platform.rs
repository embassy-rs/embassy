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
//! # Crypto
//!
//! The BLE stack's cryptographic operations (RNG, AES-ECB/CMAC/CCM, P-256
//! scalar multiplication) are served by [`embassy-crypto`]. This crate only
//! calls the `embassy-crypto` API; the final binary selects the drivers, for
//! example by enabling the matching `embassy-crypto-*` features of
//! `embassy-stm32` (the blocking RNG driver is registered by
//! `embassy-stm32`'s `embassy-crypto-rng` feature) or by depending on
//! `embassy-crypto-rustcrypto`. Exactly one driver must be registered for
//! each of:
//!
//! - `Rng` (blocking random bytes),
//! - `Aes128` (ECB), `Aes128Cmac` and `Aes128Ccm` (or `Aes256` variants are
//!   not used — the BLE stack is AES-128 only),
//! - `p256` arithmetic (`Scalar`/`Point`).
//!
//! The P-256 operations are deferred to the runner task: the BLE stack's
//! platform callbacks are invoked from the sequencer context, and the
//! completion callback (`BLEPLATCB_PkaComplete`) must be dispatched from
//! embassy task context, never re-entrantly from within `seq_resume`.
//!
//! # Usage
//!
//! The runner must be spawned as a separate embassy task:
//!
//! ```no_run
//! use embassy_executor::Spawner;
//! use embassy_stm32_wpan::{new_platform, Platform};
//!
//! /// BLE runner task - drives the BLE stack sequencer
//! #[embassy_executor::task]
//! async fn ble_runner_task(platform: &'static Platform) {
//!     platform.run_ble().await
//! }
//!
//! #[embassy_executor::main]
//! async fn main(spawner: Spawner) {
//!     // Initialize the platform; the RNG is served by the `embassy-crypto`
//!     // driver registered by the final binary.
//!     let (platform, runtime) = new_platform!(8);
//!     // Spawn the BLE runner task (required for proper BLE operation)
//!     spawner.spawn(ble_runner_task(platform).expect("Failed to spawn BLE runner"));
//!
//!     // Your application logic...
//! }
//! ```

use core::cell::UnsafeCell;

use embassy_futures::join::join;
use embassy_futures::select::select;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_sync::zerocopy_channel::Channel;
use embassy_time::Timer;

use crate::ChannelPacket;
use crate::util::Flag;
use crate::wba::{Runtime, linklayer_plat, util_seq};

/// A P-256 operation requested by the BLE stack, deferred to the runner task.
#[derive(Clone, Copy)]
pub(crate) enum P256Request {
    /// Compute the public key `k * G`.
    PublicKey { k: [u32; 8] },
    /// Compute the DH shared secret `k * P`.
    DhKey {
        k: [u32; 8],
        peer_x: [u32; 8],
        peer_y: [u32; 8],
    },
}

pub struct Platform {
    channel: UnsafeCell<Channel<'static, CriticalSectionRawMutex, ChannelPacket>>,
    p256_req: Signal<CriticalSectionRawMutex, P256Request>,
    p256_resp: Signal<CriticalSectionRawMutex, ([u32; 8], [u32; 8])>,
    ble_init: Flag,
}

impl Platform {
    pub fn new<const N: usize>(buf: &'static mut [ChannelPacket; N]) -> (Self, Runtime) {
        (
            Self {
                channel: UnsafeCell::new(Channel::new(buf)),
                p256_req: Signal::new(),
                p256_resp: Signal::new(),
                ble_init: Flag::new(false),
            },
            Runtime { _private: () },
        )
    }

    pub(crate) unsafe fn get_channel(
        &'static self,
    ) -> &'static mut Channel<'static, CriticalSectionRawMutex, ChannelPacket> {
        unsafe { &mut *self.channel.get() }
    }

    pub(crate) fn get_p256_req(&self) -> &Signal<CriticalSectionRawMutex, P256Request> {
        &self.p256_req
    }

    pub(crate) fn get_p256_resp(&self) -> &Signal<CriticalSectionRawMutex, ([u32; 8], [u32; 8])> {
        &self.p256_resp
    }

    pub(crate) fn start_run_ble(&self) {
        self.ble_init.set_high();
    }

    pub async fn run_ble(&'static self) -> ! {
        join(
            async {
                loop {
                    let req = self.p256_req.wait().await;

                    let (rx, ry) = linklayer_plat::p256_compute(&req);

                    self.p256_resp.signal((rx, ry));

                    // Dispatch deferred PKA callback (BLEPLATCB_PkaComplete) from embassy-task
                    // context. The BLE stack requires this to arrive asynchronously — calling
                    // it from within seq_resume (re-entrantly) corrupts the stack's state machine.
                    linklayer_plat::dispatch_pka_callback();
                }
            },
            async {
                info!("BLE runner started; waiting for BLE init");
                self.ble_init.wait_for_high().await;

                info!("BLE runner execution started");

                loop {
                    // Wait for either a sequencer event or a timer expiry
                    select(
                        util_seq::wait_for_event(),
                        Timer::at(linklayer_plat::earliest_timer_deadline()),
                    )
                    .await;

                    // Check for any expired timers on each iteration
                    linklayer_plat::check_expired_timers();

                    // Resume the sequencer context
                    util_seq::seq_resume();
                }
            },
        )
        .await;

        loop {}
    }
}

#[macro_export]
macro_rules! new_platform {
    ($size:expr) => {{
        static EVENT_BUFFER: ::static_cell::StaticCell<[::embassy_stm32_wpan::ChannelPacket; $size]> =
            ::static_cell::StaticCell::new();
        static PLATFORM: ::static_cell::StaticCell<::embassy_stm32_wpan::Platform> = ::static_cell::StaticCell::new();
        static RUNTIME: ::static_cell::StaticCell<::embassy_stm32_wpan::Runtime> = ::static_cell::StaticCell::new();

        let (platform, runtime) = ::embassy_stm32_wpan::Platform::new(EVENT_BUFFER.init(
            [::embassy_stm32_wpan::ChannelPacket::default(); $size],
        ));

        (
            PLATFORM.init(platform) as &'static ::embassy_stm32_wpan::Platform,
            RUNTIME.init(runtime),
        )
    }};
}
