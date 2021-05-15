#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_snake_case)]

pub fn GPIO(n: usize) -> gpio::Gpio {
    gpio::Gpio((0x48000000 + 0x400 * n) as _)
}
pub const DMA1: dma::Dma = dma::Dma(0x40020000 as _);
impl_dma_channel!(DMA1_CH0, DMA1, 0);
impl_dma_channel!(DMA1_CH1, DMA1, 1);
impl_dma_channel!(DMA1_CH2, DMA1, 2);
impl_dma_channel!(DMA1_CH3, DMA1, 3);
impl_dma_channel!(DMA1_CH4, DMA1, 4);
impl_dma_channel!(DMA1_CH5, DMA1, 5);
impl_dma_channel!(DMA1_CH6, DMA1, 6);
impl_dma_channel!(DMA1_CH7, DMA1, 7);
pub const DMA2: dma::Dma = dma::Dma(0x40020400 as _);
impl_dma_channel!(DMA2_CH0, DMA2, 0);
impl_dma_channel!(DMA2_CH1, DMA2, 1);
impl_dma_channel!(DMA2_CH2, DMA2, 2);
impl_dma_channel!(DMA2_CH3, DMA2, 3);
impl_dma_channel!(DMA2_CH4, DMA2, 4);
impl_dma_channel!(DMA2_CH5, DMA2, 5);
impl_dma_channel!(DMA2_CH6, DMA2, 6);
impl_dma_channel!(DMA2_CH7, DMA2, 7);
pub const EXTI: exti::Exti = exti::Exti(0x40010400 as _);
pub const GPIOA: gpio::Gpio = gpio::Gpio(0x48000000 as _);
impl_gpio_pin!(PA0, 0, 0, EXTI0);
impl_gpio_pin!(PA1, 0, 1, EXTI1);
impl_gpio_pin!(PA2, 0, 2, EXTI2);
impl_gpio_pin!(PA3, 0, 3, EXTI3);
impl_gpio_pin!(PA4, 0, 4, EXTI4);
impl_gpio_pin!(PA5, 0, 5, EXTI5);
impl_gpio_pin!(PA6, 0, 6, EXTI6);
impl_gpio_pin!(PA7, 0, 7, EXTI7);
impl_gpio_pin!(PA8, 0, 8, EXTI8);
impl_gpio_pin!(PA9, 0, 9, EXTI9);
impl_gpio_pin!(PA10, 0, 10, EXTI10);
impl_gpio_pin!(PA11, 0, 11, EXTI11);
impl_gpio_pin!(PA12, 0, 12, EXTI12);
impl_gpio_pin!(PA13, 0, 13, EXTI13);
impl_gpio_pin!(PA14, 0, 14, EXTI14);
impl_gpio_pin!(PA15, 0, 15, EXTI15);
pub const GPIOB: gpio::Gpio = gpio::Gpio(0x48000400 as _);
impl_gpio_pin!(PB0, 1, 0, EXTI0);
impl_gpio_pin!(PB1, 1, 1, EXTI1);
impl_gpio_pin!(PB2, 1, 2, EXTI2);
impl_gpio_pin!(PB3, 1, 3, EXTI3);
impl_gpio_pin!(PB4, 1, 4, EXTI4);
impl_gpio_pin!(PB5, 1, 5, EXTI5);
impl_gpio_pin!(PB6, 1, 6, EXTI6);
impl_gpio_pin!(PB7, 1, 7, EXTI7);
impl_gpio_pin!(PB8, 1, 8, EXTI8);
impl_gpio_pin!(PB9, 1, 9, EXTI9);
impl_gpio_pin!(PB10, 1, 10, EXTI10);
impl_gpio_pin!(PB11, 1, 11, EXTI11);
impl_gpio_pin!(PB12, 1, 12, EXTI12);
impl_gpio_pin!(PB13, 1, 13, EXTI13);
impl_gpio_pin!(PB14, 1, 14, EXTI14);
impl_gpio_pin!(PB15, 1, 15, EXTI15);
pub const GPIOC: gpio::Gpio = gpio::Gpio(0x48000800 as _);
impl_gpio_pin!(PC0, 2, 0, EXTI0);
impl_gpio_pin!(PC1, 2, 1, EXTI1);
impl_gpio_pin!(PC2, 2, 2, EXTI2);
impl_gpio_pin!(PC3, 2, 3, EXTI3);
impl_gpio_pin!(PC4, 2, 4, EXTI4);
impl_gpio_pin!(PC5, 2, 5, EXTI5);
impl_gpio_pin!(PC6, 2, 6, EXTI6);
impl_gpio_pin!(PC7, 2, 7, EXTI7);
impl_gpio_pin!(PC8, 2, 8, EXTI8);
impl_gpio_pin!(PC9, 2, 9, EXTI9);
impl_gpio_pin!(PC10, 2, 10, EXTI10);
impl_gpio_pin!(PC11, 2, 11, EXTI11);
impl_gpio_pin!(PC12, 2, 12, EXTI12);
impl_gpio_pin!(PC13, 2, 13, EXTI13);
impl_gpio_pin!(PC14, 2, 14, EXTI14);
impl_gpio_pin!(PC15, 2, 15, EXTI15);
pub const GPIOD: gpio::Gpio = gpio::Gpio(0x48000c00 as _);
impl_gpio_pin!(PD0, 3, 0, EXTI0);
impl_gpio_pin!(PD1, 3, 1, EXTI1);
impl_gpio_pin!(PD2, 3, 2, EXTI2);
impl_gpio_pin!(PD3, 3, 3, EXTI3);
impl_gpio_pin!(PD4, 3, 4, EXTI4);
impl_gpio_pin!(PD5, 3, 5, EXTI5);
impl_gpio_pin!(PD6, 3, 6, EXTI6);
impl_gpio_pin!(PD7, 3, 7, EXTI7);
impl_gpio_pin!(PD8, 3, 8, EXTI8);
impl_gpio_pin!(PD9, 3, 9, EXTI9);
impl_gpio_pin!(PD10, 3, 10, EXTI10);
impl_gpio_pin!(PD11, 3, 11, EXTI11);
impl_gpio_pin!(PD12, 3, 12, EXTI12);
impl_gpio_pin!(PD13, 3, 13, EXTI13);
impl_gpio_pin!(PD14, 3, 14, EXTI14);
impl_gpio_pin!(PD15, 3, 15, EXTI15);
pub const GPIOE: gpio::Gpio = gpio::Gpio(0x48001000 as _);
impl_gpio_pin!(PE0, 4, 0, EXTI0);
impl_gpio_pin!(PE1, 4, 1, EXTI1);
impl_gpio_pin!(PE2, 4, 2, EXTI2);
impl_gpio_pin!(PE3, 4, 3, EXTI3);
impl_gpio_pin!(PE4, 4, 4, EXTI4);
impl_gpio_pin!(PE5, 4, 5, EXTI5);
impl_gpio_pin!(PE6, 4, 6, EXTI6);
impl_gpio_pin!(PE7, 4, 7, EXTI7);
impl_gpio_pin!(PE8, 4, 8, EXTI8);
impl_gpio_pin!(PE9, 4, 9, EXTI9);
impl_gpio_pin!(PE10, 4, 10, EXTI10);
impl_gpio_pin!(PE11, 4, 11, EXTI11);
impl_gpio_pin!(PE12, 4, 12, EXTI12);
impl_gpio_pin!(PE13, 4, 13, EXTI13);
impl_gpio_pin!(PE14, 4, 14, EXTI14);
impl_gpio_pin!(PE15, 4, 15, EXTI15);
pub const GPIOF: gpio::Gpio = gpio::Gpio(0x48001400 as _);
impl_gpio_pin!(PF0, 5, 0, EXTI0);
impl_gpio_pin!(PF1, 5, 1, EXTI1);
impl_gpio_pin!(PF2, 5, 2, EXTI2);
impl_gpio_pin!(PF3, 5, 3, EXTI3);
impl_gpio_pin!(PF4, 5, 4, EXTI4);
impl_gpio_pin!(PF5, 5, 5, EXTI5);
impl_gpio_pin!(PF6, 5, 6, EXTI6);
impl_gpio_pin!(PF7, 5, 7, EXTI7);
impl_gpio_pin!(PF8, 5, 8, EXTI8);
impl_gpio_pin!(PF9, 5, 9, EXTI9);
impl_gpio_pin!(PF10, 5, 10, EXTI10);
impl_gpio_pin!(PF11, 5, 11, EXTI11);
impl_gpio_pin!(PF12, 5, 12, EXTI12);
impl_gpio_pin!(PF13, 5, 13, EXTI13);
impl_gpio_pin!(PF14, 5, 14, EXTI14);
impl_gpio_pin!(PF15, 5, 15, EXTI15);
pub const GPIOG: gpio::Gpio = gpio::Gpio(0x48001800 as _);
impl_gpio_pin!(PG0, 6, 0, EXTI0);
impl_gpio_pin!(PG1, 6, 1, EXTI1);
impl_gpio_pin!(PG2, 6, 2, EXTI2);
impl_gpio_pin!(PG3, 6, 3, EXTI3);
impl_gpio_pin!(PG4, 6, 4, EXTI4);
impl_gpio_pin!(PG5, 6, 5, EXTI5);
impl_gpio_pin!(PG6, 6, 6, EXTI6);
impl_gpio_pin!(PG7, 6, 7, EXTI7);
impl_gpio_pin!(PG8, 6, 8, EXTI8);
impl_gpio_pin!(PG9, 6, 9, EXTI9);
impl_gpio_pin!(PG10, 6, 10, EXTI10);
impl_gpio_pin!(PG11, 6, 11, EXTI11);
impl_gpio_pin!(PG12, 6, 12, EXTI12);
impl_gpio_pin!(PG13, 6, 13, EXTI13);
impl_gpio_pin!(PG14, 6, 14, EXTI14);
impl_gpio_pin!(PG15, 6, 15, EXTI15);
pub const GPIOH: gpio::Gpio = gpio::Gpio(0x48001c00 as _);
impl_gpio_pin!(PH0, 7, 0, EXTI0);
impl_gpio_pin!(PH1, 7, 1, EXTI1);
impl_gpio_pin!(PH2, 7, 2, EXTI2);
impl_gpio_pin!(PH3, 7, 3, EXTI3);
impl_gpio_pin!(PH4, 7, 4, EXTI4);
impl_gpio_pin!(PH5, 7, 5, EXTI5);
impl_gpio_pin!(PH6, 7, 6, EXTI6);
impl_gpio_pin!(PH7, 7, 7, EXTI7);
impl_gpio_pin!(PH8, 7, 8, EXTI8);
impl_gpio_pin!(PH9, 7, 9, EXTI9);
impl_gpio_pin!(PH10, 7, 10, EXTI10);
impl_gpio_pin!(PH11, 7, 11, EXTI11);
impl_gpio_pin!(PH12, 7, 12, EXTI12);
impl_gpio_pin!(PH13, 7, 13, EXTI13);
impl_gpio_pin!(PH14, 7, 14, EXTI14);
impl_gpio_pin!(PH15, 7, 15, EXTI15);
pub const GPIOI: gpio::Gpio = gpio::Gpio(0x48002000 as _);
impl_gpio_pin!(PI0, 8, 0, EXTI0);
impl_gpio_pin!(PI1, 8, 1, EXTI1);
impl_gpio_pin!(PI2, 8, 2, EXTI2);
impl_gpio_pin!(PI3, 8, 3, EXTI3);
impl_gpio_pin!(PI4, 8, 4, EXTI4);
impl_gpio_pin!(PI5, 8, 5, EXTI5);
impl_gpio_pin!(PI6, 8, 6, EXTI6);
impl_gpio_pin!(PI7, 8, 7, EXTI7);
impl_gpio_pin!(PI8, 8, 8, EXTI8);
impl_gpio_pin!(PI9, 8, 9, EXTI9);
impl_gpio_pin!(PI10, 8, 10, EXTI10);
impl_gpio_pin!(PI11, 8, 11, EXTI11);
impl_gpio_pin!(PI12, 8, 12, EXTI12);
impl_gpio_pin!(PI13, 8, 13, EXTI13);
impl_gpio_pin!(PI14, 8, 14, EXTI14);
impl_gpio_pin!(PI15, 8, 15, EXTI15);
pub const RNG: rng::Rng = rng::Rng(0x50060800 as _);
impl_rng!(RNG, RNG);
pub const SPI1: spi::Spi = spi::Spi(0x40013000 as _);
impl_spi!(SPI1, APB2);
impl_spi_pin!(SPI1, SckPin, PA1, 5);
impl_spi_pin!(SPI1, MisoPin, PA11, 5);
impl_spi_pin!(SPI1, MosiPin, PA12, 5);
impl_spi_pin!(SPI1, SckPin, PA5, 5);
impl_spi_pin!(SPI1, MisoPin, PA6, 5);
impl_spi_pin!(SPI1, MosiPin, PA7, 5);
impl_spi_pin!(SPI1, SckPin, PB3, 5);
impl_spi_pin!(SPI1, MisoPin, PB4, 5);
impl_spi_pin!(SPI1, MosiPin, PB5, 5);
impl_spi_pin!(SPI1, SckPin, PE13, 5);
impl_spi_pin!(SPI1, MisoPin, PE14, 5);
impl_spi_pin!(SPI1, MosiPin, PE15, 5);
impl_spi_pin!(SPI1, SckPin, PG2, 5);
impl_spi_pin!(SPI1, MisoPin, PG3, 5);
impl_spi_pin!(SPI1, MosiPin, PG4, 5);
pub const SPI2: spi::Spi = spi::Spi(0x40003800 as _);
impl_spi!(SPI2, APB1);
impl_spi_pin!(SPI2, SckPin, PA9, 3);
impl_spi_pin!(SPI2, SckPin, PB10, 5);
impl_spi_pin!(SPI2, SckPin, PB13, 5);
impl_spi_pin!(SPI2, MisoPin, PB14, 5);
impl_spi_pin!(SPI2, MosiPin, PB15, 5);
impl_spi_pin!(SPI2, MosiPin, PC1, 3);
impl_spi_pin!(SPI2, MisoPin, PC2, 5);
impl_spi_pin!(SPI2, MosiPin, PC3, 5);
impl_spi_pin!(SPI2, SckPin, PD1, 5);
impl_spi_pin!(SPI2, SckPin, PD3, 3);
impl_spi_pin!(SPI2, MisoPin, PD3, 5);
impl_spi_pin!(SPI2, MosiPin, PD4, 5);
impl_spi_pin!(SPI2, SckPin, PI1, 5);
impl_spi_pin!(SPI2, MisoPin, PI2, 5);
impl_spi_pin!(SPI2, MosiPin, PI3, 5);
pub const SPI3: spi::Spi = spi::Spi(0x40003c00 as _);
impl_spi!(SPI3, APB1);
impl_spi_pin!(SPI3, SckPin, PB3, 6);
impl_spi_pin!(SPI3, MisoPin, PB4, 6);
impl_spi_pin!(SPI3, MosiPin, PB5, 6);
impl_spi_pin!(SPI3, SckPin, PC10, 6);
impl_spi_pin!(SPI3, MisoPin, PC11, 6);
impl_spi_pin!(SPI3, MosiPin, PC12, 6);
impl_spi_pin!(SPI3, MisoPin, PG10, 6);
impl_spi_pin!(SPI3, MosiPin, PG11, 6);
impl_spi_pin!(SPI3, SckPin, PG9, 6);
pub const SYSCFG: syscfg::Syscfg = syscfg::Syscfg(0x40010000 as _);
pub const USART1: usart::Usart = usart::Usart(0x40013800 as _);
impl_usart!(USART1);
impl_usart_pin!(USART1, RxPin, PA10, 7);
impl_usart_pin!(USART1, CtsPin, PA11, 7);
impl_usart_pin!(USART1, RtsPin, PA12, 7);
impl_usart_pin!(USART1, CkPin, PA8, 7);
impl_usart_pin!(USART1, TxPin, PA9, 7);
impl_usart_pin!(USART1, RtsPin, PB3, 7);
impl_usart_pin!(USART1, CtsPin, PB4, 7);
impl_usart_pin!(USART1, CkPin, PB5, 7);
impl_usart_pin!(USART1, TxPin, PB6, 7);
impl_usart_pin!(USART1, RxPin, PB7, 7);
impl_usart_pin!(USART1, RxPin, PG10, 7);
impl_usart_pin!(USART1, CtsPin, PG11, 7);
impl_usart_pin!(USART1, RtsPin, PG12, 7);
impl_usart_pin!(USART1, CkPin, PG13, 7);
impl_usart_pin!(USART1, TxPin, PG9, 7);
pub const USART2: usart::Usart = usart::Usart(0x40004400 as _);
impl_usart!(USART2);
impl_usart_pin!(USART2, CtsPin, PA0, 7);
impl_usart_pin!(USART2, RtsPin, PA1, 7);
impl_usart_pin!(USART2, RxPin, PA15, 3);
impl_usart_pin!(USART2, TxPin, PA2, 7);
impl_usart_pin!(USART2, RxPin, PA3, 7);
impl_usart_pin!(USART2, CkPin, PA4, 7);
impl_usart_pin!(USART2, CtsPin, PD3, 7);
impl_usart_pin!(USART2, RtsPin, PD4, 7);
impl_usart_pin!(USART2, TxPin, PD5, 7);
impl_usart_pin!(USART2, RxPin, PD6, 7);
impl_usart_pin!(USART2, CkPin, PD7, 7);
pub const USART3: usart::Usart = usart::Usart(0x40004800 as _);
impl_usart!(USART3);
impl_usart_pin!(USART3, RtsPin, PA15, 7);
impl_usart_pin!(USART3, CtsPin, PA6, 7);
impl_usart_pin!(USART3, CkPin, PB0, 7);
impl_usart_pin!(USART3, RtsPin, PB1, 7);
impl_usart_pin!(USART3, TxPin, PB10, 7);
impl_usart_pin!(USART3, RxPin, PB11, 7);
impl_usart_pin!(USART3, CkPin, PB12, 7);
impl_usart_pin!(USART3, CtsPin, PB13, 7);
impl_usart_pin!(USART3, RtsPin, PB14, 7);
impl_usart_pin!(USART3, TxPin, PC10, 7);
impl_usart_pin!(USART3, RxPin, PC11, 7);
impl_usart_pin!(USART3, CkPin, PC12, 7);
impl_usart_pin!(USART3, TxPin, PC4, 7);
impl_usart_pin!(USART3, RxPin, PC5, 7);
impl_usart_pin!(USART3, CkPin, PD10, 7);
impl_usart_pin!(USART3, CtsPin, PD11, 7);
impl_usart_pin!(USART3, RtsPin, PD12, 7);
impl_usart_pin!(USART3, RtsPin, PD2, 7);
impl_usart_pin!(USART3, TxPin, PD8, 7);
impl_usart_pin!(USART3, RxPin, PD9, 7);
pub use regs::dma_v1 as dma;
pub use regs::exti_v1 as exti;
pub use regs::gpio_v2 as gpio;
pub use regs::rng_v1 as rng;
pub use regs::spi_v2 as spi;
pub use regs::syscfg_l4 as syscfg;
pub use regs::usart_v2 as usart;
mod regs;
use embassy_extras::peripherals;
pub use regs::generic;
peripherals!(
    EXTI0, EXTI1, EXTI2, EXTI3, EXTI4, EXTI5, EXTI6, EXTI7, EXTI8, EXTI9, EXTI10, EXTI11, EXTI12,
    EXTI13, EXTI14, EXTI15, DMA1_CH0, DMA1_CH1, DMA1_CH2, DMA1_CH3, DMA1_CH4, DMA1_CH5, DMA1_CH6,
    DMA1_CH7, DMA2_CH0, DMA2_CH1, DMA2_CH2, DMA2_CH3, DMA2_CH4, DMA2_CH5, DMA2_CH6, DMA2_CH7, EXTI,
    PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10, PA11, PA12, PA13, PA14, PA15, PB0, PB1,
    PB2, PB3, PB4, PB5, PB6, PB7, PB8, PB9, PB10, PB11, PB12, PB13, PB14, PB15, PC0, PC1, PC2, PC3,
    PC4, PC5, PC6, PC7, PC8, PC9, PC10, PC11, PC12, PC13, PC14, PC15, PD0, PD1, PD2, PD3, PD4, PD5,
    PD6, PD7, PD8, PD9, PD10, PD11, PD12, PD13, PD14, PD15, PE0, PE1, PE2, PE3, PE4, PE5, PE6, PE7,
    PE8, PE9, PE10, PE11, PE12, PE13, PE14, PE15, PF0, PF1, PF2, PF3, PF4, PF5, PF6, PF7, PF8, PF9,
    PF10, PF11, PF12, PF13, PF14, PF15, PG0, PG1, PG2, PG3, PG4, PG5, PG6, PG7, PG8, PG9, PG10,
    PG11, PG12, PG13, PG14, PG15, PH0, PH1, PH2, PH3, PH4, PH5, PH6, PH7, PH8, PH9, PH10, PH11,
    PH12, PH13, PH14, PH15, PI0, PI1, PI2, PI3, PI4, PI5, PI6, PI7, PI8, PI9, PI10, PI11, PI12,
    PI13, PI14, PI15, RNG, SPI1, SPI2, SPI3, SYSCFG, USART1, USART2, USART3
);

