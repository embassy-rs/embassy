//! Interrupt management
//!
//! This module implements an API for managing interrupts compatible with
//! nrf_softdevice::interrupt. Intended for switching between the two at compile-time.

// Re-exports
pub use embassy::interrupt::{declare, take, Interrupt};
pub use embassy_extras::interrupt::Priority3 as Priority;

#[cfg(feature = "52810")]
mod irqs {
    use super::*;
    declare!(POWER_CLOCK);
    declare!(RADIO);
    declare!(UARTE0_UART0);
    declare!(TWIM0_TWIS0_TWI0);
    declare!(SPIM0_SPIS0_SPI0);
    declare!(GPIOTE);
    declare!(SAADC);
    declare!(TIMER0);
    declare!(TIMER1);
    declare!(TIMER2);
    declare!(RTC0);
    declare!(TEMP);
    declare!(RNG);
    declare!(ECB);
    declare!(CCM_AAR);
    declare!(WDT);
    declare!(RTC1);
    declare!(QDEC);
    declare!(COMP);
    declare!(SWI0_EGU0);
    declare!(SWI1_EGU1);
    declare!(SWI2);
    declare!(SWI3);
    declare!(SWI4);
    declare!(SWI5);
    declare!(PWM0);
    declare!(PDM);
}

#[cfg(feature = "52811")]
mod irqs {
    use super::*;
    declare!(POWER_CLOCK);
    declare!(RADIO);
    declare!(UARTE0_UART0);
    declare!(TWIM0_TWIS0_TWI0_SPIM1_SPIS1_SPI1);
    declare!(SPIM0_SPIS0_SPI0);
    declare!(GPIOTE);
    declare!(SAADC);
    declare!(TIMER0);
    declare!(TIMER1);
    declare!(TIMER2);
    declare!(RTC0);
    declare!(TEMP);
    declare!(RNG);
    declare!(ECB);
    declare!(CCM_AAR);
    declare!(WDT);
    declare!(RTC1);
    declare!(QDEC);
    declare!(COMP);
    declare!(SWI0_EGU0);
    declare!(SWI1_EGU1);
    declare!(SWI2);
    declare!(SWI3);
    declare!(SWI4);
    declare!(SWI5);
    declare!(PWM0);
    declare!(PDM);
}

#[cfg(feature = "52832")]
mod irqs {
    use super::*;
    declare!(POWER_CLOCK);
    declare!(RADIO);
    declare!(UARTE0_UART0);
    declare!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    declare!(SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);
    declare!(NFCT);
    declare!(GPIOTE);
    declare!(SAADC);
    declare!(TIMER0);
    declare!(TIMER1);
    declare!(TIMER2);
    declare!(RTC0);
    declare!(TEMP);
    declare!(RNG);
    declare!(ECB);
    declare!(CCM_AAR);
    declare!(WDT);
    declare!(RTC1);
    declare!(QDEC);
    declare!(COMP_LPCOMP);
    declare!(SWI0_EGU0);
    declare!(SWI1_EGU1);
    declare!(SWI2_EGU2);
    declare!(SWI3_EGU3);
    declare!(SWI4_EGU4);
    declare!(SWI5_EGU5);
    declare!(TIMER3);
    declare!(TIMER4);
    declare!(PWM0);
    declare!(PDM);
    declare!(MWU);
    declare!(PWM1);
    declare!(PWM2);
    declare!(SPIM2_SPIS2_SPI2);
    declare!(RTC2);
    declare!(I2S);
    declare!(FPU);
}

#[cfg(feature = "52833")]
mod irqs {
    use super::*;
    declare!(POWER_CLOCK);
    declare!(RADIO);
    declare!(UARTE0_UART0);
    declare!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    declare!(SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);
    declare!(NFCT);
    declare!(GPIOTE);
    declare!(SAADC);
    declare!(TIMER0);
    declare!(TIMER1);
    declare!(TIMER2);
    declare!(RTC0);
    declare!(TEMP);
    declare!(RNG);
    declare!(ECB);
    declare!(CCM_AAR);
    declare!(WDT);
    declare!(RTC1);
    declare!(QDEC);
    declare!(COMP_LPCOMP);
    declare!(SWI0_EGU0);
    declare!(SWI1_EGU1);
    declare!(SWI2_EGU2);
    declare!(SWI3_EGU3);
    declare!(SWI4_EGU4);
    declare!(SWI5_EGU5);
    declare!(TIMER3);
    declare!(TIMER4);
    declare!(PWM0);
    declare!(PDM);
    declare!(MWU);
    declare!(PWM1);
    declare!(PWM2);
    declare!(SPIM2_SPIS2_SPI2);
    declare!(RTC2);
    declare!(I2S);
    declare!(FPU);
    declare!(USBD);
    declare!(UARTE1);
    declare!(PWM3);
    declare!(SPIM3);
}

#[cfg(feature = "52840")]
mod irqs {
    use super::*;
    declare!(POWER_CLOCK);
    declare!(RADIO);
    declare!(UARTE0_UART0);
    declare!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    declare!(SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);
    declare!(NFCT);
    declare!(GPIOTE);
    declare!(SAADC);
    declare!(TIMER0);
    declare!(TIMER1);
    declare!(TIMER2);
    declare!(RTC0);
    declare!(TEMP);
    declare!(RNG);
    declare!(ECB);
    declare!(CCM_AAR);
    declare!(WDT);
    declare!(RTC1);
    declare!(QDEC);
    declare!(COMP_LPCOMP);
    declare!(SWI0_EGU0);
    declare!(SWI1_EGU1);
    declare!(SWI2_EGU2);
    declare!(SWI3_EGU3);
    declare!(SWI4_EGU4);
    declare!(SWI5_EGU5);
    declare!(TIMER3);
    declare!(TIMER4);
    declare!(PWM0);
    declare!(PDM);
    declare!(MWU);
    declare!(PWM1);
    declare!(PWM2);
    declare!(SPIM2_SPIS2_SPI2);
    declare!(RTC2);
    declare!(I2S);
    declare!(FPU);
    declare!(USBD);
    declare!(UARTE1);
    declare!(QSPI);
    declare!(CRYPTOCELL);
    declare!(PWM3);
    declare!(SPIM3);
}

pub use irqs::*;
