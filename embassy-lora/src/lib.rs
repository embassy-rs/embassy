#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]
//! embassy-lora is a collection of async radio drivers that integrate with the lorawan-device
//! crate's async LoRaWAN MAC implementation.

pub(crate) mod fmt;

#[cfg(feature = "stm32wl")]
pub mod stm32wl;
#[cfg(feature = "sx127x")]
pub mod sx127x;

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

    type AtFuture<'m> = impl core::future::Future<Output = ()> + 'm;
    fn at<'m>(&'m mut self, millis: u64) -> Self::AtFuture<'m> {
        Timer::at(self.start + Duration::from_millis(millis))
    }

    type DelayFuture<'m> = impl core::future::Future<Output = ()> + 'm;
    fn delay_ms<'m>(&'m mut self, millis: u64) -> Self::DelayFuture<'m> {
        Timer::after(Duration::from_millis(millis))
    }
}
