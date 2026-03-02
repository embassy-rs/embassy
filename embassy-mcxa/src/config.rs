// HAL configuration (minimal), mirroring embassy-imxrt style

#[cfg(feature = "mcxa2xx")]
use crate::clocks::config::ClocksConfig;
use crate::interrupt::Priority;

#[non_exhaustive]
pub struct Config {
    pub time_interrupt_priority: Priority,
    pub rtc_interrupt_priority: Priority,
    pub adc_interrupt_priority: Priority,
    pub gpio_interrupt_priority: Priority,
    pub wwdt_interrupt_priority: Priority,
    pub cdog_interrupt_priority: Priority,
    #[cfg(feature = "mcxa2xx")]
    pub clock_cfg: ClocksConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            time_interrupt_priority: Priority::from(0),
            rtc_interrupt_priority: Priority::from(0),
            adc_interrupt_priority: Priority::from(0),
            gpio_interrupt_priority: Priority::from(0),
            wwdt_interrupt_priority: Priority::from(0),
            cdog_interrupt_priority: Priority::from(0),
            #[cfg(feature = "mcxa2xx")]
            clock_cfg: ClocksConfig::default(),
        }
    }
}
