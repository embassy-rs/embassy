#[cfg(arm_profile = "legacy")]
compile_error!("`platform-aarch32` does not support the legacy ARM profile, WFE/SEV are not available.");

#[cfg(any(feature = "executor-thread", feature = "executor-interrupt"))]
struct Aarch32Pender;

#[cfg(any(feature = "executor-thread", feature = "executor-interrupt"))]
pender_impl!(Aarch32Pender);

#[cfg(any(feature = "executor-thread", feature = "executor-interrupt"))]
impl crate::pender::Pender for Aarch32Pender {
    fn pend(context: *mut ()) {
        // `context` is either null (thread mode executor, created by `Executor::run`) or a pointer
        // to the `SgiTarget` of an `InterruptExecutor` (created by `InterruptExecutor::start`).

        #[cfg(feature = "executor-thread")]
        // Try to make Rust optimize the branching away if we only use thread mode.
        if !cfg!(feature = "executor-interrupt") || context.is_null() {
            aarch32_cpu::asm::sev();
            return;
        }

        #[cfg(feature = "executor-interrupt")]
        {
            // Safety: `context` points to the `SgiTarget` stored inside a `&'static InterruptExecutor`,
            // which is initialized in `start()` before the `raw::Executor` is created.
            let target = unsafe { &*(context as *const interrupt::SgiTarget) };
            target.pend();
        }
    }
}

#[cfg(feature = "executor-thread")]
pub use thread::*;
#[cfg(feature = "executor-thread")]
mod thread {
    use core::marker::PhantomData;
    use core::ptr;

    use aarch32_cpu::asm::wfe;
    pub use embassy_executor_macros::main_aarch32 as main;

    use crate::{Spawner, raw};

    /// Thread mode executor, using WFE/SEV.
    ///
    /// This is the simplest and most common kind of executor. It runs on
    /// thread mode (at the lowest priority level), and uses the `WFE` ARM instruction
    /// to sleep when it has no more work to do. When a task is woken, a `SEV` instruction
    /// is executed, to make the `WFE` exit from sleep and poll the task.
    ///
    /// This executor allows for ultra low power consumption for chips where `WFE`
    /// triggers low-power sleep without extra steps. If your chip requires extra steps,
    /// you may use [`raw::Executor`] directly to program custom behavior.
    pub struct Executor {
        inner: raw::Executor,
        not_send: PhantomData<*mut ()>,
    }

    impl Executor {
        /// Create a new Executor.
        pub fn new() -> Self {
            Self {
                inner: raw::Executor::new(ptr::null_mut()),
                not_send: PhantomData,
            }
        }

        /// Run the executor.
        ///
        /// The `init` closure is called with a [`Spawner`] that spawns tasks on
        /// this executor. Use it to spawn the initial task(s). After `init` returns,
        /// the executor starts running the tasks.
        ///
        /// To spawn more tasks later, you may keep copies of the [`Spawner`] (it is `Copy`),
        /// for example by passing it as an argument to the initial tasks.
        ///
        /// This function requires `&'static mut self`. This means you have to store the
        /// Executor instance in a place where it'll live forever and grants you mutable
        /// access. There's a few ways to do this:
        ///
        /// - a [StaticCell](https://docs.rs/static_cell/latest/static_cell/) (safe)
        /// - a `static mut` (unsafe)
        /// - a local variable in a function you know never returns (like `fn main() -> !`), upgrading its lifetime with `transmute`. (unsafe)
        ///
        /// This function never returns.
        pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
            init(self.inner.spawner());

            loop {
                unsafe {
                    self.inner.poll();
                }
                crate::trace_idle();
                wfe();
            }
        }
    }
}

#[cfg(feature = "executor-interrupt")]
pub use interrupt::*;
#[cfg(feature = "executor-interrupt")]
mod interrupt {
    use core::cell::{Cell, UnsafeCell};
    use core::mem::MaybeUninit;
    use core::ptr::{read_volatile, write_volatile};

    use critical_section::Mutex;

    use crate::raw;

