use embassy_extras::peripherals;
peripherals!(
    EXTI0, EXTI1, EXTI2, EXTI3, EXTI4, EXTI5, EXTI6, EXTI7, EXTI8, EXTI9, EXTI10, EXTI11, EXTI12,
    EXTI13, EXTI14, EXTI15, ADC1, ADC2, ADC3, CAN1, CAN2, CRYP, DAC, DCMI, DMA2D, ETH, EXTI, PA0,
    PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10, PA11, PA12, PA13, PA14, PA15, PB0, PB1, PB2,
    PB3, PB4, PB5, PB6, PB7, PB8, PB9, PB10, PB11, PB12, PB13, PB14, PB15, PC0, PC1, PC2, PC3, PC4,
    PC5, PC6, PC7, PC8, PC9, PC10, PC11, PC12, PC13, PC14, PC15, PD0, PD1, PD2, PD3, PD4, PD5, PD6,
    PD7, PD8, PD9, PD10, PD11, PD12, PD13, PD14, PD15, PE0, PE1, PE2, PE3, PE4, PE5, PE6, PE7, PE8,
    PE9, PE10, PE11, PE12, PE13, PE14, PE15, PF0, PF1, PF2, PF3, PF4, PF5, PF6, PF7, PF8, PF9,
    PF10, PF11, PF12, PF13, PF14, PF15, PG0, PG1, PG2, PG3, PG4, PG5, PG6, PG7, PG8, PG9, PG10,
    PG11, PG12, PG13, PG14, PG15, PH0, PH1, PH2, PH3, PH4, PH5, PH6, PH7, PH8, PH9, PH10, PH11,
    PH12, PH13, PH14, PH15, PI0, PI1, PI2, PI3, PI4, PI5, PI6, PI7, PI8, PI9, PI10, PI11, PI12,
    PI13, PI14, PI15, PJ0, PJ1, PJ2, PJ3, PJ4, PJ5, PJ6, PJ7, PJ8, PJ9, PJ10, PJ11, PJ12, PJ13,
    PJ14, PJ15, PK0, PK1, PK2, PK3, PK4, PK5, PK6, PK7, PK8, PK9, PK10, PK11, PK12, PK13, PK14,
    PK15, HASH, I2C1, I2C2, I2C3, IWDG, LTDC, RCC, RNG, RTC, SAI1, SDIO, SPI1, SPI2, SPI3, SPI4,
    SPI5, SPI6, SYSCFG, TIM1, TIM10, TIM11, TIM12, TIM13, TIM14, TIM2, TIM3, TIM4, TIM5, TIM6,
    TIM7, TIM8, TIM9, UART4, UART5, UART7, UART8, USART1, USART2, USART3, USART6, USB_OTG_FS,
    USB_OTG_HS, WWDG
);
pub const SYSCFG_BASE: usize = 0x40013800;
pub const EXTI_BASE: usize = 0x40013c00;
pub const GPIO_BASE: usize = 0x40020000;
pub const GPIO_STRIDE: usize = 0x400;

