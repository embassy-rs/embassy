use core::future::Future;
use core::mem;
use core::pin::Pin;

use embassy::traits::gpio::{WaitForAnyEdge, WaitForFallingEdge, WaitForRisingEdge};
use embassy::util::InterruptFuture;

use crate::hal::{
    exti::{Exti, ExtiLine, GpioLine, TriggerEdge},
    gpio,
    syscfg::SYSCFG,
};
use crate::interrupt;
use crate::pac::EXTI;

pub struct ExtiManager {
    syscfg: SYSCFG,
}

impl<'a> ExtiManager {
    pub fn new(_exti: Exti, syscfg: SYSCFG) -> Self {
        Self { syscfg }
    }

    pub fn new_pin<T>(&'static mut self, pin: T, interrupt: T::Interrupt) -> ExtiPin<T>
    where
        T: PinWithInterrupt,
    {
        ExtiPin {
            pin,
            interrupt,
            mgr: self,
        }
    }
}

pub struct ExtiPin<T: PinWithInterrupt> {
    pin: T,
    interrupt: T::Interrupt,
    mgr: &'static ExtiManager,
}

impl<T: PinWithInterrupt + 'static> ExtiPin<T> {
    fn wait_for_edge<'a>(
        self: Pin<&'a mut Self>,
        edge: TriggerEdge,
    ) -> impl Future<Output = ()> + 'a {
        let line = self.pin.line();
        let s = unsafe { self.get_unchecked_mut() };

        Exti::unpend(line);

        async move {
            let exti: EXTI = unsafe { mem::transmute(()) };
            let mut exti = Exti::new(exti);

            let fut = InterruptFuture::new(&mut s.interrupt);

            let port = s.pin.port();
            let syscfg = &s.mgr.syscfg as *const _ as *mut SYSCFG;
            cortex_m::interrupt::free(|_| {
                let syscfg = unsafe { &mut *syscfg };
                exti.listen_gpio(syscfg, port, line, edge);
            });

            fut.await;

            Exti::unpend(line);
        }
    }
}

impl<T: PinWithInterrupt + 'static> WaitForRisingEdge for ExtiPin<T> {
    type Future<'a> = impl Future<Output = ()> + 'a;

    fn wait_for_rising_edge<'a>(self: Pin<&'a mut Self>) -> Self::Future<'a> {
        self.wait_for_edge(TriggerEdge::Rising)
    }
}

impl<T: PinWithInterrupt + 'static> WaitForFallingEdge for ExtiPin<T> {
    type Future<'a> = impl Future<Output = ()> + 'a;

    fn wait_for_falling_edge<'a>(self: Pin<&'a mut Self>) -> Self::Future<'a> {
        self.wait_for_edge(TriggerEdge::Falling)
    }
}

impl<T: PinWithInterrupt + 'static> WaitForAnyEdge for ExtiPin<T> {
    type Future<'a> = impl Future<Output = ()> + 'a;

    fn wait_for_any_edge<'a>(self: Pin<&'a mut Self>) -> Self::Future<'a> {
        self.wait_for_edge(TriggerEdge::Both)
    }
}

mod private {
    pub trait Sealed {}
}

pub trait PinWithInterrupt: private::Sealed {
    type Interrupt: interrupt::Interrupt;
    fn port(&self) -> gpio::Port;
    fn line(&self) -> GpioLine;
}

macro_rules! exti {
    ($($PER:ident => ($set:ident, $pin:ident),)+) => {
        $(
            impl<T> private::Sealed for gpio::$set::$pin<T> {}
            impl<T> PinWithInterrupt for gpio::$set::$pin<T> {
                type Interrupt = interrupt::$PER;
                fn port(&self) -> gpio::Port {
                    self.port()
                }
                fn line(&self) -> GpioLine {
                    GpioLine::from_raw_line(self.pin_number()).unwrap()
                }
            }
        )+
    }
}

