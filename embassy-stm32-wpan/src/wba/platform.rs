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
//! The BLE stack's cryptographic operations (AES-ECB/CMAC/CCM, P-256 scalar
//! multiplication) are served by [`embassy-crypto`]. This crate only calls the
//! `embassy-crypto` API; the final binary selects the drivers, for example by
//! enabling the matching `embassy-crypto-*` features of `embassy-stm32` or by
//! depending on `embassy-crypto-rustcrypto`. Exactly one driver must be
//! registered for each of:
//!
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
//! use embassy_stm32::rng::{self, Rng};
//! use embassy_stm32_wpan::{new_platform, Platform};
//!
//! /// BLE runner task - drives the BLE stack sequencer and the shared RNG pipe
//! #[embassy_executor::task]
//! async fn ble_runner_task(platform: &'static Platform) {
//!     platform.run_ble().await
//! }
//!
//! #[embassy_executor::main]
//! async fn main(spawner: Spawner) {
//!     // Initialize hardware peripherals required by the BLE stack
//!     let (platform, runtime) = new_platform!(Rng::new(p.RNG, Irqs), 8);
//!     info!("Hardware peripherals initialized (RNG)");
//!     // Spawn the BLE runner task (required for proper BLE operation)
//!     spawner.spawn(ble_runner_task(platform).expect("Failed to spawn BLE runner"));
//!
//!     // Your application logic...
//! }
//! ```

use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::pin::pin;
use core::task::Poll;

use embassy_futures::join::join3;
use embassy_futures::select::select;
use embassy_stm32::rng::Rng;
use embassy_stm32::suspend::ResumablePeripheral;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::pipe::Pipe;
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
    rng_pipe: Pipe<CriticalSectionRawMutex, 256>,
    p256_req: Signal<CriticalSectionRawMutex, P256Request>,
    p256_resp: Signal<CriticalSectionRawMutex, ([u32; 8], [u32; 8])>,
    ble_init: Flag,
    rng: Mutex<CriticalSectionRawMutex, ResumablePeripheral<Rng<'static, embassy_stm32::mode::Async>>>,
}

impl Platform {
    pub fn new<const N: usize>(
        buf: &'static mut [ChannelPacket; N],
        rng: Rng<'static, embassy_stm32::mode::Async>,
    ) -> (Self, Runtime) {
        (
            Self {
                channel: UnsafeCell::new(Channel::new(buf)),
                rng_pipe: Pipe::new(),
                p256_req: Signal::new(),
                p256_resp: Signal::new(),
                ble_init: Flag::new(false),
                rng: Mutex::new(ResumablePeripheral::new(rng)),
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

    pub(crate) async fn wait_rng_ready(&self) {
        self.rng_pipe.wait_full().await
    }

    /// Fill `buf` from the pipe, returning how many bytes were actually written.
    /// May return less than `buf.len()` if the pipe is transiently low.
    pub(crate) fn try_fill_bytes(&self, buf: &mut [u8]) -> usize {
        self.rng_pipe.try_read(buf).unwrap_or(0)
    }

    /// Fill `buf` with random bytes from the BLE platform's RNG pipe.
    ///
    /// The pipe is fed by [`Self::run_ble`] using the same hardware RNG that
    /// backs the BLE controller, so applications sharing this `Platform` for
    /// crypto don't need a second `Rng` instance.
    pub async fn fill_random_bytes(&self, mut buf: &mut [u8]) {
        // This implementation does not allow reducing the buffer capacity by more than 64 bytes
        let mut b;
        while !buf.is_empty() {
            let mut wait_full = pin!(self.rng_pipe.wait_full());

            let free_capacity = poll_fn(|cx| {
                // Poll the future in order to register the waker
                let free_capacity = match wait_full.as_mut().poll(cx) {
                    Poll::Ready(()) => 0,
                    Poll::Pending => self.rng_pipe.free_capacity(),
                };

                if free_capacity < 64 {
                    Poll::Ready(free_capacity)
                } else {
                    Poll::Pending
                }
            })
            .await;

            (b, buf) = buf.split_at_mut(buf.len().min(64 - free_capacity));

            self.rng_pipe.try_read(&mut b).unwrap();
        }
    }

    pub async fn run_ble(&'static self) -> ! {
        join3(
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
            async {
                let mut rng = self.rng.lock().await;

                loop {
                    let mut buf = [0u8; 64];
                    let mut n;
                    // The resume guard is held for the whole iteration, including the
                    // pipe write below. Dropping it suspends the peripheral — for the
                    // RNG that clears CR.RNGEN and gates its clock — and the BLE link
                    // layer polls this same RNG from interrupt context whenever the
                    // pipe runs dry. With the clock gated even a write to RNG_CR is
                    // dropped, so that fallback could never see DRDY.
                    #[allow(unused_mut)]
                    let mut guard = rng.borrow();
                    {
                        'outer: loop {
                            n = 0;
                            if let Err(e) = guard.fill_bytes(&mut buf).await {
                                warn!("rng: err during fill bytes: {}", e);

                                continue;
                            }

                            while n < buf.len() {
                                if let Ok(len) = self.rng_pipe.try_write(&buf) {
                                    n += len;
                                } else {
                                    break 'outer;
                                }
                            }
                        }
                    }

                    self.rng_pipe.write_all(&buf[n..]).await;
                }
            },
        )
        .await;

        loop {}
    }
}

#[macro_export]
macro_rules! new_platform {
    ($rng:expr, $size:expr) => {{
        static EVENT_BUFFER: ::static_cell::StaticCell<[::embassy_stm32_wpan::ChannelPacket; $size]> =
            ::static_cell::StaticCell::new();
        static PLATFORM: ::static_cell::StaticCell<::embassy_stm32_wpan::Platform> = ::static_cell::StaticCell::new();
        static RUNTIME: ::static_cell::StaticCell<::embassy_stm32_wpan::Runtime> = ::static_cell::StaticCell::new();

        let (platform, runtime) = ::embassy_stm32_wpan::Platform::new(
            EVENT_BUFFER.init([::embassy_stm32_wpan::ChannelPacket::default(); $size]),
            $rng,
        );

        (
            PLATFORM.init(platform) as &'static ::embassy_stm32_wpan::Platform,
            RUNTIME.init(runtime),
        )
    }};
}
