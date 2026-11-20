// HAL configuration (minimal), mirroring embassy-imxrt style

use crate::clocks::config::ClocksConfig;
use crate::interrupt::Priority;

#[non_exhaustive]
pub struct Config {
    pub time_interrupt_priority: Priority,
    #[cfg(not(feature = "_mcxn"))]
    pub rtc_interrupt_priority: Priority,
    #[cfg(not(feature = "_mcxn"))]
    pub adc_interrupt_priority: Priority,
    #[cfg(not(feature = "_mcxn"))]
    pub gpio_interrupt_priority: Priority,
    #[cfg(not(feature = "_mcxn"))]
    pub wwdt_interrupt_priority: Priority,
    #[cfg(not(feature = "_mcxn"))]
    pub cdog_interrupt_priority: Priority,
    pub clock_cfg: ClocksConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            time_interrupt_priority: Priority::from(0),
            #[cfg(not(feature = "_mcxn"))]
            rtc_interrupt_priority: Priority::from(0),
            #[cfg(not(feature = "_mcxn"))]
            adc_interrupt_priority: Priority::from(0),
            #[cfg(not(feature = "_mcxn"))]
            gpio_interrupt_priority: Priority::from(0),
            #[cfg(not(feature = "_mcxn"))]
            wwdt_interrupt_priority: Priority::from(0),
            #[cfg(not(feature = "_mcxn"))]
            cdog_interrupt_priority: Priority::from(0),
            clock_cfg: ClocksConfig::default(),
        }
    }
}
