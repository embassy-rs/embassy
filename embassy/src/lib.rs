#![cfg_attr(not(feature = "std"), no_std)]
#![feature(slice_fill)]
#![feature(generic_associated_types)]
#![feature(const_fn)]

pub mod executor;
pub mod flash;
pub mod io;
pub mod time;
pub mod util;
pub mod rand;