pub mod interrupt {
    pub use cortex_m::interrupt::{CriticalSection, Mutex};
    pub use embassy::interrupt::{declare, take, Interrupt};
    pub use embassy_extras::interrupt::Priority4 as Priority;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[allow(non_camel_case_types)]
    pub enum InterruptEnum {
        ADC1_2 = 18,
        ADC3 = 47,
        CAN1_RX0 = 20,
        CAN1_RX1 = 21,
        CAN1_SCE = 22,
        CAN1_TX = 19,
        CAN2_RX0 = 87,
        CAN2_RX1 = 88,
        CAN2_SCE = 89,
        CAN2_TX = 86,
        COMP = 64,
        CRS = 82,
        DCMI = 85,
        DFSDM1_FLT0 = 61,
        DFSDM1_FLT1 = 62,
        DFSDM1_FLT2 = 63,
        DFSDM1_FLT3 = 42,
        DMA1_Channel1 = 11,
        DMA1_Channel2 = 12,
        DMA1_Channel3 = 13,
        DMA1_Channel4 = 14,
        DMA1_Channel5 = 15,
        DMA1_Channel6 = 16,
        DMA1_Channel7 = 17,
        DMA2D = 90,
        DMA2_Channel1 = 56,
        DMA2_Channel2 = 57,
        DMA2_Channel3 = 58,
        DMA2_Channel4 = 59,
        DMA2_Channel5 = 60,
        DMA2_Channel6 = 68,
        DMA2_Channel7 = 69,
        EXTI0 = 6,
        EXTI1 = 7,
        EXTI15_10 = 40,
        EXTI2 = 8,
        EXTI3 = 9,
        EXTI4 = 10,
        EXTI9_5 = 23,
        FLASH = 4,
        FMC = 48,
        FPU = 81,
        I2C1_ER = 32,
        I2C1_EV = 31,
        I2C2_ER = 34,
        I2C2_EV = 33,
        I2C3_ER = 73,
        I2C3_EV = 72,
        I2C4_ER = 84,
        I2C4_EV = 83,
        LCD = 78,
        LPTIM1 = 65,
        LPTIM2 = 66,
        LPUART1 = 70,
        OTG_FS = 67,
        PVD_PVM = 1,
        QUADSPI = 71,
        RCC = 5,
        RNG = 80,
        RTC_Alarm = 41,
        RTC_WKUP = 3,
        SAI1 = 74,
        SAI2 = 75,
        SDMMC1 = 49,
        SPI1 = 35,
        SPI2 = 36,
        SPI3 = 51,
        SWPMI1 = 76,
        TAMP_STAMP = 2,
        TIM1_BRK_TIM15 = 24,
        TIM1_CC = 27,
        TIM1_TRG_COM_TIM17 = 26,
        TIM1_UP_TIM16 = 25,
        TIM2 = 28,
        TIM3 = 29,
        TIM4 = 30,
        TIM5 = 50,
        TIM6_DAC = 54,
        TIM7 = 55,
        TIM8_BRK = 43,
        TIM8_CC = 46,
        TIM8_TRG_COM = 45,
        TIM8_UP = 44,
        TSC = 77,
        UART4 = 52,
        UART5 = 53,
        USART1 = 37,
        USART2 = 38,
        USART3 = 39,
        WWDG = 0,
    }
    unsafe impl cortex_m::interrupt::InterruptNumber for InterruptEnum {
        #[inline(always)]
        fn number(self) -> u16 {
            self as u16
        }
    }

