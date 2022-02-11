#![cfg_attr(not(any(feature = "std", feature = "wasm")), no_std)]
#![cfg_attr(
    feature = "nightly",
    feature(
        const_fn_trait_bound,
        const_fn_fn_ptr_basics,
        generic_associated_types,
        type_alias_impl_trait
    )
)]
#![allow(clippy::new_without_default)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod blocking_mutex;
pub mod channel;
pub mod waitqueue;

pub mod executor;
#[cfg(cortex_m)]
pub mod interrupt;
pub mod io;
#[cfg(feature = "time")]
pub mod time;
pub mod util;

#[cfg(feature = "nightly")]
pub use embassy_macros::{main, task};

#[doc(hidden)]
/// Implementation details for embassy macros. DO NOT USE.
pub mod export {
    pub use atomic_polyfill as atomic;
}
