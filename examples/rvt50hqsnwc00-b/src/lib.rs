#![no_std]

pub mod rvt50_board;

#[cfg(feature = "rlvgl")]
pub mod rlvgl;

#[cfg(feature = "oxivgl")]
pub mod oxivgl;
