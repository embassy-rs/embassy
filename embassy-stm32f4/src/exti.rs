use core::future::Future;
use core::mem;
use core::pin::Pin;

use embassy::interrupt::Interrupt;
use embassy::traits::gpio::{WaitForFallingEdge, WaitForRisingEdge};
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
        I: Interrupt,
    {
        pin.make_interrupt_source(&mut self.syscfg);

        ExtiPin {
            pin,
            interrupt,
            _mgr: self,
        }
    }
}

pub struct ExtiPin<T: HalExtiPin, I: Interrupt> {
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

impl<T: HalExtiPin + 'static, I: Interrupt + 'static> WaitForRisingEdge for ExtiPin<T, I> {
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

impl<T: HalExtiPin + 'static, I: Interrupt + 'static> WaitForFallingEdge for ExtiPin<T, I> {
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
    EXTI0 => (gpioa, PA0),
    EXTI1 => (gpioa, PA1),
    EXTI2 => (gpioa, PA2),
    EXTI3 => (gpioa, PA3),
    EXTI4 => (gpioa, PA4),
    EXTI9_5 => (gpioa, PA5),
    EXTI9_5 => (gpioa, PA6),
    EXTI9_5 => (gpioa, PA7),
    EXTI9_5 => (gpioa, PA8),
    EXTI9_5 => (gpioa, PA9),
    EXTI15_10 => (gpioa, PA10),
    EXTI15_10 => (gpioa, PA11),
    EXTI15_10 => (gpioa, PA12),
    EXTI15_10 => (gpioa, PA13),
    EXTI15_10 => (gpioa, PA14),
    EXTI15_10 => (gpioa, PA15),
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
    EXTI0 => (gpiob, PB0),
    EXTI1 => (gpiob, PB1),
    EXTI2 => (gpiob, PB2),
    EXTI3 => (gpiob, PB3),
    EXTI4 => (gpiob, PB4),
    EXTI9_5 => (gpiob, PB5),
    EXTI9_5 => (gpiob, PB6),
    EXTI9_5 => (gpiob, PB7),
    EXTI9_5 => (gpiob, PB8),
    EXTI9_5 => (gpiob, PB9),
    EXTI15_10 => (gpiob, PB10),
    EXTI15_10 => (gpiob, PB11),
    EXTI15_10 => (gpiob, PB12),
    EXTI15_10 => (gpiob, PB13),
    EXTI15_10 => (gpiob, PB14),
    EXTI15_10 => (gpiob, PB15),
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
    EXTI0 => (gpioc, PC0),
    EXTI1 => (gpioc, PC1),
    EXTI2 => (gpioc, PC2),
    EXTI3 => (gpioc, PC3),
    EXTI4 => (gpioc, PC4),
    EXTI9_5 => (gpioc, PC5),
    EXTI9_5 => (gpioc, PC6),
    EXTI9_5 => (gpioc, PC7),
    EXTI9_5 => (gpioc, PC8),
    EXTI9_5 => (gpioc, PC9),
    EXTI15_10 => (gpioc, PC10),
    EXTI15_10 => (gpioc, PC11),
    EXTI15_10 => (gpioc, PC12),
    EXTI15_10 => (gpioc, PC13),
    EXTI15_10 => (gpioc, PC14),
    EXTI15_10 => (gpioc, PC15),
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
    EXTI0 => (gpiod, PD0),
    EXTI1 => (gpiod, PD1),
    EXTI2 => (gpiod, PD2),
    EXTI3 => (gpiod, PD3),
    EXTI4 => (gpiod, PD4),
    EXTI9_5 => (gpiod, PD5),
    EXTI9_5 => (gpiod, PD6),
    EXTI9_5 => (gpiod, PD7),
    EXTI9_5 => (gpiod, PD8),
    EXTI9_5 => (gpiod, PD9),
    EXTI15_10 => (gpiod, PD10),
    EXTI15_10 => (gpiod, PD11),
    EXTI15_10 => (gpiod, PD12),
    EXTI15_10 => (gpiod, PD13),
    EXTI15_10 => (gpiod, PD14),
    EXTI15_10 => (gpiod, PD15),
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
    EXTI0 => (gpioe, PE0),
    EXTI1 => (gpioe, PE1),
    EXTI2 => (gpioe, PE2),
    EXTI3 => (gpioe, PE3),
    EXTI4 => (gpioe, PE4),
    EXTI9_5 => (gpioe, PE5),
    EXTI9_5 => (gpioe, PE6),
    EXTI9_5 => (gpioe, PE7),
    EXTI9_5 => (gpioe, PE8),
    EXTI9_5 => (gpioe, PE9),
    EXTI15_10 => (gpioe, PE10),
    EXTI15_10 => (gpioe, PE11),
    EXTI15_10 => (gpioe, PE12),
    EXTI15_10 => (gpioe, PE13),
    EXTI15_10 => (gpioe, PE14),
    EXTI15_10 => (gpioe, PE15),
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
    EXTI0 => (gpiof, PF0),
    EXTI1 => (gpiof, PF1),
    EXTI2 => (gpiof, PF2),
    EXTI3 => (gpiof, PF3),
    EXTI4 => (gpiof, PF4),
    EXTI9_5 => (gpiof, PF5),
    EXTI9_5 => (gpiof, PF6),
    EXTI9_5 => (gpiof, PF7),
    EXTI9_5 => (gpiof, PF8),
    EXTI9_5 => (gpiof, PF9),
    EXTI15_10 => (gpiof, PF10),
    EXTI15_10 => (gpiof, PF11),
    EXTI15_10 => (gpiof, PF12),
    EXTI15_10 => (gpiof, PF13),
    EXTI15_10 => (gpiof, PF14),
    EXTI15_10 => (gpiof, PF15),
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
    EXTI0 => (gpiog, PG0),
    EXTI1 => (gpiog, PG1),
    EXTI2 => (gpiog, PG2),
    EXTI3 => (gpiog, PG3),
    EXTI4 => (gpiog, PG4),
    EXTI9_5 => (gpiog, PG5),
    EXTI9_5 => (gpiog, PG6),
    EXTI9_5 => (gpiog, PG7),
    EXTI9_5 => (gpiog, PG8),
    EXTI9_5 => (gpiog, PG9),
    EXTI15_10 => (gpiog, PG10),
    EXTI15_10 => (gpiog, PG11),
    EXTI15_10 => (gpiog, PG12),
    EXTI15_10 => (gpiog, PG13),
    EXTI15_10 => (gpiog, PG14),
    EXTI15_10 => (gpiog, PG15),
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
    EXTI0 => (gpioh, PH0),
    EXTI1 => (gpioh, PH1),
    EXTI2 => (gpioh, PH2),
    EXTI3 => (gpioh, PH3),
    EXTI4 => (gpioh, PH4),
    EXTI9_5 => (gpioh, PH5),
    EXTI9_5 => (gpioh, PH6),
    EXTI9_5 => (gpioh, PH7),
    EXTI9_5 => (gpioh, PH8),
    EXTI9_5 => (gpioh, PH9),
    EXTI15_10 => (gpioh, PH10),
    EXTI15_10 => (gpioh, PH11),
    EXTI15_10 => (gpioh, PH12),
    EXTI15_10 => (gpioh, PH13),
    EXTI15_10 => (gpioh, PH14),
    EXTI15_10 => (gpioh, PH15),
}

#[cfg(any(feature = "stm32f401"))]
exti! {
    EXTI0 => (gpioh, PH0),
    EXTI1 => (gpioh, PH1),
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
    EXTI0 => (gpioi, PI0),
    EXTI1 => (gpioi, PI1),
    EXTI2 => (gpioi, PI2),
    EXTI3 => (gpioi, PI3),
    EXTI4 => (gpioi, PI4),
    EXTI9_5 => (gpioi, PI5),
    EXTI9_5 => (gpioi, PI6),
    EXTI9_5 => (gpioi, PI7),
    EXTI9_5 => (gpioi, PI8),
    EXTI9_5 => (gpioi, PI9),
    EXTI15_10 => (gpioi, PI10),
    EXTI15_10 => (gpioi, PI11),
    EXTI15_10 => (gpioi, PI12),
    EXTI15_10 => (gpioi, PI13),
    EXTI15_10 => (gpioi, PI14),
    EXTI15_10 => (gpioi, PI15),
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
    EXTI0 => (gpioj, PJ0),
    EXTI1 => (gpioj, PJ1),
    EXTI2 => (gpioj, PJ2),
    EXTI3 => (gpioj, PJ3),
    EXTI4 => (gpioj, PJ4),
    EXTI9_5 => (gpioj, PJ5),
    EXTI9_5 => (gpioj, PJ6),
    EXTI9_5 => (gpioj, PJ7),
    EXTI9_5 => (gpioj, PJ8),
    EXTI9_5 => (gpioj, PJ9),
    EXTI15_10 => (gpioj, PJ10),
    EXTI15_10 => (gpioj, PJ11),
    EXTI15_10 => (gpioj, PJ12),
    EXTI15_10 => (gpioj, PJ13),
    EXTI15_10 => (gpioj, PJ14),
    EXTI15_10 => (gpioj, PJ15),
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
    EXTI0 => (gpiok, PK0),
    EXTI1 => (gpiok, PK1),
    EXTI2 => (gpiok, PK2),
    EXTI3 => (gpiok, PK3),
    EXTI4 => (gpiok, PK4),
    EXTI9_5 => (gpiok, PK5),
    EXTI9_5 => (gpiok, PK6),
    EXTI9_5 => (gpiok, PK7),
}
