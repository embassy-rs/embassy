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
    ($set:ident, [
        $($INT:ident => $pin:ident,)+
    ]) => {
        $(
            impl<T> private::Sealed for gpio::$set::$pin<T> {}
            impl<T> PinWithInterrupt for gpio::$set::$pin<T> {
                type Interrupt = interrupt::$INT;
                fn port(&self) -> gpio::Port {
                    self.port()
                }
                fn line(&self) -> GpioLine {
                    GpioLine::from_raw_line(self.pin_number()).unwrap()
                }
            }
        )+

    };
}

exti!(gpioa, [
    EXTI0_1 => PA0,
    EXTI0_1 => PA1,
    EXTI2_3 => PA2,
    EXTI2_3 => PA3,
    EXTI4_15 => PA4,
    EXTI4_15 => PA5,
    EXTI4_15 => PA6,
    EXTI4_15 => PA7,
    EXTI4_15 => PA8,
    EXTI4_15 => PA9,
    EXTI4_15 => PA10,
    EXTI4_15 => PA11,
    EXTI4_15 => PA12,
    EXTI4_15 => PA13,
    EXTI4_15 => PA14,
    EXTI4_15 => PA15,
]);

exti!(gpiob, [
    EXTI0_1 => PB0,
    EXTI0_1 => PB1,
    EXTI2_3 => PB2,
    EXTI2_3 => PB3,
    EXTI4_15 => PB4,
    EXTI4_15 => PB5,
    EXTI4_15 => PB6,
    EXTI4_15 => PB7,
    EXTI4_15 => PB8,
    EXTI4_15 => PB9,
    EXTI4_15 => PB10,
    EXTI4_15 => PB11,
    EXTI4_15 => PB12,
    EXTI4_15 => PB13,
    EXTI4_15 => PB14,
    EXTI4_15 => PB15,
]);

exti!(gpioc, [
    EXTI0_1 => PC0,
    EXTI0_1 => PC1,
    EXTI2_3 => PC2,
    EXTI2_3 => PC3,
    EXTI4_15 => PC4,
    EXTI4_15 => PC5,
    EXTI4_15 => PC6,
    EXTI4_15 => PC7,
    EXTI4_15 => PC8,
    EXTI4_15 => PC9,
    EXTI4_15 => PC10,
    EXTI4_15 => PC11,
    EXTI4_15 => PC12,
    EXTI4_15 => PC13,
    EXTI4_15 => PC14,
    EXTI4_15 => PC15,
]);

exti!(gpiod, [
    EXTI0_1 => PD0,
    EXTI0_1 => PD1,
    EXTI2_3 => PD2,
    EXTI2_3 => PD3,
    EXTI4_15 => PD4,
    EXTI4_15 => PD5,
    EXTI4_15 => PD6,
    EXTI4_15 => PD7,
    EXTI4_15 => PD8,
    EXTI4_15 => PD9,
    EXTI4_15 => PD10,
    EXTI4_15 => PD11,
    EXTI4_15 => PD12,
    EXTI4_15 => PD13,
    EXTI4_15 => PD14,
    EXTI4_15 => PD15,
]);

exti!(gpioe, [
    EXTI0_1 => PE0,
    EXTI0_1 => PE1,
    EXTI2_3 => PE2,
    EXTI2_3 => PE3,
    EXTI4_15 => PE4,
    EXTI4_15 => PE5,
    EXTI4_15 => PE6,
    EXTI4_15 => PE7,
    EXTI4_15 => PE8,
    EXTI4_15 => PE9,
    EXTI4_15 => PE10,
    EXTI4_15 => PE11,
    EXTI4_15 => PE12,
    EXTI4_15 => PE13,
    EXTI4_15 => PE14,
    EXTI4_15 => PE15,
]);

exti!(gpioh, [
    EXTI0_1 => PH0,
    EXTI0_1 => PH1,
    EXTI4_15 => PH9,
    EXTI4_15 => PH10,
]);
