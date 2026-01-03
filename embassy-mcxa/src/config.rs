// HAL configuration (minimal), mirroring embassy-imxrt style

use crate::clocks::config::ClocksConfig;
use crate::interrupt::Priority;

#[non_exhaustive]
pub struct Config {
    #[cfg(feature = "time")]
    pub time_interrupt_priority: Priority,
    pub rtc_interrupt_priority: Priority,
    pub adc_interrupt_priority: Priority,
    pub gpio_interrupt_priority: Priority,
    pub wwdt_interrupt_priority: Priority,
    pub clock_cfg: ClocksConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            #[cfg(feature = "time")]
            time_interrupt_priority: Priority::from(0),
            rtc_interrupt_priority: Priority::from(0),
            adc_interrupt_priority: Priority::from(0),
            gpio_interrupt_priority: Priority::from(0),
            wwdt_interrupt_priority: Priority::from(0),
            clock_cfg: ClocksConfig::default(),
        }
    }
}
