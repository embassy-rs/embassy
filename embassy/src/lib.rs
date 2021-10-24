#![cfg_attr(not(any(feature = "std", feature = "wasm")), no_std)]
#![feature(generic_associated_types)]
#![feature(const_fn_trait_bound)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::new_without_default)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod blocking_mutex;
pub mod channel;
pub mod waitqueue;

pub mod executor;
pub mod interrupt;
pub mod io;
#[cfg(feature = "time")]
pub mod time;
pub mod util;

pub use embassy_macros::*;
pub use embassy_traits as traits;

#[doc(hidden)]
/// Implementation details for embassy macros. DO NOT USE.
pub mod export {
    pub use atomic_polyfill as atomic;
}