exti! {
    EXTI0_1 => (gpioa, PA0),
    EXTI0_1 => (gpioa, PA1),
    EXTI2_3 => (gpioa, PA2),
    EXTI2_3 => (gpioa, PA3),
    EXTI4_15 => (gpioa, PA4),
    EXTI4_15 => (gpioa, PA5),
    EXTI4_15 => (gpioa, PA6),
    EXTI4_15 => (gpioa, PA7),
    EXTI4_15 => (gpioa, PA8),
    EXTI4_15 => (gpioa, PA9),
    EXTI4_15 => (gpioa, PA10),
    EXTI4_15 => (gpioa, PA11),
    EXTI4_15 => (gpioa, PA12),
    EXTI4_15 => (gpioa, PA13),
    EXTI4_15 => (gpioa, PA14),
    EXTI4_15 => (gpioa, PA15),
}

exti! {
    EXTI0_1 => (gpiob, PB0),
    EXTI0_1 => (gpiob, PB1),
    EXTI2_3 => (gpiob, PB2),
    EXTI2_3 => (gpiob, PB3),
    EXTI4_15 => (gpiob, PB4),
    EXTI4_15 => (gpiob, PB5),
    EXTI4_15 => (gpiob, PB6),
    EXTI4_15 => (gpiob, PB7),
    EXTI4_15 => (gpiob, PB8),
    EXTI4_15 => (gpiob, PB9),
    EXTI4_15 => (gpiob, PB10),
    EXTI4_15 => (gpiob, PB11),
    EXTI4_15 => (gpiob, PB12),
    EXTI4_15 => (gpiob, PB13),
    EXTI4_15 => (gpiob, PB14),
    EXTI4_15 => (gpiob, PB15),
}

exti! {
    EXTI0_1 => (gpioc, PC0),
    EXTI0_1 => (gpioc, PC1),
    EXTI2_3 => (gpioc, PC2),
    EXTI2_3 => (gpioc, PC3),
    EXTI4_15 => (gpioc, PC4),
    EXTI4_15 => (gpioc, PC5),
    EXTI4_15 => (gpioc, PC6),
    EXTI4_15 => (gpioc, PC7),
    EXTI4_15 => (gpioc, PC8),
    EXTI4_15 => (gpioc, PC9),
    EXTI4_15 => (gpioc, PC10),
    EXTI4_15 => (gpioc, PC11),
    EXTI4_15 => (gpioc, PC12),
    EXTI4_15 => (gpioc, PC13),
    EXTI4_15 => (gpioc, PC14),
    EXTI4_15 => (gpioc, PC15),
}

exti! {
    EXTI0_1 => (gpiod, PD0),
    EXTI0_1 => (gpiod, PD1),
    EXTI2_3 => (gpiod, PD2),
    EXTI2_3 => (gpiod, PD3),
    EXTI4_15 => (gpiod, PD4),
    EXTI4_15 => (gpiod, PD5),
    EXTI4_15 => (gpiod, PD6),
    EXTI4_15 => (gpiod, PD7),
    EXTI4_15 => (gpiod, PD8),
    EXTI4_15 => (gpiod, PD9),
    EXTI4_15 => (gpiod, PD10),
    EXTI4_15 => (gpiod, PD11),
    EXTI4_15 => (gpiod, PD12),
    EXTI4_15 => (gpiod, PD13),
    EXTI4_15 => (gpiod, PD14),
    EXTI4_15 => (gpiod, PD15),
}

exti! {
    EXTI0_1 => (gpioe, PE0),
    EXTI0_1 => (gpioe, PE1),
    EXTI2_3 => (gpioe, PE2),
    EXTI2_3 => (gpioe, PE3),
    EXTI4_15 => (gpioe, PE4),
    EXTI4_15 => (gpioe, PE5),
    EXTI4_15 => (gpioe, PE6),
    EXTI4_15 => (gpioe, PE7),
    EXTI4_15 => (gpioe, PE8),
    EXTI4_15 => (gpioe, PE9),
    EXTI4_15 => (gpioe, PE10),
    EXTI4_15 => (gpioe, PE11),
    EXTI4_15 => (gpioe, PE12),
    EXTI4_15 => (gpioe, PE13),
    EXTI4_15 => (gpioe, PE14),
    EXTI4_15 => (gpioe, PE15),
}

exti! {
    EXTI0_1 => (gpioh, PH0),
    EXTI0_1 => (gpioh, PH1),
    EXTI4_15 => (gpioh, PH9),
    EXTI4_15 => (gpioh, PH10),
}
