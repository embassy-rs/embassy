use core::sync::atomic::{compiler_fence, AtomicBool, AtomicU32, Ordering};

#[derive(Clone, Copy)]
pub(crate) struct Token(());

/// Creates a token and passes it to the closure.
///
/// This is a no-op replacement for `CriticalSection::with` because we don't need any locking.
pub(crate) fn locked<R>(f: impl FnOnce(Token) -> R) -> R {
    f(Token(()))
}

// Must be kept in sync with the layout of `State`!
pub(crate) const STATE_SPAWNED: u32 = 1 << 0;
pub(crate) const STATE_RUN_QUEUED: u32 = 1 << 8;

#[repr(C, align(4))]
pub(crate) struct State {
    /// Task is spawned (has a future)
    spawned: AtomicBool,
    /// Task is in the executor run queue
    run_queued: AtomicBool,
    pad: AtomicBool,
    pad2: AtomicBool,
}

impl State {
    pub const fn new() -> State {
        Self {
            spawned: AtomicBool::new(false),
            run_queued: AtomicBool::new(false),
            pad: AtomicBool::new(false),
            pad2: AtomicBool::new(false),
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

    /// Mark the task as run-queued if it's spawned and isn't already run-queued. Run the given
    /// function if the task was successfully marked.
    #[inline(always)]
    pub fn run_enqueue(&self, f: impl FnOnce(Token)) {
        let old = self.run_queued.swap(true, Ordering::AcqRel);

        if !old {
            locked(f);
        }
    }

    /// Unmark the task as run-queued. Return whether the task is spawned.
    #[inline(always)]
    pub fn run_dequeue(&self) {
        compiler_fence(Ordering::Release);

        self.run_queued.store(false, Ordering::Relaxed);
    }
}
