#![cfg_attr(not(any(feature = "arch-std", feature = "arch-wasm")), no_std)]
#![allow(clippy::new_without_default)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub use embassy_executor_macros::task;

macro_rules! check_at_most_one {
    (@amo [$($feats:literal)*] [] [$($res:tt)*]) => {
        #[cfg(any($($res)*))]
        compile_error!(concat!("At most one of these features can be enabled at the same time:", $(" `", $feats, "`",)*));
    };
    (@amo $feats:tt [$curr:literal $($rest:literal)*] [$($res:tt)*]) => {
        check_at_most_one!(@amo $feats [$($rest)*] [$($res)* $(all(feature=$curr, feature=$rest),)*]);
    };
    ($($f:literal),*$(,)?) => {
        check_at_most_one!(@amo [$($f)*] [$($f)*] []);
    };
}
check_at_most_one!("arch-avr", "arch-cortex-m", "arch-riscv32", "arch-std", "arch-wasm",);

#[cfg(feature = "_arch")]
#[cfg_attr(feature = "arch-avr", path = "arch/avr.rs")]
#[cfg_attr(feature = "arch-cortex-m", path = "arch/cortex_m.rs")]
#[cfg_attr(feature = "arch-riscv32", path = "arch/riscv32.rs")]
#[cfg_attr(feature = "arch-std", path = "arch/std.rs")]
#[cfg_attr(feature = "arch-wasm", path = "arch/wasm.rs")]
mod arch;

#[cfg(feature = "_arch")]
#[allow(unused_imports)] // don't warn if the module is empty.
pub use arch::*;

pub mod raw;

mod spawner;
pub use spawner::*;

mod config {
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}

/// Implementation details for embassy macros.
/// Do not use. Used for macros and HALs only. Not covered by semver guarantees.
#[doc(hidden)]
#[cfg(not(feature = "nightly"))]
pub mod _export {
    use core::alloc::Layout;
    use core::cell::{Cell, UnsafeCell};
    use core::future::Future;
    use core::mem::MaybeUninit;
    use core::ptr::null_mut;

    use critical_section::{CriticalSection, Mutex};

    use crate::raw::TaskPool;

    struct Arena<const N: usize> {
        buf: UnsafeCell<MaybeUninit<[u8; N]>>,
        ptr: Mutex<Cell<*mut u8>>,
    }

    unsafe impl<const N: usize> Sync for Arena<N> {}
    unsafe impl<const N: usize> Send for Arena<N> {}

    impl<const N: usize> Arena<N> {
        const fn new() -> Self {
            Self {
                buf: UnsafeCell::new(MaybeUninit::uninit()),
                ptr: Mutex::new(Cell::new(null_mut())),
            }
        }

        fn alloc<T>(&'static self, cs: CriticalSection) -> &'static mut MaybeUninit<T> {
            let layout = Layout::new::<T>();

            let start = self.buf.get().cast::<u8>();
            let end = unsafe { start.add(N) };

            let mut ptr = self.ptr.borrow(cs).get();
            if ptr.is_null() {
                ptr = self.buf.get().cast::<u8>();
            }

            let bytes_left = (end as usize) - (ptr as usize);
            let align_offset = (ptr as usize).next_multiple_of(layout.align()) - (ptr as usize);

            if align_offset + layout.size() > bytes_left {
                panic!("embassy-executor: task arena is full. You must increase the arena size, see the documentation for details: https://docs.embassy.dev/embassy-executor/");
            }

            let res = unsafe { ptr.add(align_offset) };
            let ptr = unsafe { ptr.add(align_offset + layout.size()) };

            self.ptr.borrow(cs).set(ptr);

            unsafe { &mut *(res as *mut MaybeUninit<T>) }
        }
    }

    static ARENA: Arena<{ crate::config::TASK_ARENA_SIZE }> = Arena::new();

    pub struct TaskPoolRef {
        // type-erased `&'static mut TaskPool<F, N>`
        // Needed because statics can't have generics.
        ptr: Mutex<Cell<*mut ()>>,
    }
    unsafe impl Sync for TaskPoolRef {}
    unsafe impl Send for TaskPoolRef {}

    impl TaskPoolRef {
        pub const fn new() -> Self {
            Self {
                ptr: Mutex::new(Cell::new(null_mut())),
            }
        }

        /// Get the pool for this ref, allocating it from the arena the first time.
        ///
        /// safety: for a given TaskPoolRef instance, must always call with the exact
        /// same generic params.
        pub unsafe fn get<F: Future, const N: usize>(&'static self) -> &'static TaskPool<F, N> {
            critical_section::with(|cs| {
                let ptr = self.ptr.borrow(cs);
                if ptr.get().is_null() {
                    let pool = ARENA.alloc::<TaskPool<F, N>>(cs);
                    pool.write(TaskPool::new());
                    ptr.set(pool as *mut _ as _);
                }

                unsafe { &*(ptr.get() as *const _) }
            })
        }
    }
}
