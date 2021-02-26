#![cfg_attr(not(feature = "std"), no_std)]
#![feature(generic_associated_types)]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_option)]
#![allow(incomplete_features)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod executor;
pub mod flash;
pub mod gpio;
pub mod interrupt;
pub mod io;
pub mod rand;
pub mod time;
pub mod uart;
pub mod util;
