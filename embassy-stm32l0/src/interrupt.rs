//! Interrupt management
use crate::pac::NVIC_PRIO_BITS;

// Re-exports
pub use cortex_m::interrupt::{CriticalSection, Mutex};
pub use embassy::interrupt::{declare, take, Interrupt};

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

#[cfg(feature = "stm32l0x1")]
mod irqs {
    use super::*;
    declare!(WWDG);
    declare!(PVD);
    declare!(RTC);
    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0_1);
    declare!(EXTI2_3);
    declare!(EXTI4_15);
    declare!(DMA1_CHANNEL1);
    declare!(DMA1_CHANNEL2_3);
    declare!(DMA1_CHANNEL4_7);
    declare!(ADC_COMP);
    declare!(LPTIM1);
    declare!(USART4_USART5);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM6);
    declare!(TIM7);
    declare!(TIM21);
    declare!(I2C3);
    declare!(TIM22);
    declare!(I2C1);
    declare!(I2C2);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(AES_RNG_LPUART1);
}

#[cfg(feature = "stm32l0x2")]
mod irqs {
    use super::*;
    declare!(WWDG);
    declare!(PVD);
    declare!(RTC);
    declare!(RCC);
    declare!(EXTI0_1);
    declare!(EXTI2_3);
    declare!(EXTI4_15);
    declare!(TSC);
    declare!(DMA1_CHANNEL1);
    declare!(DMA1_CHANNEL2_3);
    declare!(DMA1_CHANNEL4_7);
    declare!(ADC_COMP);
    declare!(LPTIM1);
    declare!(USART4_USART5);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM6_DAC);
    declare!(TIM7);
    declare!(TIM21);
    declare!(I2C3);
    declare!(TIM22);
    declare!(I2C1);
    declare!(I2C2);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(AES_RNG_LPUART1);
    declare!(USB);
}

#[cfg(feature = "stm32l0x3")]
mod irqs {
    use super::*;
    declare!(WWDG);
    declare!(PVD);
    declare!(RTC);
    declare!(RCC);
    declare!(EXTI0_1);
    declare!(EXTI2_3);
    declare!(EXTI4_15);
    declare!(TSC);
    declare!(DMA1_CHANNEL1);
    declare!(DMA1_CHANNEL2_3);
    declare!(DMA1_CHANNEL4_7);
    declare!(ADC_COMP);
    declare!(LPTIM1);
    declare!(USART4_USART5);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM6_DAC);
    declare!(TIM7);
    declare!(TIM21);
    declare!(I2C3);
    declare!(TIM22);
    declare!(I2C1);
    declare!(I2C2);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(AES_RNG_LPUART1);
    declare!(LCD);
    declare!(USB);
}

pub use irqs::*;
