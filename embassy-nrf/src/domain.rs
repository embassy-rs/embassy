//! Power domains (nRF54L).
//!
//! On nRF54L, each power domain has its own GPIO port, and peripherals can only use pins
//! in their own domain:
//!
//! | Domain      | Peripherals           | Pins                  |
//! |-------------|-----------------------|-----------------------|
//! | [`Domain0`] | MCU (`xx00`)          | P2 (dedicated only)   |
//! | [`Domain1`] | Radio (`xx10`)        |                       |
//! | [`Domain2`] | Peripheral (`xx2x`)   | P1, P3                |
//! | [`Domain3`] | Low-power (`xx30`)    | P0                    |
//!
//! A peripheral implementing [`DomainPeripheral`] can use any pin implementing
//! [`DomainPin`] for its domain. P2 pins don't implement [`DomainPin`]: they can only be used
//! through their dedicated functions, which show up as explicit impls of each driver's pin
//! traits (e.g. [`spim::SckPin`](crate::spim::SckPin)).
#![macro_use]

use embassy_hal_internal::PeripheralType;

pub(crate) trait SealedDomain {}
pub(crate) trait SealedDomainPeripheral {}

/// A power domain.
#[allow(private_bounds)]
pub trait Domain: SealedDomain + 'static {}

/// MCU power domain.
pub enum Domain0 {}
/// Radio power domain.
pub enum Domain1 {}
/// Peripheral power domain.
pub enum Domain2 {}
/// Low-power power domain.
pub enum Domain3 {}

macro_rules! impl_domain_marker {
    ($($d:ident),*) => {$(
        impl SealedDomain for $d {}
        impl Domain for $d {}
    )*};
}
impl_domain_marker!(Domain0, Domain1, Domain2, Domain3);

/// A peripheral instance located in a power domain.
#[allow(private_bounds)]
pub trait DomainPeripheral: SealedDomainPeripheral + PeripheralType {
    /// The power domain this peripheral is in.
    type Domain: Domain;
}

/// A pin that any peripheral in power domain `D` can use.
pub trait DomainPin<D: Domain>: crate::gpio::Pin {}

/// Declare the power domain of peripheral instances.
macro_rules! impl_domain {
    ($domain:ident: $($instance:ident),* $(,)?) => {
        $(
            impl crate::domain::SealedDomainPeripheral for peripherals::$instance {}
            impl crate::domain::DomainPeripheral for peripherals::$instance {
                type Domain = crate::domain::$domain;
            }
        )*
    };
}