pub mod interrupt {
    pub use cortex_m::interrupt::{CriticalSection, Mutex};
    pub use embassy::interrupt::{declare, take, Interrupt};
    pub use embassy_extras::interrupt::Priority4 as Priority;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[allow(non_camel_case_types)]
    enum InterruptEnum {
        ADC = 18,
        CAN1_RX0 = 20,
        CAN1_RX1 = 21,
        CAN1_SCE = 22,
        CAN1_TX = 19,
        CAN2_RX0 = 64,
        CAN2_RX1 = 65,
        CAN2_SCE = 66,
        CAN2_TX = 63,
        CRYP = 79,
        DCMI = 78,
        DMA1_Stream0 = 11,
        DMA1_Stream1 = 12,
        DMA1_Stream2 = 13,
        DMA1_Stream3 = 14,
        DMA1_Stream4 = 15,
        DMA1_Stream5 = 16,
        DMA1_Stream6 = 17,
        DMA1_Stream7 = 47,
        DMA2D = 90,
        DMA2_Stream0 = 56,
        DMA2_Stream1 = 57,
        DMA2_Stream2 = 58,
        DMA2_Stream3 = 59,
        DMA2_Stream4 = 60,
        DMA2_Stream5 = 68,
        DMA2_Stream6 = 69,
        DMA2_Stream7 = 70,
        ETH = 61,
        ETH_WKUP = 62,
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
        HASH_RNG = 80,
        I2C1_ER = 32,
        I2C1_EV = 31,
        I2C2_ER = 34,
        I2C2_EV = 33,
        I2C3_ER = 73,
        I2C3_EV = 72,
        LTDC = 88,
        LTDC_ER = 89,
        OTG_FS = 67,
        OTG_FS_WKUP = 42,
        OTG_HS = 77,
        OTG_HS_EP1_IN = 75,
        OTG_HS_EP1_OUT = 74,
        OTG_HS_WKUP = 76,
        PVD = 1,
        RCC = 5,
        RTC_Alarm = 41,
        RTC_WKUP = 3,
        SAI1 = 87,
        SDIO = 49,
        SPI1 = 35,
        SPI2 = 36,
        SPI3 = 51,
        SPI4 = 84,
        SPI5 = 85,
        SPI6 = 86,
        TAMP_STAMP = 2,
        TIM1_BRK_TIM9 = 24,
        TIM1_CC = 27,
        TIM1_TRG_COM_TIM11 = 26,
        TIM1_UP_TIM10 = 25,
        TIM2 = 28,
        TIM3 = 29,
        TIM4 = 30,
        TIM5 = 50,
        TIM6_DAC = 54,
        TIM7 = 55,
        TIM8_BRK_TIM12 = 43,
        TIM8_CC = 46,
        TIM8_TRG_COM_TIM14 = 45,
        TIM8_UP_TIM13 = 44,
        UART4 = 52,
        UART5 = 53,
        UART7 = 82,
        UART8 = 83,
        USART1 = 37,
        USART2 = 38,
        USART3 = 39,
        USART6 = 71,
        WWDG = 0,
    }
    unsafe impl cortex_m::interrupt::InterruptNumber for InterruptEnum {
        #[inline(always)]
        fn number(self) -> u16 {
            self as u16
        }
    }

