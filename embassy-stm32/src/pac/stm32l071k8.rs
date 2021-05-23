#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_snake_case)]

pub fn GPIO(n: usize) -> gpio::Gpio {
    gpio::Gpio((0x50000000 + 0x400 * n) as _)
}
pub const DMA1: dma::Dma = dma::Dma(0x40020000 as _);
impl_dma_channel!(DMA1_CH0, 0, 0);
impl_dma_channel!(DMA1_CH1, 0, 1);
impl_dma_channel!(DMA1_CH2, 0, 2);
impl_dma_channel!(DMA1_CH3, 0, 3);
impl_dma_channel!(DMA1_CH4, 0, 4);
impl_dma_channel!(DMA1_CH5, 0, 5);
impl_dma_channel!(DMA1_CH6, 0, 6);
impl_dma_channel!(DMA1_CH7, 0, 7);
pub const EXTI: exti::Exti = exti::Exti(0x40010400 as _);
pub const GPIOA: gpio::Gpio = gpio::Gpio(0x50000000 as _);
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
pub const GPIOB: gpio::Gpio = gpio::Gpio(0x50000400 as _);
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
pub const GPIOC: gpio::Gpio = gpio::Gpio(0x50000800 as _);
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
pub const GPIOD: gpio::Gpio = gpio::Gpio(0x50000c00 as _);
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
pub const GPIOE: gpio::Gpio = gpio::Gpio(0x50001000 as _);
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
pub const GPIOH: gpio::Gpio = gpio::Gpio(0x50001c00 as _);
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
pub const RCC: rcc::Rcc = rcc::Rcc(0x40021000 as _);
pub const SYSCFG: syscfg::Syscfg = syscfg::Syscfg(0x40010000 as _);
pub const TIM2: timer::TimGp16 = timer::TimGp16(0x40000000 as _);
impl_timer!(TIM2);
pub const TIM21: timer::TimGp16 = timer::TimGp16(0x40010800 as _);
pub const TIM22: timer::TimGp16 = timer::TimGp16(0x40011400 as _);
pub const TIM3: timer::TimGp16 = timer::TimGp16(0x40000400 as _);
impl_timer!(TIM3);
pub const TIM6: timer::TimGp16 = timer::TimGp16(0x40001000 as _);
pub const TIM7: timer::TimGp16 = timer::TimGp16(0x40001400 as _);
pub const USART1: usart::Usart = usart::Usart(0x40013800 as _);
impl_usart!(USART1);
impl_usart_pin!(USART1, RxPin, PA10, 4);
impl_usart_pin!(USART1, CtsPin, PA11, 4);
impl_usart_pin!(USART1, RtsPin, PA12, 4);
impl_usart_pin!(USART1, CkPin, PA8, 4);
impl_usart_pin!(USART1, TxPin, PA9, 4);
impl_usart_pin!(USART1, RtsPin, PB3, 5);
impl_usart_pin!(USART1, CtsPin, PB4, 5);
impl_usart_pin!(USART1, CkPin, PB5, 5);
pub const USART2: usart::Usart = usart::Usart(0x40004400 as _);
impl_usart!(USART2);
impl_usart_pin!(USART2, CtsPin, PA0, 4);
impl_usart_pin!(USART2, RtsPin, PA1, 4);
impl_usart_pin!(USART2, TxPin, PA14, 4);
impl_usart_pin!(USART2, RxPin, PA15, 4);
impl_usart_pin!(USART2, TxPin, PA2, 4);
impl_usart_pin!(USART2, RxPin, PA3, 4);
impl_usart_pin!(USART2, CkPin, PA4, 4);
pub const USART4: usart::Usart = usart::Usart(0x40004c00 as _);
impl_usart!(USART4);
impl_usart_pin!(USART4, TxPin, PA0, 6);
impl_usart_pin!(USART4, RxPin, PA1, 6);
impl_usart_pin!(USART4, RtsPin, PA15, 6);
impl_usart_pin!(USART4, CtsPin, PB7, 6);
impl_usart_pin!(USART4, TxPin, PC10, 6);
impl_usart_pin!(USART4, RxPin, PC11, 6);
impl_usart_pin!(USART4, CkPin, PC12, 6);
impl_usart_pin!(USART4, TxPin, PE8, 6);
impl_usart_pin!(USART4, RxPin, PE9, 6);
pub use super::regs::dma_v1 as dma;
pub use super::regs::exti_v1 as exti;
pub use super::regs::gpio_v2 as gpio;
pub use super::regs::rcc_l0 as rcc;
pub use super::regs::syscfg_l0 as syscfg;
pub use super::regs::timer_v1 as timer;
pub use super::regs::usart_v2 as usart;
embassy_extras::peripherals!(
    EXTI0, EXTI1, EXTI2, EXTI3, EXTI4, EXTI5, EXTI6, EXTI7, EXTI8, EXTI9, EXTI10, EXTI11, EXTI12,
    EXTI13, EXTI14, EXTI15, DMA1_CH0, DMA1_CH1, DMA1_CH2, DMA1_CH3, DMA1_CH4, DMA1_CH5, DMA1_CH6,
    DMA1_CH7, EXTI, PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10, PA11, PA12, PA13, PA14,
    PA15, PB0, PB1, PB2, PB3, PB4, PB5, PB6, PB7, PB8, PB9, PB10, PB11, PB12, PB13, PB14, PB15,
    PC0, PC1, PC2, PC3, PC4, PC5, PC6, PC7, PC8, PC9, PC10, PC11, PC12, PC13, PC14, PC15, PD0, PD1,
    PD2, PD3, PD4, PD5, PD6, PD7, PD8, PD9, PD10, PD11, PD12, PD13, PD14, PD15, PE0, PE1, PE2, PE3,
    PE4, PE5, PE6, PE7, PE8, PE9, PE10, PE11, PE12, PE13, PE14, PE15, PH0, PH1, PH2, PH3, PH4, PH5,
    PH6, PH7, PH8, PH9, PH10, PH11, PH12, PH13, PH14, PH15, RCC, SYSCFG, TIM2, TIM21, TIM22, TIM3,
    TIM6, TIM7, USART1, USART2, USART4
);
pub fn DMA(n: u8) -> dma::Dma {
    match n {
        _ => DMA1,
    }
}
impl_exti_irq!(EXTI0_1, EXTI2_3, EXTI4_15);
pub mod interrupt {
    pub use bare_metal::Mutex;
    pub use critical_section::CriticalSection;
    pub use embassy::interrupt::{declare, take, Interrupt};
    pub use embassy_extras::interrupt::Priority4 as Priority;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[allow(non_camel_case_types)]
    pub enum InterruptEnum {
        ADC1_COMP = 12,
        DMA1_Channel1 = 9,
        DMA1_Channel2_3 = 10,
        DMA1_Channel4_5_6_7 = 11,
        EXTI0_1 = 5,
        EXTI2_3 = 6,
        EXTI4_15 = 7,
        FLASH = 3,
        I2C1 = 23,
        I2C2 = 24,
        I2C3 = 21,
        LPTIM1 = 13,
        LPUART1 = 29,
        PVD = 1,
        RCC = 4,
        RTC = 2,
        SPI1 = 25,
        SPI2 = 26,
        TIM2 = 15,
        TIM21 = 20,
        TIM22 = 22,
        TIM3 = 16,
        TIM6 = 17,
        TIM7 = 18,
        USART1 = 27,
        USART2 = 28,
        USART4_5 = 14,
        WWDG = 0,
    }
    unsafe impl cortex_m::interrupt::InterruptNumber for InterruptEnum {
        #[inline(always)]
        fn number(self) -> u16 {
            self as u16
        }
    }

