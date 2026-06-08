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
//! use embassy_stm32_wpan::{new_platform, Platform}
//!
//! /// RNG runner task
//! #[embassy_executor::task]
//! async fn rng_runner_task(platform: &'static Platform) {
//!     platform.run_rng().await
//! }
//!
//! /// BLE runner task - drives the BLE stack sequencer
//! #[embassy_executor::task]
//! async fn ble_runner_task(platform: &'static Platform) {
//!     platform.run_ble().await
//! }
//!
//! #[embassy_executor::main]
//! async fn main(spawner: Spawner) {
//!     // Initialize hardware peripherals required by BLE stack
//!     let (platform, runtime) = new_platform!(
//!         Rng::new(p.RNG, Irqs),
//!         Aes::new_blocking(p.AES, Irqs),
//!         Pka::new_blocking(p.PKA, Irqs),
//!         8
//!     );
//!     info!("Hardware peripherals initialized (RNG, AES, PKA)");
//!     // Spawn the RNG runner task
//!     spawner.spawn(rng_runner_task(platform).expect("Failed to spawn rng runner"));
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
use embassy_stm32::aes::Aes;
use embassy_stm32::low_power::ResumablePeripheral;
use embassy_stm32::mode::{Async, Blocking};
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph, RNG};
use embassy_stm32::pka::Pka;
use embassy_stm32::rng::Rng;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::pipe::Pipe;
use embassy_sync::signal::Signal;
use embassy_sync::zerocopy_channel::Channel;
use embassy_time::Timer;

use crate::ChannelPacket;
use crate::util::Flag;
use crate::wba::{BasicRuntime, FullRuntime, linklayer_plat, util_seq};

pub struct Platform {
    channel: UnsafeCell<Channel<'static, CriticalSectionRawMutex, ChannelPacket>>,
    rng_pipe: Pipe<CriticalSectionRawMutex, 256>,
    p256_req: Signal<CriticalSectionRawMutex, ([u32; 8], [u32; 8], [u32; 8])>,
    p256_resp: Signal<CriticalSectionRawMutex, ([u32; 8], [u32; 8])>,
    ble_init: Flag,
    rng: Mutex<CriticalSectionRawMutex, ResumablePeripheral<Rng<'static, RNG>>>,
    pka: Mutex<CriticalSectionRawMutex, Option<ResumablePeripheral<Pka<'static, PkaPeriph, Async>>>>,
    aes: Mutex<CriticalSectionRawMutex, Option<Aes<'static, AesPeriph, Blocking>>>,
}

// SAFETY: Every field except `channel` already implements Sync via
// CriticalSection-backed wrappers.  `channel` is `UnsafeCell<Channel<...>>`
// where the inner Channel uses CriticalSectionRawMutex for all state.
// `get_channel()` is `unsafe`, `pub(crate)`, and called exactly once during
// initialisation — no aliased `&mut` is ever produced at runtime.
// On single-core Cortex-M the critical-section protects all cross-context
// access, so sharing `&'static Platform` across executors is safe.
unsafe impl Sync for Platform {}

impl Platform {
    pub fn new_basic<const N: usize>(
        buf: &'static mut [ChannelPacket; N],
        rng: Rng<'static, RNG>,
    ) -> (Self, BasicRuntime) {
        (
            Self {
                channel: UnsafeCell::new(Channel::new(buf)),
                rng_pipe: Pipe::new(),
                p256_req: Signal::new(),
                p256_resp: Signal::new(),
                ble_init: Flag::new(false),
                rng: Mutex::new(ResumablePeripheral::new(rng)),
                pka: Mutex::new(None),
                aes: Mutex::new(None),
            },
            BasicRuntime { _private: () },
        )
    }

