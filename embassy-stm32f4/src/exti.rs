use core::future::Future;
use core::mem;
use core::pin::Pin;

use embassy::gpio::{WaitForFallingEdge, WaitForRisingEdge};
use embassy::interrupt::OwnedInterrupt;
use embassy::util::InterruptFuture;

use crate::hal::gpio;
use crate::hal::gpio::{Edge, ExtiPin as HalExtiPin};
use crate::hal::syscfg::SysCfg;
use crate::pac::EXTI;

use crate::interrupt;

pub struct ExtiManager {
    syscfg: SysCfg,
}

impl<'a> ExtiManager {
    pub fn new(_exti: EXTI, syscfg: SysCfg) -> Self {
        Self { syscfg }
    }

    pub fn new_pin<T, I>(&'static mut self, mut pin: T, interrupt: I) -> ExtiPin<T, I>
    where
        T: HalExtiPin + WithInterrupt<Instance = I>,
        I: OwnedInterrupt,
    {
        pin.make_interrupt_source(&mut self.syscfg);

        ExtiPin {
            pin,
            interrupt,
            _mgr: self,
        }
    }
}

pub struct ExtiPin<T: HalExtiPin, I: OwnedInterrupt> {
    pin: T,
    interrupt: I,
    _mgr: &'static ExtiManager,
}

/*
    Irq	Handler	Description
    EXTI0_IRQn	EXTI0_IRQHandler	Handler for pins connected to line 0
    EXTI1_IRQn	EXTI1_IRQHandler	Handler for pins connected to line 1
    EXTI2_IRQn	EXTI2_IRQHandler	Handler for pins connected to line 2
    EXTI3_IRQn	EXTI3_IRQHandler	Handler for pins connected to line 3
    EXTI4_IRQn	EXTI4_IRQHandler	Handler for pins connected to line 4
    EXTI9_5_IRQn	EXTI9_5_IRQHandler	Handler for pins connected to line 5 to 9
    EXTI15_10_IRQn	EXTI15_10_IRQHandler	Handler for pins connected to line 10 to 15
*/

impl<T: HalExtiPin + 'static, I: OwnedInterrupt + 'static> WaitForRisingEdge for ExtiPin<T, I> {
    type Future<'a> = impl Future<Output = ()> + 'a;

    fn wait_for_rising_edge<'a>(self: Pin<&'a mut Self>) -> Self::Future<'a> {
        let s = unsafe { self.get_unchecked_mut() };

        s.pin.clear_interrupt_pending_bit();
        async move {
            let fut = InterruptFuture::new(&mut s.interrupt);
            let mut exti: EXTI = unsafe { mem::transmute(()) };

            s.pin.trigger_on_edge(&mut exti, Edge::RISING);
            s.pin.enable_interrupt(&mut exti);
            fut.await;

            s.pin.clear_interrupt_pending_bit();
        }
    }
}

impl<T: HalExtiPin + 'static, I: OwnedInterrupt + 'static> WaitForFallingEdge for ExtiPin<T, I> {
    type Future<'a> = impl Future<Output = ()> + 'a;

    fn wait_for_falling_edge<'a>(self: Pin<&'a mut Self>) -> Self::Future<'a> {
        let s = unsafe { self.get_unchecked_mut() };

        s.pin.clear_interrupt_pending_bit();
        async move {
            let fut = InterruptFuture::new(&mut s.interrupt);
            let mut exti: EXTI = unsafe { mem::transmute(()) };

            s.pin.trigger_on_edge(&mut exti, Edge::FALLING);
            s.pin.enable_interrupt(&mut exti);
            fut.await;

            s.pin.clear_interrupt_pending_bit();
        }
    }
}

mod private {
    pub trait Sealed {}
}

pub trait WithInterrupt: private::Sealed {
    type Instance;
}

macro_rules! exti {
    ($($PER:ident => ($set:ident, $pin:ident),)+) => {
        $(
            impl<T> private::Sealed for gpio::$set::$pin<T> {}
            impl<T> WithInterrupt for gpio::$set::$pin<T> {
                type Instance = interrupt::$PER;
            }
        )+
    }
}

