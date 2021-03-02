//! Interrupt management
//!
//! This module implements an API for managing interrupts compatible with
//! nrf_softdevice::interrupt. Intended for switching between the two at compile-time.

use core::sync::atomic::{compiler_fence, Ordering};

use crate::pac::NVIC_PRIO_BITS;

// Re-exports
pub use cortex_m::interrupt::{CriticalSection, Mutex};
pub use embassy::interrupt::{declare, take, Interrupt, InterruptExt};

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

#[cfg(feature = "stm32f401")]
mod irqs {
    use super::*;
    declare!(PVD);
    declare!(TAMP_STAMP);
    declare!(RTC_WKUP);
    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_STREAM0);
    declare!(DMA1_STREAM1);
    declare!(DMA1_STREAM2);
    declare!(DMA1_STREAM3);
    declare!(DMA1_STREAM4);
    declare!(DMA1_STREAM5);
    declare!(DMA1_STREAM6);
    declare!(ADC);
    declare!(EXTI9_5);
    declare!(TIM1_BRK_TIM9);
    declare!(TIM1_UP_TIM10);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_CC);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(I2C1_EV);
    declare!(I2C1_ER);
    declare!(I2C2_EV);
    declare!(I2C2_ER);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(EXTI15_10);
    declare!(RTC_ALARM);
    declare!(OTG_FS_WKUP);
    declare!(DMA1_STREAM7);
    declare!(SDIO);
    declare!(TIM5);
    declare!(SPI3);
    declare!(DMA2_STREAM0);
    declare!(DMA2_STREAM1);
    declare!(DMA2_STREAM2);
    declare!(DMA2_STREAM3);
    declare!(DMA2_STREAM4);
    declare!(OTG_FS);
    declare!(DMA2_STREAM5);
    declare!(DMA2_STREAM6);
    declare!(DMA2_STREAM7);
    declare!(USART6);
    declare!(I2C3_EV);
    declare!(I2C3_ER);
    declare!(FPU);
    declare!(SPI4);
}

#[cfg(feature = "stm32f405")]
mod irqs {
    use super::*;
    declare!(WWDG);
    declare!(PVD);
    declare!(TAMP_STAMP);
    declare!(RTC_WKUP);
    //    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_STREAM0);
    declare!(DMA1_STREAM1);
    declare!(DMA1_STREAM2);
    declare!(DMA1_STREAM3);
    declare!(DMA1_STREAM4);
    declare!(DMA1_STREAM5);
    declare!(DMA1_STREAM6);
    declare!(ADC);
    declare!(CAN1_TX);
    declare!(CAN1_RX0);
    declare!(CAN1_RX1);
    declare!(CAN1_SCE);
    declare!(EXTI9_5);
    declare!(TIM1_BRK_TIM9);
    declare!(TIM1_UP_TIM10);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_CC);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(I2C1_EV);
    declare!(I2C1_ER);
    declare!(I2C2_EV);
    declare!(I2C2_ER);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(USART3);
    declare!(EXTI15_10);
    declare!(RTC_ALARM);
    declare!(OTG_FS_WKUP);
    declare!(TIM8_BRK_TIM12);
    declare!(TIM8_UP_TIM13);
    declare!(TIM8_TRG_COM_TIM14);
    declare!(TIM8_CC);
    declare!(DMA1_STREAM7);
    //    declare!(FMC);
    declare!(SDIO);
    declare!(TIM5);
    declare!(SPI3);
    declare!(UART4);
    declare!(UART5);
    declare!(TIM6_DAC);
    declare!(TIM7);
    declare!(DMA2_STREAM0);
    declare!(DMA2_STREAM1);
    declare!(DMA2_STREAM2);
    declare!(DMA2_STREAM3);
    declare!(DMA2_STREAM4);
    declare!(ETH);
    declare!(ETH_WKUP);
    declare!(CAN2_TX);
    declare!(CAN2_RX0);
    declare!(CAN2_RX1);
    declare!(CAN2_SCE);
    declare!(OTG_FS);
    declare!(DMA2_STREAM5);
    declare!(DMA2_STREAM6);
    declare!(DMA2_STREAM7);
    declare!(USART6);
    declare!(I2C3_EV);
    declare!(I2C3_ER);
    declare!(OTG_HS_EP1_OUT);
    declare!(OTG_HS_EP1_IN);
    declare!(OTG_HS_WKUP);
    declare!(OTG_HS);
    declare!(DCMI);
    declare!(CRYP);
    declare!(HASH_RNG);
    declare!(FPU);
    //    declare!(UART7);
    //    declare!(UART8);
    //    declare!(SPI4);
    //    declare!(SPI5);
    //    declare!(SPI6);
    //    declare!(SAI1);
    declare!(LCD_TFT);
    declare!(LCD_TFT_1);
    //    declare!(DMA2D);
}

