//! Async task executor.

#![deny(missing_docs)]

cfg_if::cfg_if! {
    if #[cfg(cortex_m)] {
        #[path="arch/cortex_m.rs"]
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

pub mod raw;

mod spawner;
pub use spawner::*;