#[cfg(any(
    feature = "stm32f401",
    feature = "stm32f405",
    feature = "stm32f407",
    feature = "stm32f410",
    feature = "stm32f411",
    feature = "stm32f412",
    feature = "stm32f413",
    feature = "stm32f415",
    feature = "stm32f417",
    feature = "stm32f423",
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f446",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti! {
    EXTI0Interrupt => (gpioa, PA0),
    EXTI1Interrupt => (gpioa, PA1),
    EXTI2Interrupt => (gpioa, PA2),
    EXTI3Interrupt => (gpioa, PA3),
    EXTI4Interrupt => (gpioa, PA4),
    EXTI9_5Interrupt => (gpioa, PA5),
    EXTI9_5Interrupt => (gpioa, PA6),
    EXTI9_5Interrupt => (gpioa, PA7),
    EXTI9_5Interrupt => (gpioa, PA8),
    EXTI9_5Interrupt => (gpioa, PA9),
    EXTI15_10Interrupt => (gpioa, PA10),
    EXTI15_10Interrupt => (gpioa, PA11),
    EXTI15_10Interrupt => (gpioa, PA12),
    EXTI15_10Interrupt => (gpioa, PA13),
    EXTI15_10Interrupt => (gpioa, PA14),
    EXTI15_10Interrupt => (gpioa, PA15),
}

#[cfg(any(
    feature = "stm32f401",
    feature = "stm32f405",
    feature = "stm32f407",
    feature = "stm32f410",
    feature = "stm32f411",
    feature = "stm32f412",
    feature = "stm32f413",
    feature = "stm32f415",
    feature = "stm32f417",
    feature = "stm32f423",
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f446",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti! {
    EXTI0Interrupt => (gpiob, PB0),
    EXTI1Interrupt => (gpiob, PB1),
    EXTI2Interrupt => (gpiob, PB2),
    EXTI3Interrupt => (gpiob, PB3),
    EXTI4Interrupt => (gpiob, PB4),
    EXTI9_5Interrupt => (gpiob, PB5),
    EXTI9_5Interrupt => (gpiob, PB6),
    EXTI9_5Interrupt => (gpiob, PB7),
    EXTI9_5Interrupt => (gpiob, PB8),
    EXTI9_5Interrupt => (gpiob, PB9),
    EXTI15_10Interrupt => (gpiob, PB10),
    EXTI15_10Interrupt => (gpiob, PB11),
    EXTI15_10Interrupt => (gpiob, PB12),
    EXTI15_10Interrupt => (gpiob, PB13),
    EXTI15_10Interrupt => (gpiob, PB14),
    EXTI15_10Interrupt => (gpiob, PB15),
}

#[cfg(any(
    feature = "stm32f401",
    feature = "stm32f405",
    feature = "stm32f407",
    feature = "stm32f410",
    feature = "stm32f411",
    feature = "stm32f412",
    feature = "stm32f413",
    feature = "stm32f415",
    feature = "stm32f417",
    feature = "stm32f423",
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f446",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti! {
    EXTI0Interrupt => (gpioc, PC0),
    EXTI1Interrupt => (gpioc, PC1),
    EXTI2Interrupt => (gpioc, PC2),
    EXTI3Interrupt => (gpioc, PC3),
    EXTI4Interrupt => (gpioc, PC4),
    EXTI9_5Interrupt => (gpioc, PC5),
    EXTI9_5Interrupt => (gpioc, PC6),
    EXTI9_5Interrupt => (gpioc, PC7),
    EXTI9_5Interrupt => (gpioc, PC8),
    EXTI9_5Interrupt => (gpioc, PC9),
    EXTI15_10Interrupt => (gpioc, PC10),
    EXTI15_10Interrupt => (gpioc, PC11),
    EXTI15_10Interrupt => (gpioc, PC12),
    EXTI15_10Interrupt => (gpioc, PC13),
    EXTI15_10Interrupt => (gpioc, PC14),
    EXTI15_10Interrupt => (gpioc, PC15),
}