    declare!(ADC1_2);
    declare!(ADC3);
    declare!(CAN1_RX0);
    declare!(CAN1_RX1);
    declare!(CAN1_SCE);
    declare!(CAN1_TX);
    declare!(CAN2_RX0);
    declare!(CAN2_RX1);
    declare!(CAN2_SCE);
    declare!(CAN2_TX);
    declare!(COMP);
    declare!(CRS);
    declare!(DCMI);
    declare!(DFSDM1_FLT0);
    declare!(DFSDM1_FLT1);
    declare!(DFSDM1_FLT2);
    declare!(DFSDM1_FLT3);
    declare!(DMA1_Channel1);
    declare!(DMA1_Channel2);
    declare!(DMA1_Channel3);
    declare!(DMA1_Channel4);
    declare!(DMA1_Channel5);
    declare!(DMA1_Channel6);
    declare!(DMA1_Channel7);
    declare!(DMA2D);
    declare!(DMA2_Channel1);
    declare!(DMA2_Channel2);
    declare!(DMA2_Channel3);
    declare!(DMA2_Channel4);
    declare!(DMA2_Channel5);
    declare!(DMA2_Channel6);
    declare!(DMA2_Channel7);
    declare!(EXTI0);
    declare!(EXTI1);
    declare!(EXTI15_10);
    declare!(EXTI2);
    declare!(EXTI3);
    declare!(EXTI4);
    declare!(EXTI9_5);
    declare!(FLASH);
    declare!(FMC);
    declare!(FPU);
    declare!(I2C1_ER);
    declare!(I2C1_EV);
    declare!(I2C2_ER);
    declare!(I2C2_EV);
    declare!(I2C3_ER);
    declare!(I2C3_EV);
    declare!(I2C4_ER);
    declare!(I2C4_EV);
    declare!(LCD);
    declare!(LPTIM1);
    declare!(LPTIM2);
    declare!(LPUART1);
    declare!(OTG_FS);
    declare!(PVD_PVM);
    declare!(QUADSPI);
    declare!(RCC);
    declare!(RNG);
    declare!(RTC_Alarm);
    declare!(RTC_WKUP);
    declare!(SAI1);
    declare!(SAI2);
    declare!(SDMMC1);
    declare!(SPI1);
    declare!(SPI2);
    declare!(SPI3);
    declare!(SWPMI1);
    declare!(TAMP_STAMP);
    declare!(TIM1_BRK_TIM15);
    declare!(TIM1_CC);
    declare!(TIM1_TRG_COM_TIM17);
    declare!(TIM1_UP_TIM16);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(TIM5);
    declare!(TIM6_DAC);
    declare!(TIM7);
    declare!(TIM8_BRK);
    declare!(TIM8_CC);
    declare!(TIM8_TRG_COM);
    declare!(TIM8_UP);
    declare!(TSC);
    declare!(UART4);
    declare!(UART5);
    declare!(USART1);
    declare!(USART2);
    declare!(USART3);
    declare!(WWDG);
}
mod interrupt_vector {
    extern "C" {
        fn ADC1_2();
        fn ADC3();
        fn CAN1_RX0();
        fn CAN1_RX1();
        fn CAN1_SCE();
        fn CAN1_TX();
        fn CAN2_RX0();
        fn CAN2_RX1();
        fn CAN2_SCE();
        fn CAN2_TX();
        fn COMP();
        fn CRS();
        fn DCMI();
        fn DFSDM1_FLT0();
        fn DFSDM1_FLT1();
        fn DFSDM1_FLT2();
        fn DFSDM1_FLT3();
        fn DMA1_Channel1();
        fn DMA1_Channel2();
        fn DMA1_Channel3();
        fn DMA1_Channel4();
        fn DMA1_Channel5();
        fn DMA1_Channel6();
        fn DMA1_Channel7();
        fn DMA2D();
        fn DMA2_Channel1();
        fn DMA2_Channel2();
        fn DMA2_Channel3();
        fn DMA2_Channel4();
        fn DMA2_Channel5();
        fn DMA2_Channel6();
        fn DMA2_Channel7();
        fn EXTI0();
        fn EXTI1();
        fn EXTI15_10();
        fn EXTI2();
        fn EXTI3();
        fn EXTI4();
        fn EXTI9_5();
        fn FLASH();
        fn FMC();
        fn FPU();
        fn I2C1_ER();
        fn I2C1_EV();
        fn I2C2_ER();
        fn I2C2_EV();
        fn I2C3_ER();
        fn I2C3_EV();
        fn I2C4_ER();
        fn I2C4_EV();
        fn LCD();
        fn LPTIM1();
        fn LPTIM2();
        fn LPUART1();
        fn OTG_FS();
        fn PVD_PVM();
        fn QUADSPI();
        fn RCC();
        fn RNG();
        fn RTC_Alarm();
        fn RTC_WKUP();
        fn SAI1();
        fn SAI2();
        fn SDMMC1();
        fn SPI1();
        fn SPI2();
        fn SPI3();
        fn SWPMI1();
        fn TAMP_STAMP();
        fn TIM1_BRK_TIM15();
        fn TIM1_CC();
        fn TIM1_TRG_COM_TIM17();
        fn TIM1_UP_TIM16();
        fn TIM2();
        fn TIM3();
        fn TIM4();
        fn TIM5();
        fn TIM6_DAC();
        fn TIM7();
        fn TIM8_BRK();
        fn TIM8_CC();
        fn TIM8_TRG_COM();
        fn TIM8_UP();
        fn TSC();
        fn UART4();
        fn UART5();
        fn USART1();
        fn USART2();
        fn USART3();
        fn WWDG();
    }
    pub union Vector {
        _handler: unsafe extern "C" fn(),
        _reserved: u32,
    }
    #[link_section = ".vector_table.interrupts"]
    #[no_mangle]
    pub static __INTERRUPTS: [Vector; 91] = [
        Vector { _handler: WWDG },
        Vector { _handler: PVD_PVM },
        Vector {
            _handler: TAMP_STAMP,
        },
        Vector { _handler: RTC_WKUP },
        Vector { _handler: FLASH },
        Vector { _handler: RCC },
        Vector { _handler: EXTI0 },
        Vector { _handler: EXTI1 },
        Vector { _handler: EXTI2 },
        Vector { _handler: EXTI3 },
        Vector { _handler: EXTI4 },
        Vector {
            _handler: DMA1_Channel1,
        },
        Vector {
            _handler: DMA1_Channel2,
        },
        Vector {
            _handler: DMA1_Channel3,
        },
        Vector {
            _handler: DMA1_Channel4,
        },
        Vector {
            _handler: DMA1_Channel5,
        },
        Vector {
            _handler: DMA1_Channel6,
        },
        Vector {
            _handler: DMA1_Channel7,
        },
        Vector { _handler: ADC1_2 },
        Vector { _handler: CAN1_TX },
        Vector { _handler: CAN1_RX0 },
        Vector { _handler: CAN1_RX1 },
        Vector { _handler: CAN1_SCE },
        Vector { _handler: EXTI9_5 },
        Vector {
            _handler: TIM1_BRK_TIM15,
        },
        Vector {
            _handler: TIM1_UP_TIM16,
        },
        Vector {
            _handler: TIM1_TRG_COM_TIM17,
        },
        Vector { _handler: TIM1_CC },
        Vector { _handler: TIM2 },
        Vector { _handler: TIM3 },
        Vector { _handler: TIM4 },
        Vector { _handler: I2C1_EV },
        Vector { _handler: I2C1_ER },
        Vector { _handler: I2C2_EV },
        Vector { _handler: I2C2_ER },
        Vector { _handler: SPI1 },
        Vector { _handler: SPI2 },
        Vector { _handler: USART1 },
        Vector { _handler: USART2 },
        Vector { _handler: USART3 },
        Vector {
            _handler: EXTI15_10,
        },
        Vector {
            _handler: RTC_Alarm,
        },
        Vector {
            _handler: DFSDM1_FLT3,
        },
        Vector { _handler: TIM8_BRK },
        Vector { _handler: TIM8_UP },
        Vector {
            _handler: TIM8_TRG_COM,
        },
        Vector { _handler: TIM8_CC },
        Vector { _handler: ADC3 },
        Vector { _handler: FMC },
        Vector { _handler: SDMMC1 },
        Vector { _handler: TIM5 },
        Vector { _handler: SPI3 },
        Vector { _handler: UART4 },
        Vector { _handler: UART5 },
        Vector { _handler: TIM6_DAC },
        Vector { _handler: TIM7 },
        Vector {
            _handler: DMA2_Channel1,
        },
        Vector {
            _handler: DMA2_Channel2,
        },
        Vector {
            _handler: DMA2_Channel3,
        },
        Vector {
            _handler: DMA2_Channel4,
        },
        Vector {
            _handler: DMA2_Channel5,
        },
        Vector {
            _handler: DFSDM1_FLT0,
        },
        Vector {
            _handler: DFSDM1_FLT1,
        },
        Vector {
            _handler: DFSDM1_FLT2,
        },
        Vector { _handler: COMP },
        Vector { _handler: LPTIM1 },
        Vector { _handler: LPTIM2 },
        Vector { _handler: OTG_FS },
        Vector {
            _handler: DMA2_Channel6,
        },
        Vector {
            _handler: DMA2_Channel7,
        },
        Vector { _handler: LPUART1 },
        Vector { _handler: QUADSPI },
        Vector { _handler: I2C3_EV },
        Vector { _handler: I2C3_ER },
        Vector { _handler: SAI1 },
        Vector { _handler: SAI2 },
        Vector { _handler: SWPMI1 },
        Vector { _handler: TSC },
        Vector { _handler: LCD },
        Vector { _reserved: 0 },
        Vector { _handler: RNG },
        Vector { _handler: FPU },
        Vector { _handler: CRS },
        Vector { _handler: I2C4_EV },
        Vector { _handler: I2C4_ER },
        Vector { _handler: DCMI },
        Vector { _handler: CAN2_TX },
        Vector { _handler: CAN2_RX0 },
        Vector { _handler: CAN2_RX1 },
        Vector { _handler: CAN2_SCE },
        Vector { _handler: DMA2D },
    ];
}
