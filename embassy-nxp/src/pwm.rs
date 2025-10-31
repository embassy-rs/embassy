//! Pulse-Width Modulation (PWM) driver.

#[cfg_attr(feature = "lpc55-core0", path = "./pwm/lpc55.rs")]
mod inner;
pub use inner::*;
