//! Clock configuration.

/// Configuration for the core system clocks, passed to [`super::init()`].
///
/// It has no fields: [`super::init()`] leaves the main clock on its reset
/// default (`fro_hf`, RM "Chapter 33 Clocking") with `AHBCLKDIV` as found.
/// `#[non_exhaustive]` lets settings be added without breaking callers that
/// build it with `Default`.
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct ClocksConfig {}