#[cfg(feature = "stm32f407")]
mod irqs {
    use super::*;

    declare!(WWDG);
    declare!(PVD);
    declare!(TAMP_STAMP);
    declare!(RTC_WKUP);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_STREAM0);
    declare!(DMA1_STREAM1);
    declare!(DMA1_STREAM2);
    declare!(DMA1_STREAM3);
    declare!(DMA1_STREAM4);
    declare!(DMA1_STREAM5);
    declare!(DMA1_STREAM6);
    declare!(ADC);
    declare!(CAN1_TX);
    declare!(CAN1_RX0);
    declare!(CAN1_RX1);
    declare!(CAN1_SCE);
    declare!(EXTI9_5);
    declare!(TIM1_BRK_TIM9);
    declare!(TIM1_UP_TIM10);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_CC);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(I2C1_EV);
    declare!(I2C1_ER);
    declare!(I2C2_EV);
    declare!(I2C2_ER);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(USART3);
    declare!(EXTI15_10);
    declare!(RTC_ALARM);
    declare!(OTG_FS_WKUP);
    declare!(TIM8_BRK_TIM12);
    declare!(TIM8_UP_TIM13);
    declare!(TIM8_TRG_COM_TIM14);
    declare!(TIM8_CC);
    declare!(DMA1_STREAM7);
    declare!(FSMC);
    declare!(SDIO);
    declare!(TIM5);
    declare!(SPI3);
    declare!(UART4);
    declare!(UART5);
    declare!(TIM6_DAC);
    declare!(TIM7);
    declare!(DMA2_STREAM0);
    declare!(DMA2_STREAM1);
    declare!(DMA2_STREAM2);
    declare!(DMA2_STREAM3);
    declare!(DMA2_STREAM4);
    declare!(ETH);
    declare!(ETH_WKUP);
    declare!(CAN2_TX);
    declare!(CAN2_RX0);
    declare!(CAN2_RX1);
    declare!(CAN2_SCE);
    declare!(OTG_FS);
    declare!(DMA2_STREAM5);
    declare!(DMA2_STREAM6);
    declare!(DMA2_STREAM7);
    declare!(USART6);
    declare!(I2C3_EV);
    declare!(I2C3_ER);
    declare!(OTG_HS_EP1_OUT);
    declare!(OTG_HS_EP1_IN);
    declare!(OTG_HS_WKUP);
    declare!(OTG_HS);
    declare!(DCMI);
    declare!(CRYP);
    declare!(HASH_RNG);
    declare!(FPU);
    declare!(LCD_TFT);
    declare!(LCD_TFT_1);
}

#[cfg(feature = "stm32f410")]
mod irqs {
    use super::*;

    declare!(WWDG);
    declare!(PVD);
    declare!(TAMP_STAMP);
    declare!(RTC_WKUP);
    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_STREAM0);
    declare!(DMA1_STREAM1);
    declare!(DMA1_STREAM2);
    declare!(DMA1_STREAM3);
    declare!(DMA1_STREAM4);
    declare!(DMA1_STREAM5);
    declare!(DMA1_STREAM6);
    declare!(ADC);
    declare!(EXTI9_5);
    declare!(TIM1_BRK_TIM9);
    declare!(PWM1_UP);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_CC);
    declare!(I2C1_EV);
    declare!(I2C1_ER);
    declare!(I2C2_EV);
    declare!(I2C2_ER);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(EXTI15_10);
    declare!(RTC_ALARM);
    declare!(DMA1_STREAM7);
    declare!(TIM5);
    declare!(TIM6_DAC1);
    declare!(DMA2_STREAM0);
    declare!(DMA2_STREAM1);
    declare!(DMA2_STREAM2);
    declare!(DMA2_STREAM3);
    declare!(DMA2_STREAM4);
    declare!(EXTI19);
    declare!(DMA2_STREAM5);
    declare!(DMA2_STREAM6);
    declare!(DMA2_STREAM7);
    declare!(USART6);
    declare!(EXTI20);
    declare!(RNG);
    declare!(FPU);
    declare!(SPI5);
    declare!(I2C4_EV);
    declare!(I2C4_ER);
    declare!(LPTIM1);
}

