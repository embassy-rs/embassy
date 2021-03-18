use core::cell::UnsafeCell;
use core::future::Future;
use core::mem;
use core::pin::Pin;
use cortex_m;

use embassy::interrupt::Interrupt;
use embassy::traits::gpio::{WaitForFallingEdge, WaitForRisingEdge};
use embassy::util::InterruptFuture;

use crate::hal::gpio;
use crate::hal::gpio::Edge;
use crate::hal::syscfg::SysCfg;
use crate::pac::EXTI;
use embedded_hal::digital::v2 as digital;

use crate::interrupt;

pub struct ExtiPin<T: gpio::ExtiPin + WithInterrupt> {
    pin: T,
    interrupt: T::Interrupt,
}

impl<T: gpio::ExtiPin + WithInterrupt> ExtiPin<T> {
    fn new(mut pin: T, interrupt: T::Interrupt) -> Self {
        let mut syscfg: SysCfg = unsafe { mem::transmute(()) };

        cortex_m::interrupt::free(|_| {
            pin.make_interrupt_source(&mut syscfg);
        });

        Self { pin, interrupt }
    }
}

impl<T: gpio::ExtiPin + WithInterrupt + digital::OutputPin> digital::OutputPin for ExtiPin<T> {
    type Error = T::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high()
    }
}

impl<T: gpio::ExtiPin + WithInterrupt + digital::StatefulOutputPin> digital::StatefulOutputPin
    for ExtiPin<T>
{
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        self.pin.is_set_low()
    }

    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.pin.is_set_high()
    }
}

impl<T: gpio::ExtiPin + WithInterrupt + digital::ToggleableOutputPin> digital::ToggleableOutputPin
    for ExtiPin<T>
{
    type Error = T::Error;

    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.pin.toggle()
    }
}

impl<T: gpio::ExtiPin + WithInterrupt + digital::InputPin> digital::InputPin for ExtiPin<T> {
    type Error = T::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.pin.is_high()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.pin.is_low()
    }
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

impl<T: gpio::ExtiPin + WithInterrupt + 'static> WaitForRisingEdge for ExtiPin<T> {
    type Future<'a> = impl Future<Output = ()> + 'a;

    fn wait_for_rising_edge<'a>(self: Pin<&'a mut Self>) -> Self::Future<'a> {
        let s = unsafe { self.get_unchecked_mut() };

        s.pin.clear_interrupt_pending_bit();
        async move {
            let fut = InterruptFuture::new(&mut s.interrupt);
            cortex_m::interrupt::free(|_| {
                let mut exti: EXTI = unsafe { mem::transmute(()) };

                s.pin.trigger_on_edge(&mut exti, Edge::RISING);
                s.pin.enable_interrupt(&mut exti);
            });
            fut.await;

            s.pin.clear_interrupt_pending_bit();
        }
    }
}

impl<T: gpio::ExtiPin + WithInterrupt + 'static> WaitForFallingEdge for ExtiPin<T> {
    type Future<'a> = impl Future<Output = ()> + 'a;

    fn wait_for_falling_edge<'a>(self: Pin<&'a mut Self>) -> Self::Future<'a> {
        let s = unsafe { self.get_unchecked_mut() };

        s.pin.clear_interrupt_pending_bit();
        async move {
            let fut = InterruptFuture::new(&mut s.interrupt);
            cortex_m::interrupt::free(|_| {
                let mut exti: EXTI = unsafe { mem::transmute(()) };

                s.pin.trigger_on_edge(&mut exti, Edge::FALLING);
                s.pin.enable_interrupt(&mut exti);
            });
            fut.await;

            s.pin.clear_interrupt_pending_bit();
        }
    }
}

mod private {
    pub trait Sealed {}
}

pub trait WithInterrupt: private::Sealed {
    type Interrupt: interrupt::Interrupt;
}

