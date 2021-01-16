//! Interrupt management
//!
//! This module implements an API for managing interrupts.

use core::sync::atomic::{compiler_fence, Ordering};

use crate::pac::NVIC_PRIO_BITS;

// Re-exports
pub use crate::pac::Interrupt;
pub use crate::pac::Interrupt::*; // needed for cortex-m-rt #[interrupt]
pub use cortex_m::interrupt::{CriticalSection, Mutex};
pub use embassy::interrupt::{declare, take, OwnedInterrupt};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Priority {
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    Level5 = 5,
    Level6 = 6,
    Level7 = 7,
    Level8 = 8,
    Level9 = 9,
    Level10 = 10,
    Level11 = 11,
    Level12 = 12,
    Level13 = 13,
    Level14 = 14,
    Level15 = 15,
}

impl From<u8> for Priority {
    fn from(priority: u8) -> Self {
        match priority >> (8 - NVIC_PRIO_BITS) {
            0 => Self::Level0,
            1 => Self::Level1,
            2 => Self::Level2,
            3 => Self::Level3,
            4 => Self::Level4,
            5 => Self::Level5,
            6 => Self::Level6,
            7 => Self::Level7,
            8 => Self::Level8,
            9 => Self::Level9,
            10 => Self::Level10,
            11 => Self::Level11,
            12 => Self::Level12,
            13 => Self::Level13,
            14 => Self::Level14,
            15 => Self::Level15,
            _ => unreachable!(),
        }
    }
}

impl From<Priority> for u8 {
    fn from(p: Priority) -> Self {
        (p as u8) << (8 - NVIC_PRIO_BITS)
    }
}

#[inline]
pub fn free<F, R>(f: F) -> R
where
    F: FnOnce(&CriticalSection) -> R,
{
    unsafe {
        // TODO: assert that we're in privileged level
        // Needed because disabling irqs in non-privileged level is a noop, which would break safety.

        let primask: u32;
        asm!("mrs {}, PRIMASK", out(reg) primask);

        asm!("cpsid i");

        // Prevent compiler from reordering operations inside/outside the critical section.
        compiler_fence(Ordering::SeqCst);

        let r = f(&CriticalSection::new());

        compiler_fence(Ordering::SeqCst);

        if primask & 1 == 0 {
            asm!("cpsie i");
        }

        r
    }
}

#[cfg(feature = "55")]
mod irqs {
    use super::*;

    declare!(WWDG);
    declare!(PVD);
    declare!(RTC_TAMP);
    declare!(RTC_WKUP);
    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_CHANNEL1);
    declare!(DMA1_CHANNEL2);
    declare!(DMA1_CHANNEL3);
    declare!(DMA1_CHANNEL4);
    declare!(DMA1_CHANNEL5);
    declare!(DMA1_CHANNEL6);
    declare!(DMA1_CHANNEL7);
    declare!(ADC1);
    declare!(USB_HP);
    declare!(USB_LP);
    declare!(C2SEV);
    declare!(COMP);
    declare!(EXTI5_9);
    declare!(TIM1_BRK);
    declare!(TIM1_UP);
    declare!(TIM1_TRG_COM_TIM17);
    declare!(TIM1_CC);
    declare!(TIM2);
    declare!(PKA);
    declare!(I2C1_EV);
    declare!(I2C1_ER);
    declare!(I2C3_EV);
    declare!(I2C3_ER);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(LPUART1);
    declare!(SAI1);
    declare!(TSC);
    declare!(EXTI10_15);
    declare!(RTC_ALARM);
    declare!(CRS_IT);
    declare!(PWR_SOTF);
    declare!(IPCC_C1_RX_IT);
    declare!(IPCC_C1_TX_IT);
    declare!(HSEM);
    declare!(LPTIM1);
    declare!(LPTIM2);
    declare!(LCD);
    declare!(QUADSPI);
    declare!(AES1);
    declare!(AES2);
    declare!(TRUE_RNG);
    declare!(FPU);
    declare!(DMA2_CH1);
    declare!(DMA2_CH2);
    declare!(DMA2_CH3);
    declare!(DMA2_CH4);
    declare!(DMA2_CH5);
    declare!(DMA2_CH6);
    declare!(DMA2_CH7);
    declare!(DMAMUX_OVR);
}

pub use irqs::*;