#[cfg(feature = "stm32f411")]
mod irqs {
    use super::*;

    declare!(WWDG);
    declare!(PVD);
    declare!(TAMP_STAMP);
    declare!(RTC_WKUP);
    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_STREAM0);
    declare!(DMA1_STREAM1);
    declare!(DMA1_STREAM2);
    declare!(DMA1_STREAM3);
    declare!(DMA1_STREAM4);
    declare!(DMA1_STREAM5);
    declare!(DMA1_STREAM6);
    declare!(ADC);
    declare!(EXTI9_5);
    declare!(TIM1_BRK_TIM9);
    declare!(TIM1_UP_TIM10);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_CC);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(I2C1_EV);
    declare!(I2C1_ER);
    declare!(I2C2_EV);
    declare!(I2C2_ER);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(EXTI15_10);
    declare!(RTC_ALARM);
    declare!(OTG_FS_WKUP);
    declare!(DMA1_STREAM7);
    declare!(SDIO);
    declare!(TIM5);
    declare!(SPI3);
    declare!(DMA2_STREAM0);
    declare!(DMA2_STREAM1);
    declare!(DMA2_STREAM2);
    declare!(DMA2_STREAM3);
    declare!(DMA2_STREAM4);
    declare!(OTG_FS);
    declare!(DMA2_STREAM5);
    declare!(DMA2_STREAM6);
    declare!(DMA2_STREAM7);
    declare!(USART6);
    declare!(I2C3_EV);
    declare!(I2C3_ER);
    declare!(FPU);
    declare!(SPI4);
    declare!(SPI5);
}

#[cfg(feature = "stm32f412")]
mod irqs {
    use super::*;

    declare!(WWDG);
    declare!(PVD);
    declare!(TAMP_STAMP);
    declare!(RTC_WKUP);
    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_STREAM0);
    declare!(DMA1_STREAM1);
    declare!(DMA1_STREAM2);
    declare!(DMA1_STREAM3);
    declare!(DMA1_STREAM4);
    declare!(DMA1_STREAM5);
    declare!(DMA1_STREAM6);
    declare!(ADC);
    declare!(CAN1_TX);
    declare!(CAN1_RX0);
    declare!(CAN1_RX1);
    declare!(CAN1_SCE);
    declare!(EXTI9_5);
    declare!(TIM1_BRK_TIM9);
    declare!(TIM1_UP_TIM10);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_CC);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(I2C1_EV);
    declare!(I2C1_ER);
    declare!(I2C2_EV);
    declare!(I2C2_ER);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(USART3);
    declare!(EXTI15_10);
    declare!(RTC_ALARM);
    declare!(OTG_FS_WKUP);
    declare!(TIM12);
    declare!(TIM13);
    declare!(TIM14);
    declare!(TIM8_CC);
    declare!(DMA1_STREAM7);
    declare!(FSMC);
    declare!(SDIO);
    declare!(TIM5);
    declare!(SPI3);
    declare!(TIM6_DACUNDER);
    declare!(TIM7);
    declare!(DMA2_STREAM0);
    declare!(DMA2_STREAM1);
    declare!(DMA2_STREAM2);
    declare!(DMA2_STREAM3);
    declare!(DMA2_STREAM4);
    declare!(DFSDM1_FLT0);
    declare!(DFSDM1_FLT1);
    declare!(CAN2_TX);
    declare!(CAN2_RX0);
    declare!(CAN2_RX1);
    declare!(CAN2_SCE);
    declare!(OTG_FS);
    declare!(DMA2_STREAM5);
    declare!(DMA2_STREAM6);
    declare!(DMA2_STREAM7);
    declare!(USART6);
    declare!(I2C3_EV);
    declare!(I2C3_ER);
    declare!(HASH_RNG);
    declare!(FPU);
    declare!(SPI4);
    declare!(SPI5);
    declare!(QUAD_SPI);
    declare!(I2CFMP1_EVENT);
    declare!(I2CFMP1_ERROR);
}

#[cfg(feature = "stm32f413")]
mod irqs {
    use super::*;

