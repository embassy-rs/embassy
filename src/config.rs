// HAL configuration (minimal), mirroring embassy-imxrt style

use crate::interrupt::Priority;

#[non_exhaustive]
pub struct Config {
    pub time_interrupt_priority: Priority,
    pub rtc_interrupt_priority: Priority,
    pub adc_interrupt_priority: Priority,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            time_interrupt_priority: Priority::from(0),
            rtc_interrupt_priority: Priority::from(0),
            adc_interrupt_priority: Priority::from(0),
        }
    }
}