    declare!(ADC);
    declare!(CAN1_RX0);
    declare!(CAN1_RX1);
    declare!(CAN1_SCE);
    declare!(CAN1_TX);
    declare!(CAN2_RX0);
    declare!(CAN2_RX1);
    declare!(CAN2_SCE);
    declare!(CAN2_TX);
    declare!(CRYP);
    declare!(DCMI);
    declare!(DMA1_Stream0);
    declare!(DMA1_Stream1);
    declare!(DMA1_Stream2);
    declare!(DMA1_Stream3);
    declare!(DMA1_Stream4);
    declare!(DMA1_Stream5);
    declare!(DMA1_Stream6);
    declare!(DMA1_Stream7);
    declare!(DMA2D);
    declare!(DMA2_Stream0);
    declare!(DMA2_Stream1);
    declare!(DMA2_Stream2);
    declare!(DMA2_Stream3);
    declare!(DMA2_Stream4);
    declare!(DMA2_Stream5);
    declare!(DMA2_Stream6);
    declare!(DMA2_Stream7);
    declare!(ETH);
    declare!(ETH_WKUP);
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
    declare!(HASH_RNG);
    declare!(I2C1_ER);
    declare!(I2C1_EV);
    declare!(I2C2_ER);
    declare!(I2C2_EV);
    declare!(I2C3_ER);
    declare!(I2C3_EV);
    declare!(LTDC);
    declare!(LTDC_ER);
    declare!(OTG_FS);
    declare!(OTG_FS_WKUP);
    declare!(OTG_HS);
    declare!(OTG_HS_EP1_IN);
    declare!(OTG_HS_EP1_OUT);
    declare!(OTG_HS_WKUP);
    declare!(PVD);
    declare!(RCC);
    declare!(RTC_Alarm);
    declare!(RTC_WKUP);
    declare!(SAI1);
    declare!(SDIO);
    declare!(SPI1);
    declare!(SPI2);
    declare!(SPI3);
    declare!(SPI4);
    declare!(SPI5);
    declare!(SPI6);
    declare!(TAMP_STAMP);
    declare!(TIM1_BRK_TIM9);
    declare!(TIM1_CC);
    declare!(TIM1_TRG_COM_TIM11);
    declare!(TIM1_UP_TIM10);
    declare!(TIM2);
    declare!(TIM3);
    declare!(TIM4);
    declare!(TIM5);
    declare!(TIM6_DAC);
    declare!(TIM7);
    declare!(TIM8_BRK_TIM12);
    declare!(TIM8_CC);
    declare!(TIM8_TRG_COM_TIM14);
    declare!(TIM8_UP_TIM13);
    declare!(UART4);
    declare!(UART5);
    declare!(UART7);
    declare!(UART8);
    declare!(USART1);
    declare!(USART2);
    declare!(USART3);
    declare!(USART6);
    declare!(WWDG);
}
mod interrupt_vector {
    extern "C" {
        fn ADC();
        fn CAN1_RX0();
        fn CAN1_RX1();
        fn CAN1_SCE();
        fn CAN1_TX();
        fn CAN2_RX0();
        fn CAN2_RX1();
        fn CAN2_SCE();
        fn CAN2_TX();
        fn CRYP();
        fn DCMI();
        fn DMA1_Stream0();
        fn DMA1_Stream1();
        fn DMA1_Stream2();
        fn DMA1_Stream3();
        fn DMA1_Stream4();
        fn DMA1_Stream5();
        fn DMA1_Stream6();
        fn DMA1_Stream7();
        fn DMA2D();
        fn DMA2_Stream0();
        fn DMA2_Stream1();
        fn DMA2_Stream2();
        fn DMA2_Stream3();
        fn DMA2_Stream4();
        fn DMA2_Stream5();
        fn DMA2_Stream6();
        fn DMA2_Stream7();
        fn ETH();
        fn ETH_WKUP();
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
        fn HASH_RNG();
        fn I2C1_ER();
        fn I2C1_EV();
        fn I2C2_ER();
        fn I2C2_EV();
        fn I2C3_ER();
        fn I2C3_EV();
        fn LTDC();
        fn LTDC_ER();
        fn OTG_FS();
        fn OTG_FS_WKUP();
        fn OTG_HS();
        fn OTG_HS_EP1_IN();
        fn OTG_HS_EP1_OUT();
        fn OTG_HS_WKUP();
        fn PVD();
        fn RCC();
        fn RTC_Alarm();
        fn RTC_WKUP();
        fn SAI1();
        fn SDIO();
        fn SPI1();
        fn SPI2();
        fn SPI3();
        fn SPI4();
        fn SPI5();
        fn SPI6();
        fn TAMP_STAMP();
        fn TIM1_BRK_TIM9();
        fn TIM1_CC();
        fn TIM1_TRG_COM_TIM11();
        fn TIM1_UP_TIM10();
        fn TIM2();
        fn TIM3();
        fn TIM4();
        fn TIM5();
        fn TIM6_DAC();
        fn TIM7();
        fn TIM8_BRK_TIM12();
        fn TIM8_CC();
        fn TIM8_TRG_COM_TIM14();
        fn TIM8_UP_TIM13();
        fn UART4();
        fn UART5();
        fn UART7();
        fn UART8();
        fn USART1();
        fn USART2();
        fn USART3();
        fn USART6();
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
        Vector { _handler: PVD },
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
            _handler: DMA1_Stream0,
        },
        Vector {
            _handler: DMA1_Stream1,
        },
        Vector {
            _handler: DMA1_Stream2,
        },
        Vector {
            _handler: DMA1_Stream3,
        },
        Vector {
            _handler: DMA1_Stream4,
        },
        Vector {
            _handler: DMA1_Stream5,
        },
        Vector {
            _handler: DMA1_Stream6,
        },
        Vector { _handler: ADC },
        Vector { _handler: CAN1_TX },
        Vector { _handler: CAN1_RX0 },
        Vector { _handler: CAN1_RX1 },
        Vector { _handler: CAN1_SCE },
        Vector { _handler: EXTI9_5 },
        Vector {
            _handler: TIM1_BRK_TIM9,
        },
        Vector {
            _handler: TIM1_UP_TIM10,
        },
        Vector {
            _handler: TIM1_TRG_COM_TIM11,
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
            _handler: OTG_FS_WKUP,
        },
        Vector {
            _handler: TIM8_BRK_TIM12,
        },
        Vector {
            _handler: TIM8_UP_TIM13,
        },
        Vector {
            _handler: TIM8_TRG_COM_TIM14,
        },
        Vector { _handler: TIM8_CC },
        Vector {
            _handler: DMA1_Stream7,
        },
        Vector { _handler: FMC },
        Vector { _handler: SDIO },
        Vector { _handler: TIM5 },
        Vector { _handler: SPI3 },
        Vector { _handler: UART4 },
        Vector { _handler: UART5 },
        Vector { _handler: TIM6_DAC },
        Vector { _handler: TIM7 },
        Vector {
            _handler: DMA2_Stream0,
        },
        Vector {
            _handler: DMA2_Stream1,
        },
        Vector {
            _handler: DMA2_Stream2,
        },
        Vector {
            _handler: DMA2_Stream3,
        },
        Vector {
            _handler: DMA2_Stream4,
        },
        Vector { _handler: ETH },
        Vector { _handler: ETH_WKUP },
        Vector { _handler: CAN2_TX },
        Vector { _handler: CAN2_RX0 },
        Vector { _handler: CAN2_RX1 },
        Vector { _handler: CAN2_SCE },
        Vector { _handler: OTG_FS },
        Vector {
            _handler: DMA2_Stream5,
        },
        Vector {
            _handler: DMA2_Stream6,
        },
        Vector {
            _handler: DMA2_Stream7,
        },
        Vector { _handler: USART6 },
        Vector { _handler: I2C3_EV },
        Vector { _handler: I2C3_ER },
        Vector {
            _handler: OTG_HS_EP1_OUT,
        },
        Vector {
            _handler: OTG_HS_EP1_IN,
        },
        Vector {
            _handler: OTG_HS_WKUP,
        },
        Vector { _handler: OTG_HS },
        Vector { _handler: DCMI },
        Vector { _handler: CRYP },
        Vector { _handler: HASH_RNG },
        Vector { _handler: FPU },
        Vector { _handler: UART7 },
        Vector { _handler: UART8 },
        Vector { _handler: SPI4 },
        Vector { _handler: SPI5 },
        Vector { _handler: SPI6 },
        Vector { _handler: SAI1 },
        Vector { _handler: LTDC },
        Vector { _handler: LTDC_ER },
        Vector { _handler: DMA2D },
    ];
}
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
impl_gpio_pin!(PJ0, 9, 0, EXTI0);
impl_gpio_pin!(PJ1, 9, 1, EXTI1);
impl_gpio_pin!(PJ2, 9, 2, EXTI2);
impl_gpio_pin!(PJ3, 9, 3, EXTI3);
impl_gpio_pin!(PJ4, 9, 4, EXTI4);
impl_gpio_pin!(PJ5, 9, 5, EXTI5);
impl_gpio_pin!(PJ6, 9, 6, EXTI6);
impl_gpio_pin!(PJ7, 9, 7, EXTI7);
impl_gpio_pin!(PJ8, 9, 8, EXTI8);
impl_gpio_pin!(PJ9, 9, 9, EXTI9);
impl_gpio_pin!(PJ10, 9, 10, EXTI10);
impl_gpio_pin!(PJ11, 9, 11, EXTI11);
impl_gpio_pin!(PJ12, 9, 12, EXTI12);
impl_gpio_pin!(PJ13, 9, 13, EXTI13);
impl_gpio_pin!(PJ14, 9, 14, EXTI14);
impl_gpio_pin!(PJ15, 9, 15, EXTI15);
impl_gpio_pin!(PK0, 10, 0, EXTI0);
impl_gpio_pin!(PK1, 10, 1, EXTI1);
impl_gpio_pin!(PK2, 10, 2, EXTI2);
impl_gpio_pin!(PK3, 10, 3, EXTI3);
impl_gpio_pin!(PK4, 10, 4, EXTI4);
impl_gpio_pin!(PK5, 10, 5, EXTI5);
impl_gpio_pin!(PK6, 10, 6, EXTI6);
impl_gpio_pin!(PK7, 10, 7, EXTI7);
impl_gpio_pin!(PK8, 10, 8, EXTI8);
impl_gpio_pin!(PK9, 10, 9, EXTI9);
impl_gpio_pin!(PK10, 10, 10, EXTI10);
impl_gpio_pin!(PK11, 10, 11, EXTI11);
impl_gpio_pin!(PK12, 10, 12, EXTI12);
impl_gpio_pin!(PK13, 10, 13, EXTI13);
impl_gpio_pin!(PK14, 10, 14, EXTI14);
impl_gpio_pin!(PK15, 10, 15, EXTI15);
impl_rng!(0x50060800);
impl_usart!(USART1, 0x40011000);
impl_usart_pin!(USART1, RxPin, PA10, 7);
impl_usart_pin!(USART1, CtsPin, PA11, 7);
impl_usart_pin!(USART1, RtsPin, PA12, 7);
impl_usart_pin!(USART1, CkPin, PA8, 7);
impl_usart_pin!(USART1, TxPin, PA9, 7);
impl_usart_pin!(USART1, TxPin, PB6, 7);
impl_usart_pin!(USART1, RxPin, PB7, 7);
impl_usart!(USART2, 0x40004400);
impl_usart_pin!(USART2, CtsPin, PA0, 7);
impl_usart_pin!(USART2, RtsPin, PA1, 7);
impl_usart_pin!(USART2, TxPin, PA2, 7);
impl_usart_pin!(USART2, RxPin, PA3, 7);
impl_usart_pin!(USART2, CkPin, PA4, 7);
impl_usart_pin!(USART2, CtsPin, PD3, 7);
impl_usart_pin!(USART2, RtsPin, PD4, 7);
impl_usart_pin!(USART2, TxPin, PD5, 7);
impl_usart_pin!(USART2, RxPin, PD6, 7);
impl_usart_pin!(USART2, CkPin, PD7, 7);
impl_usart!(USART3, 0x40004800);
impl_usart_pin!(USART3, TxPin, PB10, 7);
impl_usart_pin!(USART3, RxPin, PB11, 7);
impl_usart_pin!(USART3, CkPin, PB12, 7);
impl_usart_pin!(USART3, CtsPin, PB13, 7);
impl_usart_pin!(USART3, RtsPin, PB14, 7);
impl_usart_pin!(USART3, TxPin, PC10, 7);
impl_usart_pin!(USART3, RxPin, PC11, 7);
impl_usart_pin!(USART3, CkPin, PC12, 7);
impl_usart_pin!(USART3, CkPin, PD10, 7);
impl_usart_pin!(USART3, CtsPin, PD11, 7);
impl_usart_pin!(USART3, RtsPin, PD12, 7);
impl_usart_pin!(USART3, TxPin, PD8, 7);
impl_usart_pin!(USART3, RxPin, PD9, 7);
impl_usart!(USART6, 0x40011400);
impl_usart_pin!(USART6, TxPin, PC6, 8);
impl_usart_pin!(USART6, RxPin, PC7, 8);
impl_usart_pin!(USART6, CkPin, PC8, 8);
impl_usart_pin!(USART6, RtsPin, PG12, 8);
impl_usart_pin!(USART6, CtsPin, PG13, 8);
impl_usart_pin!(USART6, TxPin, PG14, 8);
impl_usart_pin!(USART6, CtsPin, PG15, 8);
impl_usart_pin!(USART6, CkPin, PG7, 8);
impl_usart_pin!(USART6, RtsPin, PG8, 8);
impl_usart_pin!(USART6, RxPin, PG9, 8);
