use core::convert::Infallible;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use embassy::traits::gpio::{WaitForAnyEdge, WaitForFallingEdge, WaitForRisingEdge};
use embassy::util::{AtomicWaker, Unborrow};
use embedded_hal::digital::v2::InputPin;
use pac::exti::{regs, vals};

use crate::gpio::{AnyPin, Input, Pin as GpioPin};
use crate::pac;
use crate::pac::CORE_INDEX;
use crate::pac::{EXTI, SYSCFG};

const EXTI_COUNT: usize = 16;
const NEW_AW: AtomicWaker = AtomicWaker::new();
static EXTI_WAKERS: [AtomicWaker; EXTI_COUNT] = [NEW_AW; EXTI_COUNT];

pub unsafe fn on_irq() {
    let bits = EXTI.pr(0).read().0;

    // Mask all the channels that fired.
    EXTI.cpu(CORE_INDEX)
        .imr(CORE_INDEX)
        .modify(|w| w.0 &= !bits);

    // Wake the tasks
    for pin in BitIter(bits) {
        EXTI_WAKERS[pin as usize].wake();
    }

    // Clear pending
    EXTI.pr(0).write_value(regs::Pr(bits));
}

struct BitIter(u32);

impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.trailing_zeros() {
            32 => None,
            b => {
                self.0 &= !(1 << b);
                Some(b)
            }
        }
    }
}

/// EXTI input driver
pub struct ExtiInput<'d, T: GpioPin> {
    pin: Input<'d, T>,
}

impl<'d, T: GpioPin> Unpin for ExtiInput<'d, T> {}

impl<'d, T: GpioPin> ExtiInput<'d, T> {
    pub fn new(pin: Input<'d, T>, _ch: impl Unborrow<Target = T::ExtiChannel> + 'd) -> Self {
        Self { pin }
    }
}

impl<'d, T: GpioPin> InputPin for ExtiInput<'d, T> {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.pin.is_high()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.pin.is_low()
    }
}

impl<'d, T: GpioPin> WaitForRisingEdge for ExtiInput<'d, T> {
    type Future<'a> = ExtiInputFuture<'a>;

    fn wait_for_rising_edge<'a>(&'a mut self) -> Self::Future<'a> {
        ExtiInputFuture::new(
            self.pin.pin.pin(),
            self.pin.pin.port(),
            vals::Rt::ENABLED,
            vals::Ft::DISABLED,
        )
    }
}

impl<'d, T: GpioPin> WaitForFallingEdge for ExtiInput<'d, T> {
    type Future<'a> = ExtiInputFuture<'a>;

    fn wait_for_falling_edge<'a>(&'a mut self) -> Self::Future<'a> {
        ExtiInputFuture::new(
            self.pin.pin.pin(),
            self.pin.pin.port(),
            vals::Rt::DISABLED,
            vals::Ft::ENABLED,
        )
    }
}

impl<'d, T: GpioPin> WaitForAnyEdge for ExtiInput<'d, T> {
    type Future<'a> = ExtiInputFuture<'a>;

    fn wait_for_any_edge<'a>(&'a mut self) -> Self::Future<'a> {
        ExtiInputFuture::new(
            self.pin.pin.pin(),
            self.pin.pin.port(),
            vals::Rt::ENABLED,
            vals::Ft::ENABLED,
        )
    }
}

pub struct ExtiInputFuture<'a> {
    pin: u8,
    phantom: PhantomData<&'a mut AnyPin>,
}

impl<'a> ExtiInputFuture<'a> {
    fn new(pin: u8, port: u8, rising: vals::Rt, falling: vals::Ft) -> Self {
        cortex_m::interrupt::free(|_| unsafe {
            let pin = pin as usize;
            SYSCFG.exticr(pin / 4).modify(|w| w.set_exti(pin % 4, port));
            EXTI.rtsr(CORE_INDEX).modify(|w| w.set_rt(pin, rising));
            EXTI.ftsr(CORE_INDEX).modify(|w| w.set_ft(pin, falling));
            EXTI.pr(CORE_INDEX).write(|w| w.set_pif(pin, true)); // clear pending bit
            EXTI.cpu(CORE_INDEX)
                .imr(CORE_INDEX)
                .modify(|w| w.set_im(pin, vals::Mr::UNMASKED));
        });

        Self {
            pin,
            phantom: PhantomData,
        }
    }
}

impl<'a> Drop for ExtiInputFuture<'a> {
    fn drop(&mut self) {
        cortex_m::interrupt::free(|_| unsafe {
            let pin = self.pin as _;
            EXTI.cpu(CORE_INDEX)
                .imr(CORE_INDEX)
                .modify(|w| w.set_im(pin, vals::Mr::MASKED));
        });
    }
}

impl<'a> Future for ExtiInputFuture<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        EXTI_WAKERS[self.pin as usize].register(cx.waker());

        if unsafe {
            EXTI.cpu(CORE_INDEX)
                .imr(CORE_INDEX)
                .read()
                .im(self.pin as _)
                == vals::Mr::MASKED
        } {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

use crate::interrupt;

macro_rules! impl_irq {
    ($e:ident) => {
        #[interrupt]
        unsafe fn $e() {
            on_irq()
        }
    };
}

foreach_exti_irq!(impl_irq);
