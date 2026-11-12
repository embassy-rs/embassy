//! Family-selected clock configuration and peripheral control.

#[cfg(any(feature = "_mcxa2xx", feature = "_mcxa5xx"))]
mod mcxa;
#[cfg(any(feature = "_mcxa2xx", feature = "_mcxa5xx"))]
pub use mcxa::*;

#[cfg(feature = "_mcxn")]
mod mcxn;
#[cfg(any(feature = "_mcxa2xx", feature = "_mcxa5xx"))]
pub(crate) use mcxa::default_register_value;
#[cfg(any(mcxa_eim, mcxa_erm))]
pub(crate) use mcxa::enable_ecc_ram_a_clock;
#[cfg(all(any(feature = "_mcxa2xx", feature = "_mcxa5xx"), time_driver_ostimer))]
pub(crate) use mcxa::enable_ostimer_clock;
#[cfg(mcxa_gdet)]
pub(crate) use mcxa::gdet_access_ready;
#[cfg(mcxa_glikey)]
pub(crate) use mcxa::glikey_access_ready;
#[cfg(all(feature = "_mcxn", time_driver_ostimer))]
pub(crate) use mcxn::enable_ostimer_clock;
#[cfg(feature = "_mcxn")]
pub use mcxn::*;
#[cfg(feature = "_mcxn")]
pub(crate) use mcxn::{Gate, init_gpio_clocks, wait};