    /// Offset of the `GICD_IGROUPR0` register from the GIC distributor base address.
    const GICD_IGROUPR0: usize = 0x080;
    /// Offset of the `GICD_ISENABLER0` register from the GIC distributor base address.
    const GICD_ISENABLER0: usize = 0x100;
    /// Offset of the `GICD_SGIR` register from the GIC distributor base address.
    const GICD_SGIR: usize = 0xF00;
    /// `GICD_SGIR.TargetListFilter` value that forwards the SGI only to the CPU that requested it.
    const SGIR_TARGET_SELF: u32 = 0b10 << 24;
    /// `GICD_SGIR.NSATT`: forward the SGI only if it is configured as Group 1 (instead of Group 0).
    const SGIR_NSATT: u32 = 1 << 15;

    /// A Software Generated Interrupt (SGI) on a GICv1/GICv2 distributor.
    ///
    /// This is what the pender uses to pend the interrupt of an [`InterruptExecutor`].
    pub(super) struct SgiTarget {
        gicd_base: usize,
        sgi_id: u8,
        /// Value to write to `GICD_SGIR` to pend the SGI on the current CPU.
        sgir_value: u32,
    }

    impl SgiTarget {
        /// Create the target for `sgi_id` on the distributor at `gicd_base`.
        fn new(gicd_base: usize, sgi_id: u8) -> Self {
            // Safety: reading `GICD_IGROUPR0` has no side effects.
            let group1 = unsafe { read_volatile((gicd_base + GICD_IGROUPR0) as *const u32) & (1 << sgi_id) != 0 };

            // `NSATT` must match the SGI's group for the SGI to be forwarded when writing
            // `GICD_SGIR` from Secure state. It is ignored for Non-secure writes (where
            // `GICD_IGROUPR0` reads as zero), and when the GIC has no Security Extensions.
            let mut sgir_value = SGIR_TARGET_SELF | sgi_id as u32;
            if group1 {
                sgir_value |= SGIR_NSATT;
            }

            Self {
                gicd_base,
                sgi_id,
                sgir_value,
            }
        }

        /// Enable the SGI in the distributor.
        fn enable(&self) {
            // Safety: `GICD_ISENABLER0` is write-1-to-set, so this only enables the configured SGI
            // and leaves all other interrupts untouched.
            unsafe { write_volatile((self.gicd_base + GICD_ISENABLER0) as *mut u32, 1 << self.sgi_id) }
        }

        /// Pend the SGI on the current CPU.
        pub(super) fn pend(&self) {
            // Safety: writing `GICD_SGIR` only generates the configured SGI, it has no other side effects.
            unsafe { write_volatile((self.gicd_base + GICD_SGIR) as *mut u32, self.sgir_value) }
        }
    }

    struct Inner {
        target: SgiTarget,
        executor: raw::Executor,
    }

    /// Interrupt mode executor.
    ///
    /// This executor runs tasks in interrupt mode. The interrupt handler is set up
    /// to poll tasks, and when a task is woken the interrupt is pended from software.
    ///
    /// This allows running async tasks at a priority higher than thread mode. One
    /// use case is to leave thread mode free for non-async tasks. Another use case is
    /// to run multiple executors: one in thread mode for low priority tasks and another in
    /// interrupt mode for higher priority tasks. Higher priority tasks will preempt lower
    /// priority ones.
    ///
    /// It is even possible to run multiple interrupt mode executors at different priorities,
    /// by assigning different priorities to the interrupts.
    ///
    /// The interrupt is a Software Generated Interrupt (SGI, interrupt IDs 0-15) of the
    /// Generic Interrupt Controller (GIC), which is pended by writing to the `GICD_SGIR`
    /// register of the GIC distributor. Only GICv1 and GICv2 (memory-mapped SGI generation)
    /// are supported.
    ///
    /// It is somewhat more complex to use, it's recommended to use the thread-mode
    /// [`Executor`](crate::Executor) instead, if it works for your use case.
    pub struct InterruptExecutor {
        started: Mutex<Cell<bool>>,
        inner: UnsafeCell<MaybeUninit<Inner>>,
    }

    unsafe impl Send for InterruptExecutor {}
    unsafe impl Sync for InterruptExecutor {}

