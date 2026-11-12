//! HAL configuration, passed to [`crate::init()`].

use crate::clocks::config::ClocksConfig;

/// Configuration for [`crate::init()`].
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Core clock configuration.
    pub clock_cfg: ClocksConfig,
}
