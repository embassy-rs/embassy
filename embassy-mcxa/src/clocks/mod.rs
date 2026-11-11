//! Family-selected clock configuration and peripheral control.

#[cfg(any(feature = "_mcxa2xx", feature = "_mcxa5xx"))]
mod mcxa;
#[cfg(any(feature = "_mcxa2xx", feature = "_mcxa5xx"))]
pub use mcxa::*;

#[cfg(any(feature = "_mcxa2xx", feature = "_mcxa5xx"))]
pub(crate) use mcxa::default_register_value;
#[cfg(any(mcxa_eim, mcxa_erm))]
pub(crate) use mcxa::enable_ecc_ram_a_clock;
#[cfg(mcxa_gdet)]
pub(crate) use mcxa::gdet_access_ready;
#[cfg(mcxa_glikey)]
pub(crate) use mcxa::glikey_access_ready;
