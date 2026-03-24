#![macro_use]

//! Pulse-Width Modulation (PWM) driver.

#[cfg_attr(lpc55, path = "./pwm/lpc55.rs")]
mod inner;
pub use inner::*;