    declare!(ADC1_COMP);
    declare!(DMA1_Channel1);
    declare!(DMA1_Channel2_3);
    declare!(DMA1_Channel4_5_6_7);
    declare!(EXTI0_1);
    declare!(EXTI2_3);
    declare!(EXTI4_15);
    declare!(FLASH);
    declare!(I2C1);
    declare!(I2C2);
    declare!(I2C3);
    declare!(LPTIM1);
    declare!(LPUART1);
    declare!(PVD);
    declare!(RCC);
    declare!(RTC);
    declare!(SPI1);
    declare!(SPI2);
    declare!(TIM2);
    declare!(TIM21);
    declare!(TIM22);
    declare!(TIM3);
    declare!(TIM6);
    declare!(TIM7);
    declare!(USART1);
    declare!(USART2);
    declare!(USART4_5);
    declare!(WWDG);
}
mod interrupt_vector {
    extern "C" {
        fn ADC1_COMP();
        fn DMA1_Channel1();
        fn DMA1_Channel2_3();
        fn DMA1_Channel4_5_6_7();
        fn EXTI0_1();
        fn EXTI2_3();
        fn EXTI4_15();
        fn FLASH();
        fn I2C1();
        fn I2C2();
        fn I2C3();
        fn LPTIM1();
        fn LPUART1();
        fn PVD();
        fn RCC();
        fn RTC();
        fn SPI1();
        fn SPI2();
        fn TIM2();
        fn TIM21();
        fn TIM22();
        fn TIM3();
        fn TIM6();
        fn TIM7();
        fn USART1();
        fn USART2();
        fn USART4_5();
        fn WWDG();
    }
    pub union Vector {
        _handler: unsafe extern "C" fn(),
        _reserved: u32,
    }
    #[link_section = ".vector_table.interrupts"]
    #[no_mangle]
    pub static __INTERRUPTS: [Vector; 30] = [
        Vector { _handler: WWDG },
        Vector { _handler: PVD },
        Vector { _handler: RTC },
        Vector { _handler: FLASH },
        Vector { _handler: RCC },
        Vector { _handler: EXTI0_1 },
        Vector { _handler: EXTI2_3 },
        Vector { _handler: EXTI4_15 },
        Vector { _reserved: 0 },
        Vector {
            _handler: DMA1_Channel1,
        },
        Vector {
            _handler: DMA1_Channel2_3,
        },
        Vector {
            _handler: DMA1_Channel4_5_6_7,
        },
        Vector {
            _handler: ADC1_COMP,
        },
        Vector { _handler: LPTIM1 },
        Vector { _handler: USART4_5 },
        Vector { _handler: TIM2 },
        Vector { _handler: TIM3 },
        Vector { _handler: TIM6 },
        Vector { _handler: TIM7 },
        Vector { _reserved: 0 },
        Vector { _handler: TIM21 },
        Vector { _handler: I2C3 },
        Vector { _handler: TIM22 },
        Vector { _handler: I2C1 },
        Vector { _handler: I2C2 },
        Vector { _handler: SPI1 },
        Vector { _handler: SPI2 },
        Vector { _handler: USART1 },
        Vector { _handler: USART2 },
        Vector { _handler: LPUART1 },
    ];
}