    declare!(PVD);
    declare!(TAMP_STAMP);
    declare!(RTC_WKUP);
    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_STREAM0);
    declare!(DMA1_STREAM1);
    declare!(DMA1_STREAM2);
    declare!(DMA1_STREAM3);
    declare!(DMA1_STREAM4);
    declare!(DMA1_STREAM5);
    declare!(DMA1_STREAM6);
    declare!(ADC);
    declare!(CAN1_TX);
    declare!(CAN1_RX0);
    declare!(CAN1_RX1);
    declare!(CAN1_SCE);
    declare!(EXTI9_5);
    declare!(TIM1_BRK_TIM9);
    declare!(TIM1_UP_TIM10);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_CC);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(I2C1_EVT);
    declare!(I2C1_ERR);
    declare!(I2C2_EVT);
    declare!(I2C2_ERR);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(USART3);
    declare!(EXTI15_10);
    declare!(EXTI17_RTC_ALARM);
    declare!(TIM8_BRK_TIM12);
    declare!(TIM8_UP_TIM13);
    declare!(TIM8_TRG_COM_TIM14);
    declare!(TIM8_CC);
    declare!(DMA1_STREAM7);
    declare!(FSMC);
    declare!(SDIO);
    declare!(TIM5);
    declare!(SPI3);
    declare!(USART4);
    declare!(UART5);
    declare!(TIM6_GLB_IT_DAC1_DAC2);
    declare!(TIM7);
    declare!(DMA2_STREAM0);
    declare!(DMA2_STREAM1);
    declare!(DMA2_STREAM2);
    declare!(DMA2_STREAM3);
    declare!(DMA2_STREAM4);
    declare!(DFSDM1_FLT0);
    declare!(DFSDM1_FLT1);
    declare!(CAN2_TX);
    declare!(CAN2_RX0);
    declare!(CAN2_RX1);
    declare!(CAN2_SCE);
    declare!(OTG_FS);
    declare!(DMA2_STREAM5);
    declare!(DMA2_STREAM6);
    declare!(DMA2_STREAM7);
    declare!(USART6);
    declare!(I2C3_EV);
    declare!(I2C3_ER);
    declare!(CAN3_TX);
    declare!(CAN3_RX0);
    declare!(CAN3_RX1);
    declare!(CAN3_SCE);
    declare!(CRYPTO);
    declare!(RNG);
    declare!(FPU);
    declare!(USART7);
    declare!(USART8);
    declare!(SPI4);
    declare!(SPI5);
    declare!(SAI1);
    declare!(UART9);
    declare!(UART10);
    declare!(QUADSPI);
    declare!(I2CFMP1EVENT);
    declare!(I2CFMP1ERROR);
    declare!(LPTIM1_OR_IT_EIT_23);
    declare!(DFSDM2_FILTER1);
    declare!(DFSDM2_FILTER2);
    declare!(DFSDM2_FILTER3);
    declare!(DFSDM2_FILTER4);
}

#[cfg(feature = "stm32f427")]
mod irqs {
    use super::*;

    declare!(WWDG);
    declare!(PVD);
    declare!(TAMP_STAMP);
    declare!(RTC_WKUP);
    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_STREAM0);
    declare!(DMA1_STREAM1);
    declare!(DMA1_STREAM2);
    declare!(DMA1_STREAM3);
    declare!(DMA1_STREAM4);
    declare!(DMA1_STREAM5);
    declare!(DMA1_STREAM6);
    declare!(ADC);
    declare!(CAN1_TX);
    declare!(CAN1_RX0);
    declare!(CAN1_RX1);
    declare!(CAN1_SCE);
    declare!(EXTI9_5);
    declare!(TIM1_BRK_TIM9);
    declare!(TIM1_UP_TIM10);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_CC);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(I2C1_EV);
    declare!(I2C1_ER);
    declare!(I2C2_EV);
    declare!(I2C2_ER);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(USART3);
    declare!(EXTI15_10);
    declare!(RTC_ALARM);
    declare!(OTG_FS_WKUP);
    declare!(TIM8_BRK_TIM12);
    declare!(TIM8_UP_TIM13);
    declare!(TIM8_TRG_COM_TIM14);
    declare!(TIM8_CC);
    declare!(DMA1_STREAM7);
    declare!(FMC);
    declare!(SDIO);
    declare!(TIM5);
    declare!(SPI3);
    declare!(UART4);
    declare!(UART5);
    declare!(TIM6_DAC);
    declare!(TIM7);
    declare!(DMA2_STREAM0);
    declare!(DMA2_STREAM1);
    declare!(DMA2_STREAM2);
    declare!(DMA2_STREAM3);
    declare!(DMA2_STREAM4);
    declare!(ETH);
    declare!(ETH_WKUP);
    declare!(CAN2_TX);
    declare!(CAN2_RX0);
    declare!(CAN2_RX1);
    declare!(CAN2_SCE);
    declare!(OTG_FS);
    declare!(DMA2_STREAM5);
    declare!(DMA2_STREAM6);
    declare!(DMA2_STREAM7);
    declare!(USART6);
    declare!(I2C3_EV);
    declare!(I2C3_ER);
    declare!(OTG_HS_EP1_OUT);
    declare!(OTG_HS_EP1_IN);
    declare!(OTG_HS_WKUP);
    declare!(OTG_HS);
    declare!(DCMI);
    declare!(CRYP);
    declare!(HASH_RNG);
    declare!(FPU);
    declare!(UART7);
    declare!(UART8);
    declare!(SPI4);
    declare!(SPI5);
    declare!(SPI6);
    declare!(LCD_TFT);
    declare!(LCD_TFT_1);
}