#[cfg(any(
    feature = "stm32f401",
    feature = "stm32f405",
    feature = "stm32f407",
    feature = "stm32f411",
    feature = "stm32f412",
    feature = "stm32f413",
    feature = "stm32f415",
    feature = "stm32f417",
    feature = "stm32f423",
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f446",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti! {
    EXTI0Interrupt => (gpiod, PD0),
    EXTI1Interrupt => (gpiod, PD1),
    EXTI2Interrupt => (gpiod, PD2),
    EXTI3Interrupt => (gpiod, PD3),
    EXTI4Interrupt => (gpiod, PD4),
    EXTI9_5Interrupt => (gpiod, PD5),
    EXTI9_5Interrupt => (gpiod, PD6),
    EXTI9_5Interrupt => (gpiod, PD7),
    EXTI9_5Interrupt => (gpiod, PD8),
    EXTI9_5Interrupt => (gpiod, PD9),
    EXTI15_10Interrupt => (gpiod, PD10),
    EXTI15_10Interrupt => (gpiod, PD11),
    EXTI15_10Interrupt => (gpiod, PD12),
    EXTI15_10Interrupt => (gpiod, PD13),
    EXTI15_10Interrupt => (gpiod, PD14),
    EXTI15_10Interrupt => (gpiod, PD15),
}

#[cfg(any(
    feature = "stm32f401",
    feature = "stm32f405",
    feature = "stm32f407",
    feature = "stm32f411",
    feature = "stm32f412",
    feature = "stm32f413",
    feature = "stm32f415",
    feature = "stm32f417",
    feature = "stm32f423",
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f446",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti! {
    EXTI0Interrupt => (gpioe, PE0),
    EXTI1Interrupt => (gpioe, PE1),
    EXTI2Interrupt => (gpioe, PE2),
    EXTI3Interrupt => (gpioe, PE3),
    EXTI4Interrupt => (gpioe, PE4),
    EXTI9_5Interrupt => (gpioe, PE5),
    EXTI9_5Interrupt => (gpioe, PE6),
    EXTI9_5Interrupt => (gpioe, PE7),
    EXTI9_5Interrupt => (gpioe, PE8),
    EXTI9_5Interrupt => (gpioe, PE9),
    EXTI15_10Interrupt => (gpioe, PE10),
    EXTI15_10Interrupt => (gpioe, PE11),
    EXTI15_10Interrupt => (gpioe, PE12),
    EXTI15_10Interrupt => (gpioe, PE13),
    EXTI15_10Interrupt => (gpioe, PE14),
    EXTI15_10Interrupt => (gpioe, PE15),
}

#[cfg(any(
    feature = "stm32f405",
    feature = "stm32f407",
    feature = "stm32f412",
    feature = "stm32f413",
    feature = "stm32f415",
    feature = "stm32f417",
    feature = "stm32f423",
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f446",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti! {
    EXTI0Interrupt => (gpiof, PF0),
    EXTI1Interrupt => (gpiof, PF1),
    EXTI2Interrupt => (gpiof, PF2),
    EXTI3Interrupt => (gpiof, PF3),
    EXTI4Interrupt => (gpiof, PF4),
    EXTI9_5Interrupt => (gpiof, PF5),
    EXTI9_5Interrupt => (gpiof, PF6),
    EXTI9_5Interrupt => (gpiof, PF7),
    EXTI9_5Interrupt => (gpiof, PF8),
    EXTI9_5Interrupt => (gpiof, PF9),
    EXTI15_10Interrupt => (gpiof, PF10),
    EXTI15_10Interrupt => (gpiof, PF11),
    EXTI15_10Interrupt => (gpiof, PF12),
    EXTI15_10Interrupt => (gpiof, PF13),
    EXTI15_10Interrupt => (gpiof, PF14),
    EXTI15_10Interrupt => (gpiof, PF15),
}

#[cfg(any(
    feature = "stm32f405",
    feature = "stm32f407",
    feature = "stm32f412",
    feature = "stm32f413",
    feature = "stm32f415",
    feature = "stm32f417",
    feature = "stm32f423",
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f446",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti! {
    EXTI0Interrupt => (gpiog, PG0),
    EXTI1Interrupt => (gpiog, PG1),
    EXTI2Interrupt => (gpiog, PG2),
    EXTI3Interrupt => (gpiog, PG3),
    EXTI4Interrupt => (gpiog, PG4),
    EXTI9_5Interrupt => (gpiog, PG5),
    EXTI9_5Interrupt => (gpiog, PG6),
    EXTI9_5Interrupt => (gpiog, PG7),
    EXTI9_5Interrupt => (gpiog, PG8),
    EXTI9_5Interrupt => (gpiog, PG9),
    EXTI15_10Interrupt => (gpiog, PG10),
    EXTI15_10Interrupt => (gpiog, PG11),
    EXTI15_10Interrupt => (gpiog, PG12),
    EXTI15_10Interrupt => (gpiog, PG13),
    EXTI15_10Interrupt => (gpiog, PG14),
    EXTI15_10Interrupt => (gpiog, PG15),
}

