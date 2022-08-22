#![cfg_attr(not(any(feature = "std", feature = "wasm")), no_std)]
#![cfg_attr(all(feature = "nightly", target_arch = "xtensa"), feature(asm_experimental_arch))]
#![allow(clippy::new_without_default)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[cfg(feature = "nightly")]
pub use embassy_macros::{main, task};

cfg_if::cfg_if! {
    if #[cfg(cortex_m)] {
        #[path="arch/cortex_m.rs"]
        mod arch;
        pub use arch::*;
    }
    else if #[cfg(target_arch="riscv32")] {
        #[path="arch/riscv32.rs"]
        mod arch;
        pub use arch::*;
    }
    else if #[cfg(all(target_arch="xtensa", feature = "nightly"))] {
        #[path="arch/xtensa.rs"]
        mod arch;
        pub use arch::*;
    }
    else if #[cfg(feature="wasm")] {
        #[path="arch/wasm.rs"]
        mod arch;
        pub use arch::*;
    }
    else if #[cfg(feature="std")] {
        #[path="arch/std.rs"]
        mod arch;
        pub use arch::*;
    }
}

#[doc(hidden)]
/// Implementation details for embassy macros. DO NOT USE.
pub mod export {
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

pub mod raw;

mod spawner;
pub use spawner::*;

/// Do not use. Used for macros and HALs only. Not covered by semver guarantees.
#[doc(hidden)]
pub mod _export {
    pub use static_cell::StaticCell;
}
