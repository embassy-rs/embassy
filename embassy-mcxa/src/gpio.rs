//! Family-selected GPIO support.

#[cfg(any(feature = "_mcxa2xx", feature = "_mcxa5xx"))]
#[path = "gpio/mcxa.rs"]
mod mcxa;
#[cfg(any(feature = "_mcxa2xx", feature = "_mcxa5xx"))]
pub use mcxa::*;
