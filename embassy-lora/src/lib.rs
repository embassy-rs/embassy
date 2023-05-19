#![no_std]
#![feature(async_fn_in_trait, impl_trait_projections)]
//! embassy-lora holds LoRa-specific functionality.

pub(crate) mod fmt;

/// interface variants required by the external lora physical layer crate (lora-phy)
pub mod iv;

#[cfg(feature = "time")]
use embassy_time::{Duration, Instant, Timer};

/// A convenience timer to use with the LoRaWAN crate
#[cfg(feature = "time")]
pub struct LoraTimer {
    start: Instant,
}

#[cfg(feature = "time")]
impl LoraTimer {
    pub fn new() -> Self {
        Self { start: Instant::now() }
    }
}

#[cfg(feature = "time")]
impl lorawan_device::async_device::radio::Timer for LoraTimer {
    fn reset(&mut self) {
        self.start = Instant::now();
    }

    async fn at(&mut self, millis: u64) {
        Timer::at(self.start + Duration::from_millis(millis)).await
    }

    async fn delay_ms(&mut self, millis: u64) {
        Timer::after(Duration::from_millis(millis)).await
    }
}