    impl InterruptExecutor {
        /// Create a new, not started `InterruptExecutor`.
        #[inline]
        pub const fn new() -> Self {
            Self {
                started: Mutex::new(Cell::new(false)),
                inner: UnsafeCell::new(MaybeUninit::uninit()),
            }
        }

        /// Executor interrupt callback.
        ///
        /// # Safety
        ///
        /// - You MUST call this from the interrupt handler, and from nowhere else.
        /// - You must not call this before calling `start()`.
        pub unsafe fn on_interrupt(&'static self) {
            let inner = unsafe { (&*self.inner.get()).assume_init_ref() };
            unsafe {
                inner.executor.poll();
            }
        }

        /// Start the executor.
        ///
        /// This initializes the executor, enables the interrupt, and returns.
        /// The executor keeps running in the background through the interrupt.
        ///
        /// `gicd_base` is the base address of the GIC distributor (the `GICD_*` register block).
        /// For example, it is `0xF8F0_1000` on the Xilinx Zynq 7000. On Cortex-A9/A7/A15
        /// cores, it can also be found at offset `0x1000` from the peripheral base in `CBAR`.
        ///
        /// `sgi_id` is the ID of the Software Generated Interrupt to use, from 0 to 15.
        ///
        /// This returns a [`SendSpawner`] you can use to spawn tasks on it. A [`SendSpawner`]
        /// is returned instead of a [`Spawner`](crate::Spawner) because the executor effectively runs in a
        /// different "thread" (the interrupt), so spawning tasks on it is effectively
        /// sending them.
        ///
        /// To obtain a [`Spawner`](crate::Spawner) for this executor, use [`Spawner::for_current_executor()`](crate::Spawner::for_current_executor()) from
        /// a task running in it.
        ///
        /// # Interrupt requirements
        ///
        /// You must write the interrupt handler yourself, and make it call [`on_interrupt()`](Self::on_interrupt).
        ///
        /// This method already enables (unmasks) the interrupt in `GICD_ISENABLER0`, you must NOT do it yourself.
        ///
        /// You must set the interrupt priority (in `GICD_IPRIORITYR`) before calling this method.
        /// You MUST NOT do it after.
        ///
        /// The SGI may be configured as either Group 0 or Group 1 in `GICD_IGROUPR0`. You must
        /// configure the group before calling this method, you MUST NOT change it after.
        ///
        /// The SGI is always sent to the CPU that pends it, so on multi-core chips the executor
        /// must only be woken from the core it was started on.
        ///
        /// # Panics
        ///
        /// Panics if `sgi_id` is greater than 15, or if the executor was already started.
        ///
        /// [`SendSpawner`]: crate::SendSpawner
        pub fn start(&'static self, gicd_base: usize, sgi_id: u8) -> crate::SendSpawner {
            assert!(sgi_id < 16, "SGI interrupt ID must be in the range 0..=15");

            if critical_section::with(|cs| self.started.borrow(cs).replace(true)) {
                panic!("InterruptExecutor::start() called multiple times on the same executor.");
            }

            let inner = unsafe {
                let inner = (*self.inner.get()).as_mut_ptr();
                // Initialize the target first: the executor's pender context points to it.
                (&raw mut (*inner).target).write(SgiTarget::new(gicd_base, sgi_id));
                let target = &raw const (*inner).target;
                (&raw mut (*inner).executor).write(raw::Executor::new(target as *mut ()));
                &*inner
            };

            inner.target.enable();

            inner.executor.spawner().make_send()
        }

        /// Get a SendSpawner for this executor
        ///
        /// This returns a [`SendSpawner`](crate::SendSpawner) you can use to spawn tasks on this
        /// executor.
        ///
        /// This MUST only be called on an executor that has already been started.
        /// The function will panic otherwise.
        pub fn spawner(&'static self) -> crate::SendSpawner {
            if !critical_section::with(|cs| self.started.borrow(cs).get()) {
                panic!("InterruptExecutor::spawner() called on uninitialized executor.");
            }
            let inner = unsafe { (&*self.inner.get()).assume_init_ref() };
            inner.executor.spawner().make_send()
        }
    }
}
