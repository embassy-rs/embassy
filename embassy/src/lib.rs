#![cfg_attr(not(feature = "std"), no_std)]
#![feature(generic_associated_types)]
#![feature(const_fn_trait_bound)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_option)]
#![allow(incomplete_features)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![feature(maybe_uninit_ref)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

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
