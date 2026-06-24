#![no_std]

pub mod board;
pub mod firmware_id;
pub mod ft5446;
#[cfg(feature = "oxivgl-demo")]
pub mod oxivgl;
pub mod pio_rgb;
#[cfg(feature = "rlvgl-demo")]
pub mod rlvgl;
pub mod touch_feed;
