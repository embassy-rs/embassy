#![macro_use]
use core::convert::Infallible;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use embassy::traits::gpio::{WaitForAnyEdge, WaitForFallingEdge, WaitForRisingEdge};
use embassy::util::{AtomicWaker, Unborrow};
use embassy_extras::unsafe_impl_unborrow;
use embedded_hal::digital::v2::InputPin;
use pac::exti::{regs, vals};

use crate::gpio::{AnyPin, Input, Pin as GpioPin};
use crate::pac;
use crate::pac::{EXTI, SYSCFG};
use crate::peripherals;

const EXTI_COUNT: usize = 16;
const NEW_AW: AtomicWaker = AtomicWaker::new();
static EXTI_WAKERS: [AtomicWaker; EXTI_COUNT] = [NEW_AW; EXTI_COUNT];

pub unsafe fn on_irq() {
    let bits = EXTI.pr().read().0;

    // Mask all the channels that fired.
    EXTI.imr().modify(|w| w.0 &= !bits);

    // Wake the tasks
    for pin in BitIter(bits) {
        EXTI_WAKERS[pin as usize].wake();
    }

    // Clear pending
    EXTI.pr().write_value(regs::Pr(bits));
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
            vals::Tr::ENABLED,
            vals::Tr::DISABLED,
        )
    }
}

impl<'d, T: GpioPin> WaitForFallingEdge for ExtiInput<'d, T> {
    type Future<'a> = ExtiInputFuture<'a>;

    fn wait_for_falling_edge<'a>(&'a mut self) -> Self::Future<'a> {
        ExtiInputFuture::new(
            self.pin.pin.pin(),
            self.pin.pin.port(),
            vals::Tr::DISABLED,
            vals::Tr::ENABLED,
        )
    }
}

impl<'d, T: GpioPin> WaitForAnyEdge for ExtiInput<'d, T> {
    type Future<'a> = ExtiInputFuture<'a>;

    fn wait_for_any_edge<'a>(&'a mut self) -> Self::Future<'a> {
        ExtiInputFuture::new(
            self.pin.pin.pin(),
            self.pin.pin.port(),
            vals::Tr::ENABLED,
            vals::Tr::ENABLED,
        )
    }
}

pub struct ExtiInputFuture<'a> {
    pin: u8,
    phantom: PhantomData<&'a mut AnyPin>,
}

impl<'a> ExtiInputFuture<'a> {
    fn new(pin: u8, port: u8, rising: vals::Tr, falling: vals::Tr) -> Self {
        cortex_m::interrupt::free(|_| unsafe {
            let pin = pin as usize;
            SYSCFG.exticr(pin / 4).modify(|w| w.set_exti(pin % 4, port));
            EXTI.rtsr().modify(|w| w.set_tr(pin, rising));
            EXTI.ftsr().modify(|w| w.set_tr(pin, falling));
            EXTI.pr().write(|w| w.set_pr(pin, true)); // clear pending bit
            EXTI.imr().modify(|w| w.set_mr(pin, vals::Mr::UNMASKED));
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
            EXTI.imr().modify(|w| w.set_mr(pin, vals::Mr::MASKED));
        });
    }
}

impl<'a> Future for ExtiInputFuture<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        EXTI_WAKERS[self.pin as usize].register(cx.waker());

        if unsafe { EXTI.imr().read().mr(self.pin as _) == vals::Mr::MASKED } {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub(crate) mod sealed {
    pub trait Channel {}
}

pub trait Channel: sealed::Channel + Sized {
    fn number(&self) -> usize;
    fn degrade(self) -> AnyChannel {
        AnyChannel {
            number: self.number() as u8,
        }
    }
}

pub struct AnyChannel {
    number: u8,
}
unsafe_impl_unborrow!(AnyChannel);
impl sealed::Channel for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> usize {
        self.number as usize
    }
}

macro_rules! impl_exti {
    ($type:ident, $number:expr) => {
        impl sealed::Channel for peripherals::$type {}
        impl Channel for peripherals::$type {
            fn number(&self) -> usize {
                $number as usize
            }
        }
    };
}

impl_exti!(EXTI0, 0);
impl_exti!(EXTI1, 1);
impl_exti!(EXTI2, 2);
impl_exti!(EXTI3, 3);
impl_exti!(EXTI4, 4);
impl_exti!(EXTI5, 5);
impl_exti!(EXTI6, 6);
impl_exti!(EXTI7, 7);
impl_exti!(EXTI8, 8);
impl_exti!(EXTI9, 9);
impl_exti!(EXTI10, 10);
impl_exti!(EXTI11, 11);
impl_exti!(EXTI12, 12);
impl_exti!(EXTI13, 13);
impl_exti!(EXTI14, 14);
impl_exti!(EXTI15, 15);

macro_rules! impl_exti_irq {
    ($($e:ident),+) => {
        /// safety: must be called only once
        pub(crate) unsafe fn init_exti() {
            $(
                crate::interrupt::$e::steal().enable();
            )+
        }

        $(
            #[crate::interrupt]
            unsafe fn $e() {
                crate::exti::on_irq()
            }
        )+
    };
}
