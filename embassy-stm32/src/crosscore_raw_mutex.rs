//! Cross-core [`RawMutex`](embassy_sync::blocking_mutex::raw::RawMutex) for
//! dual-core STM32H7 built from shared-RAM atomics — the **HSEM-free**
//! variant. For a version that needs no shared memory or MPU setup at all,
//! see [`crate::hsem_raw_mutex`].
//!
//! Locking semantics:
//!
//! * Contention from the *other* core spins (busy-wait).
//! * Re-entrant acquisition from the *same* core — another task, or an IRQ
//!   that preempted this core's lock holder — panics instead of deadlocking.
//!
//! # Memory placement (critical!)
//!
//! The mutex state lives in RAM and MUST be placed where both cores see it
//! coherently: SRAM4 (`0x3800_0000`, D3 domain) with the MPU mapping it
//! non-cacheable on the Cortex-M7 (see the accompanying demo crate). The same
//! requirement applies to everything the mutex guards. Zero-initialized RAM
//! is a valid unlocked state (`owner == 0`), so a NOLOAD linker section is
//! safe and no runtime initialization is needed.
//!
//! # How the same-core panic works
//!
//! `owner` records the holding core (`CoreId::to_index() + 1`, so 0 means
//! "unlocked"). On contention, a core that sees *itself* as owner is
//! re-entering (thread→thread, thread→IRQ, IRQ→IRQ) and panics. The check is
//! confirmed with a second Acquire read to avoid a stale-read false positive;
//! like the HSEM variant, this path is best-effort debugging — mutual
//! exclusion rests on the compare-exchange, which is exact.

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use embassy_sync::blocking_mutex::raw::RawMutex;

use crate::cpu::CoreId;

/// `owner == 0` means "not held". Keeping 0 as "unlocked" makes
/// zero-initialized RAM a valid state.
const UNLOCKED: u8 = 0;

/// Current core as 1 (M7) or 2 (M4).
#[inline]
fn current_core() -> u8 {
    CoreId::current().to_index() + 1
}

/// A cross-core raw mutex in shared, non-cacheable RAM.
///
/// **The static must be placed in shared non-cacheable memory** (SRAM4 /
/// `.ram_d3` on the H7), along with everything it guards:
///
/// ```rust,ignore
/// use embassy_stm32::crosscore_raw_mutex::CrossCoreRawMutex;
/// use embassy_sync::channel::Channel;
///
/// #[link_section = ".ram_d3"]
/// static CH: Channel<CrossCoreRawMutex, u32, 4> = Channel::new();
/// ```
pub struct CrossCoreRawMutex {
    locked: AtomicBool,
    owner: AtomicU8,
}

// Safety: mutual exclusion is provided by the compare-exchange protocol below
// on memory both cores access coherently (non-cacheable SRAM4).
unsafe impl Send for CrossCoreRawMutex {}
unsafe impl Sync for CrossCoreRawMutex {}

impl CrossCoreRawMutex {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            owner: AtomicU8::new(UNLOCKED),
        }
    }
}

impl Default for CrossCoreRawMutex {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl RawMutex for CrossCoreRawMutex {
    const INIT: Self = Self::new();

    fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        let me = current_core();
        loop {
            // The safety-critical line: CAS guarantees two contexts can never
            // both observe success, on one core or two.
            if self
                .locked
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                self.owner.store(me, Ordering::Relaxed);
                let ret = f();
                self.owner.store(UNLOCKED, Ordering::Relaxed);
                // Release: the owner reset above is visible to the next
                // acquirer before they see `locked == false`.
                self.locked.store(false, Ordering::Release);
                return ret;
            }

            // Contended. If the owner is THIS core (another task, or an IRQ
            // that preempted this core's holder), locking again would
            // deadlock — panic instead. Double-read to avoid a stale relaxed
            // read (ABA across a previous own ownership).
            if self.owner.load(Ordering::Acquire) == me && self.locked.load(Ordering::Acquire) {
                if self.owner.load(Ordering::Acquire) == me {
                    panic!("CrossCoreRawMutex: re-acquired on same core");
                }
            }

            core::hint::spin_loop();
        }
    }
}
