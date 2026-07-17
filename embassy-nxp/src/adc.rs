#![macro_use]
//! Analog to digital converter (ADC) driver

#[cfg_attr(lpc55, path= "./adc/lpc55.rs")]
mod inner;
pub use innner::*;
