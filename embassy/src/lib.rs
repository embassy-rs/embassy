#![cfg_attr(not(feature = "std"), no_std)]
#![feature(generic_associated_types)]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_option)]
#![allow(incomplete_features)]
#![feature(type_alias_impl_trait)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod executor;
pub mod interrupt;
pub mod io;
pub mod time;
pub mod util;

pub use embassy_traits as traits;
pub use atomic_polyfill as atomic;
