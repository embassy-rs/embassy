#![cfg_attr(not(feature = "std"), no_std)]
#![feature(generic_associated_types)]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_option)]
#![allow(incomplete_features)]

pub mod flash;
pub mod gpio;
pub mod uart;