    pub fn new_full<const N: usize>(
        buf: &'static mut [ChannelPacket; N],
        rng: Rng<'static, RNG>,
        pka: Pka<'static, PkaPeriph, Async>,
        aes: Aes<'static, AesPeriph, Blocking>,
    ) -> (Self, FullRuntime) {
        (
            Self {
                channel: UnsafeCell::new(Channel::new(buf)),
                rng_pipe: Pipe::new(),
                p256_req: Signal::new(),
                p256_resp: Signal::new(),
                ble_init: Flag::new(false),
                rng: Mutex::new(ResumablePeripheral::new(rng)),
                pka: Mutex::new(Some(ResumablePeripheral::new(pka))),
                aes: Mutex::new(Some(aes)),
            },
            FullRuntime { _private: () },
        )
    }

    pub(crate) unsafe fn get_channel(
        &'static self,
    ) -> &'static mut Channel<'static, CriticalSectionRawMutex, ChannelPacket> {
        unsafe { &mut *self.channel.get() }
    }

    pub(crate) fn get_p256_req(&self) -> &Signal<CriticalSectionRawMutex, ([u32; 8], [u32; 8], [u32; 8])> {
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
    /// The pipe is fed by [`Self::run_rng`] using the same hardware RNG that
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

    /// Borrow the shared PKA peripheral for the duration of `f`.
    ///
    /// Panics if the platform was built with [`Self::new_basic`] (no PKA).
    pub async fn borrow_pka<R>(&self, f: impl AsyncFnOnce(&mut Pka<'static, PkaPeriph, Async>) -> R) -> R {
        let mut guard = self.pka.lock().await;
        let pka = guard.as_mut().unwrap();

        f(&mut *pka.borrow()).await
    }

    /// Borrow the shared AES peripheral for the duration of `f`.
    ///
    /// Panics if the platform was built with [`Self::new_basic`] (no AES).
    pub fn borrow_aes<R>(&self, f: impl FnOnce(&mut Aes<'static, AesPeriph, Blocking>) -> R) -> R {
        let mut guard = self.aes.try_lock().unwrap();
        let aes = guard.as_mut().unwrap();

        f(&mut *aes)
    }

    pub async fn run_ble(&'static self) -> ! {
        join3(
            async {
                loop {
                    let (k, px, py) = self.p256_req.wait().await;
                    let mut guard = self.pka.lock().await;
                    let pka = guard.as_mut().unwrap();

                    let mut rx = [0u32; 8];
                    let mut ry = [0u32; 8];

                    linklayer_plat::pka_p256_mul(&mut *pka.borrow(), &k, &px, &py, &mut rx, &mut ry).await;

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
                    {
                        #[allow(unused_mut)]
                        let mut guard = rng.borrow();
                        'outer: loop {
                            n = 0;
                            if let Err(e) = guard.async_fill_bytes(&mut buf).await {
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
        static RUNTIME: ::static_cell::StaticCell<::embassy_stm32_wpan::BasicRuntime> =
            ::static_cell::StaticCell::new();

        let (platform, runtime) = ::embassy_stm32_wpan::Platform::new_basic(
            EVENT_BUFFER.init([::embassy_stm32_wpan::ChannelPacket::default(); $size]),
            $rng,
        );

        (
            PLATFORM.init(platform) as &'static ::embassy_stm32_wpan::Platform,
            RUNTIME.init(runtime),
        )
    }};
    ($rng:expr, $pka:expr, $aes:expr, $size:expr) => {{
        static EVENT_BUFFER: ::static_cell::StaticCell<[::embassy_stm32_wpan::ChannelPacket; $size]> =
            ::static_cell::StaticCell::new();
        static PLATFORM: ::static_cell::StaticCell<::embassy_stm32_wpan::Platform> = ::static_cell::StaticCell::new();
        static RUNTIME: ::static_cell::StaticCell<::embassy_stm32_wpan::FullRuntime> = ::static_cell::StaticCell::new();

        let (platform, runtime) = ::embassy_stm32_wpan::Platform::new_full(
            EVENT_BUFFER.init([::embassy_stm32_wpan::ChannelPacket::default(); $size]),
            $rng,
            $pka,
            $aes,
        );

        (
            PLATFORM.init(platform) as &'static ::embassy_stm32_wpan::Platform,
            RUNTIME.init(runtime),
        )
    }};
}