macro_rules! exti {
    ($set:ident, [
        $($INT:ident => $pin:ident,)+
    ]) => {
        $(
            impl<T> private::Sealed for gpio::$set::$pin<T> {}
            impl<T> WithInterrupt for gpio::$set::$pin<T> {
                type Interrupt = interrupt::$INT;
            }
        )+

    };
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
exti!(gpioa, [
    EXTI0 => PA0,
    EXTI1 => PA1,
    EXTI2 => PA2,
    EXTI3 => PA3,
    EXTI4 => PA4,
    EXTI9_5 => PA5,
    EXTI9_5 => PA6,
    EXTI9_5 => PA7,
    EXTI9_5 => PA8,
    EXTI9_5 => PA9,
    EXTI15_10 => PA10,
    EXTI15_10 => PA11,
    EXTI15_10 => PA12,
    EXTI15_10 => PA13,
    EXTI15_10 => PA14,
    EXTI15_10 => PA15,
]);

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
exti!(gpiob, [
    EXTI0 => PB0,
    EXTI1 => PB1,
    EXTI2 => PB2,
    EXTI3 => PB3,
    EXTI4 => PB4,
    EXTI9_5 => PB5,
    EXTI9_5 => PB6,
    EXTI9_5 => PB7,
    EXTI9_5 => PB8,
    EXTI9_5 => PB9,
    EXTI15_10 => PB10,
    EXTI15_10 => PB11,
    EXTI15_10 => PB12,
    EXTI15_10 => PB13,
    EXTI15_10 => PB14,
    EXTI15_10 => PB15,
]);

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
exti!(gpioc, [
    EXTI0 => PC0,
    EXTI1 => PC1,
    EXTI2 => PC2,
    EXTI3 => PC3,
    EXTI4 => PC4,
    EXTI9_5 => PC5,
    EXTI9_5 => PC6,
    EXTI9_5 => PC7,
    EXTI9_5 => PC8,
    EXTI9_5 => PC9,
    EXTI15_10 => PC10,
    EXTI15_10 => PC11,
    EXTI15_10 => PC12,
    EXTI15_10 => PC13,
    EXTI15_10 => PC14,
    EXTI15_10 => PC15,
]);

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
exti!(gpiod, [
    EXTI0 => PD0,
    EXTI1 => PD1,
    EXTI2 => PD2,
    EXTI3 => PD3,
    EXTI4 => PD4,
    EXTI9_5 => PD5,
    EXTI9_5 => PD6,
    EXTI9_5 => PD7,
    EXTI9_5 => PD8,
    EXTI9_5 => PD9,
    EXTI15_10 => PD10,
    EXTI15_10 => PD11,
    EXTI15_10 => PD12,
    EXTI15_10 => PD13,
    EXTI15_10 => PD14,
    EXTI15_10 => PD15,
]);

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
exti!(gpioe, [
    EXTI0 => PE0,
    EXTI1 => PE1,
    EXTI2 => PE2,
    EXTI3 => PE3,
    EXTI4 => PE4,
    EXTI9_5 => PE5,
    EXTI9_5 => PE6,
    EXTI9_5 => PE7,
    EXTI9_5 => PE8,
    EXTI9_5 => PE9,
    EXTI15_10 => PE10,
    EXTI15_10 => PE11,
    EXTI15_10 => PE12,
    EXTI15_10 => PE13,
    EXTI15_10 => PE14,
    EXTI15_10 => PE15,
]);

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
exti!(gpiof, [
    EXTI0 => PF0,
    EXTI1 => PF1,
    EXTI2 => PF2,
    EXTI3 => PF3,
    EXTI4 => PF4,
    EXTI9_5 => PF5,
    EXTI9_5 => PF6,
    EXTI9_5 => PF7,
    EXTI9_5 => PF8,
    EXTI9_5 => PF9,
    EXTI15_10 => PF10,
    EXTI15_10 => PF11,
    EXTI15_10 => PF12,
    EXTI15_10 => PF13,
    EXTI15_10 => PF14,
    EXTI15_10 => PF15,
]);

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
exti!(gpiog, [
    EXTI0 => PG0,
    EXTI1 => PG1,
    EXTI2 => PG2,
    EXTI3 => PG3,
    EXTI4 => PG4,
    EXTI9_5 => PG5,
    EXTI9_5 => PG6,
    EXTI9_5 => PG7,
    EXTI9_5 => PG8,
    EXTI9_5 => PG9,
    EXTI15_10 => PG10,
    EXTI15_10 => PG11,
    EXTI15_10 => PG12,
    EXTI15_10 => PG13,
    EXTI15_10 => PG14,
    EXTI15_10 => PG15,
]);

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
exti!(gpioh, [
    EXTI0 => PH0,
    EXTI1 => PH1,
    EXTI2 => PH2,
    EXTI3 => PH3,
    EXTI4 => PH4,
    EXTI9_5 => PH5,
    EXTI9_5 => PH6,
    EXTI9_5 => PH7,
    EXTI9_5 => PH8,
    EXTI9_5 => PH9,
    EXTI15_10 => PH10,
    EXTI15_10 => PH11,
    EXTI15_10 => PH12,
    EXTI15_10 => PH13,
    EXTI15_10 => PH14,
    EXTI15_10 => PH15,
]);

#[cfg(any(feature = "stm32f401"))]
exti!(gpioh, [
    EXTI0 => PH0,
    EXTI1 => PH1,
]);

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
exti!(gpioi, [
    EXTI0 => PI0,
    EXTI1 => PI1,
    EXTI2 => PI2,
    EXTI3 => PI3,
    EXTI4 => PI4,
    EXTI9_5 => PI5,
    EXTI9_5 => PI6,
    EXTI9_5 => PI7,
    EXTI9_5 => PI8,
    EXTI9_5 => PI9,
    EXTI15_10 => PI10,
    EXTI15_10 => PI11,
    EXTI15_10 => PI12,
    EXTI15_10 => PI13,
    EXTI15_10 => PI14,
    EXTI15_10 => PI15,
]);

#[cfg(any(
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti!(gpioj, [
    EXTI0 => PJ0,
    EXTI1 => PJ1,
    EXTI2 => PJ2,
    EXTI3 => PJ3,
    EXTI4 => PJ4,
    EXTI9_5 => PJ5,
    EXTI9_5 => PJ6,
    EXTI9_5 => PJ7,
    EXTI9_5 => PJ8,
    EXTI9_5 => PJ9,
    EXTI15_10 => PJ10,
    EXTI15_10 => PJ11,
    EXTI15_10 => PJ12,
    EXTI15_10 => PJ13,
    EXTI15_10 => PJ14,
    EXTI15_10 => PJ15,
]);

#[cfg(any(
    feature = "stm32f427",
    feature = "stm32f429",
    feature = "stm32f437",
    feature = "stm32f439",
    feature = "stm32f469",
    feature = "stm32f479"
))]
exti!(gpiok, [
    EXTI0 => PK0,
    EXTI1 => PK1,
    EXTI2 => PK2,
    EXTI3 => PK3,
    EXTI4 => PK4,
    EXTI9_5 => PK5,
    EXTI9_5 => PK6,
    EXTI9_5 => PK7,
]);
