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
check_at_most_one!(
    "arch-avr",
    "arch-cortex-m",
    "arch-riscv32",
    "arch-std",
    "arch-wasm",
    "arch-spin",
);

#[cfg(feature = "_arch")]
#[cfg_attr(feature = "arch-avr", path = "arch/avr.rs")]
#[cfg_attr(feature = "arch-cortex-m", path = "arch/cortex_m.rs")]
#[cfg_attr(feature = "arch-riscv32", path = "arch/riscv32.rs")]
#[cfg_attr(feature = "arch-std", path = "arch/std.rs")]
#[cfg_attr(feature = "arch-wasm", path = "arch/wasm.rs")]
#[cfg_attr(feature = "arch-spin", path = "arch/spin.rs")]
mod arch;

#[cfg(feature = "_arch")]
#[allow(unused_imports)] // don't warn if the module is empty.
pub use arch::*;
#[cfg(not(feature = "_arch"))]
pub use embassy_executor_macros::main_unspecified as main;

pub mod raw;

mod spawner;
pub use spawner::*;

/// Implementation details for embassy macros.
/// Do not use. Used for macros and HALs only. Not covered by semver guarantees.
#[doc(hidden)]
#[cfg(not(feature = "nightly"))]
pub mod _export {
    use core::cell::UnsafeCell;
    use core::future::Future;
    use core::mem::MaybeUninit;

    use crate::raw::TaskPool;

    pub trait TaskFn<Args>: Copy {
        type Fut: Future + 'static;
    }

    macro_rules! task_fn_impl {
        ($($Tn:ident),*) => {
            impl<F, Fut, $($Tn,)*> TaskFn<($($Tn,)*)> for F
            where
                F: Copy + FnOnce($($Tn,)*) -> Fut,
                Fut: Future + 'static,
            {
                type Fut = Fut;
            }
        };
    }

    task_fn_impl!();
    task_fn_impl!(T0);
    task_fn_impl!(T0, T1);
    task_fn_impl!(T0, T1, T2);
    task_fn_impl!(T0, T1, T2, T3);
    task_fn_impl!(T0, T1, T2, T3, T4);
    task_fn_impl!(T0, T1, T2, T3, T4, T5);
    task_fn_impl!(T0, T1, T2, T3, T4, T5, T6);
    task_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7);
    task_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
    task_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
    task_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
    task_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
    task_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
    task_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
    task_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
    task_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);

    #[allow(private_bounds)]
    #[repr(C)]
    pub struct TaskPoolHolder<const SIZE: usize, const ALIGN: usize>
    where
        Align<ALIGN>: Alignment,
    {
        data: UnsafeCell<[MaybeUninit<u8>; SIZE]>,
        align: Align<ALIGN>,
    }

    unsafe impl<const SIZE: usize, const ALIGN: usize> Send for TaskPoolHolder<SIZE, ALIGN> where Align<ALIGN>: Alignment {}
    unsafe impl<const SIZE: usize, const ALIGN: usize> Sync for TaskPoolHolder<SIZE, ALIGN> where Align<ALIGN>: Alignment {}

    #[allow(private_bounds)]
    impl<const SIZE: usize, const ALIGN: usize> TaskPoolHolder<SIZE, ALIGN>
    where
        Align<ALIGN>: Alignment,
    {
        pub const fn get(&self) -> *const u8 {
            self.data.get().cast()
        }
    }

    pub const fn task_pool_size<F, Args, Fut, const POOL_SIZE: usize>(_: F) -> usize
    where
        F: TaskFn<Args, Fut = Fut>,
        Fut: Future + 'static,
    {
        size_of::<TaskPool<Fut, POOL_SIZE>>()
    }

    pub const fn task_pool_align<F, Args, Fut, const POOL_SIZE: usize>(_: F) -> usize
    where
        F: TaskFn<Args, Fut = Fut>,
        Fut: Future + 'static,
    {
        align_of::<TaskPool<Fut, POOL_SIZE>>()
    }

    pub const fn task_pool_new<F, Args, Fut, const POOL_SIZE: usize>(_: F) -> TaskPool<Fut, POOL_SIZE>
    where
        F: TaskFn<Args, Fut = Fut>,
        Fut: Future + 'static,
    {
        TaskPool::new()
    }

    #[allow(private_bounds)]
    #[repr(transparent)]
    pub struct Align<const N: usize>([<Self as Alignment>::Archetype; 0])
    where
        Self: Alignment;

    trait Alignment {
        /// A zero-sized type of particular alignment.
        type Archetype: Copy + Eq + PartialEq + Send + Sync + Unpin;
    }

    macro_rules! aligns {
        ($($AlignX:ident: $n:literal,)*) => {
            $(
                #[derive(Copy, Clone, Eq, PartialEq)]
                #[repr(align($n))]
                struct $AlignX {}
                impl Alignment for Align<$n> {
                    type Archetype = $AlignX;
                }
            )*
        };
    }

    aligns!(
        Align1:         1,
        Align2:         2,
        Align4:         4,
        Align8:         8,
        Align16:        16,
        Align32:        32,
        Align64:        64,
        Align128:       128,
        Align256:       256,
        Align512:       512,
        Align1024:      1024,
        Align2048:      2048,
        Align4096:      4096,
        Align8192:      8192,
        Align16384:     16384,
    );
    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
    aligns!(
        Align32768:     32768,
        Align65536:     65536,
        Align131072:    131072,
        Align262144:    262144,
        Align524288:    524288,
        Align1048576:   1048576,
        Align2097152:   2097152,
        Align4194304:   4194304,
        Align8388608:   8388608,
        Align16777216:  16777216,
        Align33554432:  33554432,
        Align67108864:  67108864,
        Align134217728: 134217728,
        Align268435456: 268435456,
        Align536870912: 536870912,
    );
}
