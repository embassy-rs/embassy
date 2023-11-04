#![cfg_attr(not(any(feature = "arch-std", feature = "arch-wasm")), no_std)]
#![cfg_attr(all(feature = "nightly", feature = "arch-xtensa"), feature(asm_experimental_arch))]
#![allow(clippy::new_without_default)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[cfg(feature = "nightly")]
pub use embassy_macros::task;

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
check_at_most_one!("arch-cortex-m", "arch-riscv32", "arch-xtensa", "arch-std", "arch-wasm",);

#[cfg(feature = "_arch")]
#[cfg_attr(feature = "arch-cortex-m", path = "arch/cortex_m.rs")]
#[cfg_attr(feature = "arch-riscv32", path = "arch/riscv32.rs")]
#[cfg_attr(feature = "arch-xtensa", path = "arch/xtensa.rs")]
#[cfg_attr(feature = "arch-std", path = "arch/std.rs")]
#[cfg_attr(feature = "arch-wasm", path = "arch/wasm.rs")]
mod arch;

#[cfg(feature = "_arch")]
#[allow(unused_imports)] // don't warn if the module is empty.
pub use arch::*;

pub mod raw;

mod spawner;
pub use spawner::*;

/// Implementation details for embassy macros.
/// Do not use. Used for macros and HALs only. Not covered by semver guarantees.
#[doc(hidden)]
pub mod _export {
    #[cfg(feature = "rtos-trace")]
    pub use rtos_trace::trace;

    /// Expands the given block of code when `embassy-executor` is compiled with
    /// the `rtos-trace-interrupt` feature.
    #[doc(hidden)]
    #[macro_export]
    #[cfg(feature = "rtos-trace-interrupt")]
    macro_rules! rtos_trace_interrupt {
        ($($tt:tt)*) => { $($tt)* };
    }

    /// Does not expand the given block of code when `embassy-executor` is
    /// compiled without the `rtos-trace-interrupt` feature.
    #[doc(hidden)]
    #[macro_export]
    #[cfg(not(feature = "rtos-trace-interrupt"))]
    macro_rules! rtos_trace_interrupt {
        ($($tt:tt)*) => {};
    }
}