#[cfg(any(
    feature = "stm32f405",
    feature = "stm32f407",
    feature = "stm32f410",
    feature = "stm32f411",
    feature = "stm32f412",
    feature = "stm32f413",
    feature = "stm32f415",
    feature = "stm32f417",
    feature = "stm32f423",
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f446",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti! {
    EXTI0Interrupt => (gpioh, PH0),
    EXTI1Interrupt => (gpioh, PH1),
    EXTI2Interrupt => (gpioh, PH2),
    EXTI3Interrupt => (gpioh, PH3),
    EXTI4Interrupt => (gpioh, PH4),
    EXTI9_5Interrupt => (gpioh, PH5),
    EXTI9_5Interrupt => (gpioh, PH6),
    EXTI9_5Interrupt => (gpioh, PH7),
    EXTI9_5Interrupt => (gpioh, PH8),
    EXTI9_5Interrupt => (gpioh, PH9),
    EXTI15_10Interrupt => (gpioh, PH10),
    EXTI15_10Interrupt => (gpioh, PH11),
    EXTI15_10Interrupt => (gpioh, PH12),
    EXTI15_10Interrupt => (gpioh, PH13),
    EXTI15_10Interrupt => (gpioh, PH14),
    EXTI15_10Interrupt => (gpioh, PH15),
}

#[cfg(any(feature = "stm32f401"))]
exti! {
    EXTI0Interrupt => (gpioh, PH0),
    EXTI1Interrupt => (gpioh, PH1),
}

#[cfg(any(
    feature = "stm32f405",
    feature = "stm32f407",
    feature = "stm32f415",
    feature = "stm32f417",
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti! {
    EXTI0Interrupt => (gpioi, PI0),
    EXTI1Interrupt => (gpioi, PI1),
    EXTI2Interrupt => (gpioi, PI2),
    EXTI3Interrupt => (gpioi, PI3),
    EXTI4Interrupt => (gpioi, PI4),
    EXTI9_5Interrupt => (gpioi, PI5),
    EXTI9_5Interrupt => (gpioi, PI6),
    EXTI9_5Interrupt => (gpioi, PI7),
    EXTI9_5Interrupt => (gpioi, PI8),
    EXTI9_5Interrupt => (gpioi, PI9),
    EXTI15_10Interrupt => (gpioi, PI10),
    EXTI15_10Interrupt => (gpioi, PI11),
    EXTI15_10Interrupt => (gpioi, PI12),
    EXTI15_10Interrupt => (gpioi, PI13),
    EXTI15_10Interrupt => (gpioi, PI14),
    EXTI15_10Interrupt => (gpioi, PI15),
}

#[cfg(any(
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti! {
    EXTI0Interrupt => (gpioj, PJ0),
    EXTI1Interrupt => (gpioj, PJ1),
    EXTI2Interrupt => (gpioj, PJ2),
    EXTI3Interrupt => (gpioj, PJ3),
    EXTI4Interrupt => (gpioj, PJ4),
    EXTI9_5Interrupt => (gpioj, PJ5),
    EXTI9_5Interrupt => (gpioj, PJ6),
    EXTI9_5Interrupt => (gpioj, PJ7),
    EXTI9_5Interrupt => (gpioj, PJ8),
    EXTI9_5Interrupt => (gpioj, PJ9),
    EXTI15_10Interrupt => (gpioj, PJ10),
    EXTI15_10Interrupt => (gpioj, PJ11),
    EXTI15_10Interrupt => (gpioj, PJ12),
    EXTI15_10Interrupt => (gpioj, PJ13),
    EXTI15_10Interrupt => (gpioj, PJ14),
    EXTI15_10Interrupt => (gpioj, PJ15),
}

#[cfg(any(
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti! {
    EXTI0Interrupt => (gpiok, PK0),
    EXTI1Interrupt => (gpiok, PK1),
    EXTI2Interrupt => (gpiok, PK2),
    EXTI3Interrupt => (gpiok, PK3),
    EXTI4Interrupt => (gpiok, PK4),
    EXTI9_5Interrupt => (gpiok, PK5),
    EXTI9_5Interrupt => (gpiok, PK6),
    EXTI9_5Interrupt => (gpiok, PK7),
}
