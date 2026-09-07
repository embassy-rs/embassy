//! Cross-core [`RawMutex`](embassy_sync::blocking_mutex::raw::RawMutex)
//! backed by a hardware semaphore (HSEM).
//!
//! Locking semantics:
//!
//! * Contention from the *other* core spins (busy-wait).
//! * Re-entrant acquisition from the *same* core — another task, or an IRQ
//!   that preempted this core's lock holder — panics instead of deadlocking.
//!
//! # Why HSEM
//!
//! On the STM32H7 dual-core parts the HSEM peripheral is the only inter-core
//! lock that needs no cache management: the lock state lives in a peripheral
//! register (device memory, coherent by construction).
//!
//! The *data guarded* by this mutex must still live in memory both cores can
//! access coherently (e.g. SRAM4, mapped non-cacheable on the Cortex-M7). The
//! mutex itself needs nothing: no shared RAM, no MPU configuration.
//!
//! # How the same-core panic works
//!
//! The HSEM readback alone can **not** detect same-core re-entry: a lock
//! write to an already-locked semaphore is ignored (RM0399 §11.3), leaving a
//! readback identical to a fresh acquisition when the holder is this core
//! with the same PROCID. Instead, each mutex carries a `held` flag in
//! **core-private** memory (it links into each core's own `.bss`, which on
//! the standard H7 memory maps are different physical RAMs — the M4 cannot
//! even reach the M7's AXI SRAM). A same-core re-entry fails the flag's
//! test-and-set before the HSEM is touched, which is deterministic and free
//! of false positives. The HSEM only arbitrates between the two cores.
//!
//! # Requirements and caveats
//!
//! * Call [`crate::init_primary`] / [`crate::init_secondary`] first: the HSEM
//!   peripheral clock is only enabled by embassy's dual-core init.
//! * HSEM index `0` is reserved (embassy's dual-core init handshake uses it).
//!   Valid indices are `1..=31`, enforced at compile time via [`RawMutex::INIT`].
//! * Each `HsemRawMutex` consumes one hardware semaphore. Distinct mutexes
//!   must use distinct indices.
//! * The `held` flag must stay in core-private RAM (default `.bss` placement
//!   satisfies this). Do not move it into shared memory.
//! * The lock never masks interrupts and never sleeps. Do not hold it across
//!   `.await`; keep critical sections short.
//! * On panic, the mutex stays locked (flag and semaphore) on purpose:
//!   fail fast, keep the evidence.

use core::sync::atomic::{AtomicBool, Ordering, compiler_fence};

use cortex_m::asm::dsb;
use embassy_sync::blocking_mutex::raw::RawMutex;

use crate::cpu::CoreId;

/// Process ID used for all lock/unlock operations of this mutex.
const PROCID: u8 = 0xA5;

/// A cross-core raw mutex backed by hardware semaphore `N`.
///
/// `N` is the HSEM register index, `1..=31` (index `0` is reserved by
/// embassy's dual-core initialization handshake).
///
/// Construct via [`RawMutex::INIT`]:
///
/// ```rust,ignore
/// use embassy_stm32::hsem_raw_mutex::HsemRawMutex;
/// use embassy_sync::channel::Channel;
///
/// static CH: Channel<HsemRawMutex<3>, u32, 4> = Channel::new();
/// ```
pub struct HsemRawMutex<const N: u8> {
    /// Core-private "this core currently holds the semaphore" flag.
    ///
    /// In each firmware image this links into that core's own `.bss`, i.e.
    /// different physical RAM per core, so no cross-core coherence (and no
    /// cache configuration) is needed. It is what makes same-core re-entry
    /// detection airtight — see the module docs.
    held: AtomicBool,
}

// Safety: mutual exclusion between cores is enforced by the HSEM hardware
// (RM0399 §11.3: a lock write to an already-locked semaphore has no effect),
// which is coherent across cores by construction. Re-entry from the same core
// is excluded by the `held` flag protocol below. The DSBs order the guarded
// data against lock acquisition/release per ST AN4839.
unsafe impl<const N: u8> Send for HsemRawMutex<N> {}
unsafe impl<const N: u8> Sync for HsemRawMutex<N> {}

unsafe impl<const N: u8> RawMutex for HsemRawMutex<N> {
    const INIT: Self = {
        const {
            core::assert!(
                N >= 1 && N <= 31,
                "HsemRawMutex index must be in 1..=31 (index 0 is reserved by embassy's dual-core init handshake)"
            )
        };
        Self {
            held: AtomicBool::new(false),
        }
    };

    fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        // 1. Same-core re-entry check. If any context on THIS core already
        //    holds the mutex (thread or IRQ), locking again would deadlock —
        //    panic instead. Core-private, so this can never false-positive.
        if self.held.swap(true, Ordering::Acquire) {
            panic!("HsemRawMutex<{}>: re-acquired on same core", N);
        }

        // 2. Cross-core arbitration in hardware.
        //    CoreId values match the HSEM COREID field encoding (RM0399
        //    table 95): Core0 (Cortex-M7) = 0x3, Core1 (Cortex-M4) = 0x1.
        let me = CoreId::current() as u8;
        let sem = crate::pac::HSEM.r(N as usize);
        loop {
            // 2-step lock procedure (RM0399 §11.3.7): write the lock request,
            // then read back to check it was accepted. If the other core
            // holds it, the write is ignored and the readback says so.
            sem.write(|w| {
                w.set_procid(PROCID);
                w.set_coreid(me);
                w.set_lock(true);
            });
            let reg = sem.read();

            if reg.lock() && reg.coreid() == me && reg.procid() == PROCID {
                break;
            }
            core::hint::spin_loop();
        }

        // 3. Acquired. Order the critical section against the lock
        //    acquisition (barrier guidance from ST AN4839).
        compiler_fence(Ordering::Acquire);
        dsb();

        let ret = f();

        compiler_fence(Ordering::Release);
        dsb();
        // Unlock is honored only if COREID+PROCID match the holder's, so a
        // stray unlock from the wrong context is a no-op.
        sem.write(|w| {
            w.set_procid(PROCID);
            w.set_coreid(me);
            w.set_lock(false);
        });
        // Clear the local flag last, so a preempting context that observes
        // `held == false` is ordered after the completed unlock.
        compiler_fence(Ordering::Release);
        self.held.store(false, Ordering::Release);

        ret
    }
}