#[cfg(feature = "stm32f429")]
mod irqs {
    use super::*;

    declare!(WWDG);
    declare!(PVD);
    declare!(TAMP_STAMP);
    declare!(RTC_WKUP);
    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_STREAM0);
    declare!(DMA1_STREAM1);
    declare!(DMA1_STREAM2);
    declare!(DMA1_STREAM3);
    declare!(DMA1_STREAM4);
    declare!(DMA1_STREAM5);
    declare!(DMA1_STREAM6);
    declare!(ADC);
    declare!(CAN1_TX);
    declare!(CAN1_RX0);
    declare!(CAN1_RX1);
    declare!(CAN1_SCE);
    declare!(EXTI9_5);
    declare!(TIM1_BRK_TIM9);
    declare!(TIM1_UP_TIM10);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_CC);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(I2C1_EV);
    declare!(I2C1_ER);
    declare!(I2C2_EV);
    declare!(I2C2_ER);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(USART3);
    declare!(EXTI15_10);
    declare!(RTC_ALARM);
    declare!(OTG_FS_WKUP);
    declare!(TIM8_BRK_TIM12);
    declare!(TIM8_UP_TIM13);
    declare!(TIM8_TRG_COM_TIM14);
    declare!(TIM8_CC);
    declare!(DMA1_STREAM7);
    declare!(FMC);
    declare!(SDIO);
    declare!(TIM5);
    declare!(SPI3);
    declare!(UART4);
    declare!(UART5);
    declare!(TIM6_DAC);
    declare!(TIM7);
    declare!(DMA2_STREAM0);
    declare!(DMA2_STREAM1);
    declare!(DMA2_STREAM2);
    declare!(DMA2_STREAM3);
    declare!(DMA2_STREAM4);
    declare!(ETH);
    declare!(ETH_WKUP);
    declare!(CAN2_TX);
    declare!(CAN2_RX0);
    declare!(CAN2_RX1);
    declare!(CAN2_SCE);
    declare!(OTG_FS);
    declare!(DMA2_STREAM5);
    declare!(DMA2_STREAM6);
    declare!(DMA2_STREAM7);
    declare!(USART6);
    declare!(I2C3_EV);
    declare!(I2C3_ER);
    declare!(OTG_HS_EP1_OUT);
    declare!(OTG_HS_EP1_IN);
    declare!(OTG_HS_WKUP);
    declare!(OTG_HS);
    declare!(DCMI);
    declare!(CRYP);
    declare!(HASH_RNG);
    declare!(FPU);
    declare!(UART7);
    declare!(UART8);
    declare!(SPI4);
    declare!(SPI5);
    declare!(SPI6);
    declare!(SAI1);
    declare!(LCD_TFT);
    declare!(LCD_TFT_1);
    declare!(DMA2D);
}

#[cfg(feature = "stm32f446")]
mod irqs {
    use super::*;

