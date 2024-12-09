use core::arch::asm;
use core::sync::atomic::{compiler_fence, AtomicBool, AtomicU32, Ordering};

// Must be kept in sync with the layout of `State`!
pub(crate) const STATE_SPAWNED: u32 = 1 << 0;
pub(crate) const STATE_RUN_QUEUED: u32 = 1 << 8;

#[repr(C, align(4))]
pub(crate) struct State {
    /// Task is spawned (has a future)
    spawned: AtomicBool,
    /// Task is in the executor run queue
    run_queued: AtomicBool,
    /// Task is in the executor timer queue
    timer_queued: AtomicBool,
    pad: AtomicBool,
}

impl State {
    pub const fn new() -> State {
        Self {
            spawned: AtomicBool::new(false),
            run_queued: AtomicBool::new(false),
            timer_queued: AtomicBool::new(false),
            pad: AtomicBool::new(false),
        }
    }

    fn as_u32(&self) -> &AtomicU32 {
        unsafe { &*(self as *const _ as *const AtomicU32) }
    }

    /// If task is idle, mark it as spawned + run_queued and return true.
    #[inline(always)]
    pub fn spawn(&self) -> bool {
        compiler_fence(Ordering::Release);
        let r = self
            .as_u32()
            .compare_exchange(
                0,
                STATE_SPAWNED | STATE_RUN_QUEUED,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok();
        compiler_fence(Ordering::Acquire);
        r
    }

    /// Unmark the task as spawned.
    #[inline(always)]
    pub fn despawn(&self) {
        compiler_fence(Ordering::Release);
        self.spawned.store(false, Ordering::Relaxed);
    }

    /// Mark the task as run-queued if it's spawned and isn't already run-queued. Return true on success.
    #[inline(always)]
    pub fn run_enqueue(&self) -> bool {
        unsafe {
            loop {
                let state: u32;
                asm!("ldrex {}, [{}]", out(reg) state, in(reg) self, options(nostack));

                if (state & STATE_RUN_QUEUED != 0) || (state & STATE_SPAWNED == 0) {
                    asm!("clrex", options(nomem, nostack));
                    return false;
                }

                let outcome: usize;
                let new_state = state | STATE_RUN_QUEUED;
                asm!("strex {}, {}, [{}]", out(reg) outcome, in(reg) new_state, in(reg) self, options(nostack));
                if outcome == 0 {
                    return true;
                }
            }
        }
    }

    /// Unmark the task as run-queued. Return whether the task is spawned.
    #[inline(always)]
    pub fn run_dequeue(&self) -> bool {
        compiler_fence(Ordering::Release);

        let r = self.spawned.load(Ordering::Relaxed);
        self.run_queued.store(false, Ordering::Relaxed);
        r
    }

    /// Mark the task as timer-queued. Return whether it was newly queued (i.e. not queued before)
    #[cfg(feature = "integrated-timers")]
    #[inline(always)]
    pub fn timer_enqueue(&self) -> bool {
        !self.timer_queued.swap(true, Ordering::Relaxed)
    }

    /// Unmark the task as timer-queued.
    #[cfg(feature = "integrated-timers")]
    #[inline(always)]
    pub fn timer_dequeue(&self) {
        self.timer_queued.store(false, Ordering::Relaxed);
    }
}
