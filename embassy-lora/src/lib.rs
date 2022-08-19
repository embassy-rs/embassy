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

/// A convenience timer to use with the LoRaWAN crate
pub struct LoraTimer;

#[cfg(feature = "time")]
impl lorawan_device::async_device::radio::Timer for LoraTimer {
    type DelayFuture<'m> = impl core::future::Future<Output = ()> + 'm;
    fn delay_ms<'m>(&'m mut self, millis: u64) -> Self::DelayFuture<'m> {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(millis))
    }
}