    declare!(WWDG);
    declare!(TAMP_STAMP);
    declare!(RTC_WKUP);
    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_STREAM0);
    declare!(DMA1_STREAM1);
    declare!(DMA1_STREAM2);
    declare!(DMA1_STREAM3);
    declare!(DMA1_STREAM4);
    declare!(DMA1_STREAM5);
    declare!(DMA1_STREAM6);
    declare!(ADC);
    declare!(CAN1_TX);
    declare!(CAN1_RX0);
    declare!(CAN1_RX1);
    declare!(CAN1_SCE);
    declare!(EXTI9_5);
    declare!(TIM1_BRK_TIM9);
    declare!(TIM1_UP_TIM10);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_CC);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(I2C1_EV);
    declare!(I2C1_ER);
    declare!(I2C2_EV);
    declare!(I2C2_ER);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(USART3);
    declare!(EXTI15_10);
    declare!(RTC_ALARM);
    declare!(OTG_FS_WKUP);
    declare!(TIM8_BRK_TIM12);
    declare!(TIM8_UP_TIM13);
    declare!(TIM8_TRG_COM_TIM14);
    declare!(TIM8_CC);
    declare!(DMA1_STREAM7);
    declare!(FMC);
    declare!(SDIO);
    declare!(TIM5);
    declare!(SPI3);
    declare!(UART4);
    declare!(UART5);
    declare!(TIM6_DAC);
    declare!(TIM7);
    declare!(DMA2_STREAM0);
    declare!(DMA2_STREAM1);
    declare!(DMA2_STREAM2);
    declare!(DMA2_STREAM3);
    declare!(DMA2_STREAM4);
    declare!(ETH);
    declare!(ETH_WKUP);
    declare!(CAN2_TX);
    declare!(CAN2_RX0);
    declare!(CAN2_RX1);
    declare!(CAN2_SCE);
    declare!(OTG_FS);
    declare!(DMA2_STREAM5);
    declare!(DMA2_STREAM6);
    declare!(DMA2_STREAM7);
    declare!(USART6);
    declare!(I2C3_EV);
    declare!(I2C3_ER);
    declare!(DCMI);
    declare!(FPU);
    declare!(UART7);
    declare!(UART8);
    declare!(SPI4);
    declare!(LCD_TFT);
    declare!(LCD_TFT_1);
}

#[cfg(feature = "stm32f469")]
mod irqs {
    use super::*;

    declare!(WWDG);
    declare!(PVD);
    declare!(TAMP_STAMP);
    declare!(RTC_WKUP);
    declare!(FLASH);
    declare!(RCC);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(DMA1_STREAM0);
    declare!(DMA1_STREAM1);
    declare!(DMA1_STREAM2);
    declare!(DMA1_STREAM3);
    declare!(DMA1_STREAM4);
    declare!(DMA1_STREAM5);
    declare!(DMA1_STREAM6);
    declare!(ADC);
    declare!(CAN1_TX);
    declare!(CAN1_RX0);
    declare!(CAN1_RX1);
    declare!(CAN1_SCE);
    declare!(EXTI9_5);
    declare!(TIM1_BRK_TIM9);
    declare!(TIM1_UP_TIM10);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_CC);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(I2C1_EV);
    declare!(I2C1_ER);
    declare!(I2C2_EV);
    declare!(I2C2_ER);
    declare!(SPI1);
    declare!(SPI2);
    declare!(USART1);
    declare!(USART2);
    declare!(USART3);
    declare!(EXTI15_10);
    declare!(RTC_ALARM);
    declare!(OTG_FS_WKUP);
    declare!(TIM8_BRK_TIM12);
    declare!(TIM8_UP_TIM13);
    declare!(TIM8_TRG_COM_TIM14);
    declare!(TIM8_CC);
    declare!(DMA1_STREAM7);
    declare!(FMC);
    declare!(SDIO);
    declare!(TIM5);
    declare!(SPI3);
    declare!(UART4);
    declare!(UART5);
    declare!(TIM6_DAC);
    declare!(TIM7);
    declare!(DMA2_STREAM0);
    declare!(DMA2_STREAM1);
    declare!(DMA2_STREAM2);
    declare!(DMA2_STREAM3);
    declare!(DMA2_STREAM4);
    declare!(ETH);
    declare!(ETH_WKUP);
    declare!(CAN2_TX);
    declare!(CAN2_RX0);
    declare!(CAN2_RX1);
    declare!(CAN2_SCE);
    declare!(OTG_FS);
    declare!(DMA2_STREAM5);
    declare!(DMA2_STREAM6);
    declare!(DMA2_STREAM7);
    declare!(USART6);
    declare!(I2C3_EV);
    declare!(I2C3_ER);
    declare!(OTG_HS_EP1_OUT);
    declare!(OTG_HS_EP1_IN);
    declare!(OTG_HS_WKUP);
    declare!(OTG_HS);
    declare!(DCMI);
    declare!(CRYP);
    declare!(HASH_RNG);
    declare!(FPU);
    declare!(UART7);
    declare!(UART8);
    declare!(SPI4);
    declare!(SPI5);
    declare!(SPI6);
    declare!(SAI1);
    declare!(LCD_TFT);
    declare!(LCD_TFT_1);
    declare!(DMA2D);
    declare!(QUADSPI);
    declare!(DSIHOST);
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
