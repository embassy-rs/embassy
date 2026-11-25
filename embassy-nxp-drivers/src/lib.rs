#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(gpio)]
pub mod gpio;

#[allow(unused_imports)]
use nxp_pac as pac;
