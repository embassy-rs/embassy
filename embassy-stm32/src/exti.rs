use core::convert::Infallible;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use embassy::traits::gpio::{WaitForAnyEdge, WaitForFallingEdge, WaitForRisingEdge};
use embassy::util::{AtomicWaker, Unborrow};
use embassy_hal_common::unsafe_impl_unborrow;
use embedded_hal::digital::v2::InputPin;

use crate::gpio::{AnyPin, Input, Pin as GpioPin};
use crate::interrupt;
use crate::pac;
use crate::pac::{EXTI, SYSCFG};
use crate::peripherals;

const EXTI_COUNT: usize = 16;
const NEW_AW: AtomicWaker = AtomicWaker::new();
static EXTI_WAKERS: [AtomicWaker; EXTI_COUNT] = [NEW_AW; EXTI_COUNT];

#[cfg(exti_w)]
fn cpu_regs() -> pac::exti::Cpu {
    EXTI.cpu(crate::pac::CORE_INDEX)
}

#[cfg(not(exti_w))]
fn cpu_regs() -> pac::exti::Exti {
    EXTI
}

pub unsafe fn on_irq() {
    let bits = EXTI.pr(0).read();

    // Mask all the channels that fired.
    cpu_regs().imr(0).modify(|w| w.0 &= !bits.0);

    // Wake the tasks
    for pin in BitIter(bits.0) {
        EXTI_WAKERS[pin as usize].wake();
    }

    // Clear pending
    EXTI.pr(0).write_value(bits);
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
        ExtiInputFuture::new(self.pin.pin.pin(), self.pin.pin.port(), true, false)
    }
}

impl<'d, T: GpioPin> WaitForFallingEdge for ExtiInput<'d, T> {
    type Future<'a> = ExtiInputFuture<'a>;

    fn wait_for_falling_edge<'a>(&'a mut self) -> Self::Future<'a> {
        ExtiInputFuture::new(self.pin.pin.pin(), self.pin.pin.port(), false, true)
    }
}

impl<'d, T: GpioPin> WaitForAnyEdge for ExtiInput<'d, T> {
    type Future<'a> = ExtiInputFuture<'a>;

    fn wait_for_any_edge<'a>(&'a mut self) -> Self::Future<'a> {
        ExtiInputFuture::new(self.pin.pin.pin(), self.pin.pin.port(), true, true)
    }
}

pub struct ExtiInputFuture<'a> {
    pin: u8,
    phantom: PhantomData<&'a mut AnyPin>,
}

impl<'a> ExtiInputFuture<'a> {
    fn new(pin: u8, port: u8, rising: bool, falling: bool) -> Self {
        cortex_m::interrupt::free(|_| unsafe {
            let pin = pin as usize;
            SYSCFG.exticr(pin / 4).modify(|w| w.set_exti(pin % 4, port));
            EXTI.rtsr(0).modify(|w| w.set_line(pin, rising));
            EXTI.ftsr(0).modify(|w| w.set_line(pin, falling));
            EXTI.pr(0).write(|w| w.set_line(pin, true)); // clear pending bit
            cpu_regs().imr(0).modify(|w| w.set_line(pin, true));
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
            cpu_regs().imr(0).modify(|w| w.set_line(pin, false));
        });
    }
}

impl<'a> Future for ExtiInputFuture<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        EXTI_WAKERS[self.pin as usize].register(cx.waker());

        let imr = unsafe { cpu_regs().imr(0).read() };
        if !imr.line(self.pin as _) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

macro_rules! foreach_exti_irq {
    ($action:ident) => {
        crate::pac::interrupts!(
            (EXTI0)  => { $action!(EXTI0); };
            (EXTI1)  => { $action!(EXTI1); };
            (EXTI2)  => { $action!(EXTI2); };
            (EXTI3)  => { $action!(EXTI3); };
            (EXTI4)  => { $action!(EXTI4); };
            (EXTI5)  => { $action!(EXTI5); };
            (EXTI6)  => { $action!(EXTI6); };
            (EXTI7)  => { $action!(EXTI7); };
            (EXTI8)  => { $action!(EXTI8); };
            (EXTI9)  => { $action!(EXTI9); };
            (EXTI10) => { $action!(EXTI10); };
            (EXTI11) => { $action!(EXTI11); };
            (EXTI12) => { $action!(EXTI12); };
            (EXTI13) => { $action!(EXTI13); };
            (EXTI14) => { $action!(EXTI14); };
            (EXTI15) => { $action!(EXTI15); };

            // plus the weird ones
            (EXTI0_1)   => { $action!( EXTI0_1 ); };
            (EXTI15_10) => { $action!(EXTI15_10); };
            (EXTI15_4)  => { $action!(EXTI15_4); };
            (EXTI1_0)   => { $action!(EXTI1_0); };
            (EXTI2_3)   => { $action!(EXTI2_3); };
            (EXTI2_TSC) => { $action!(EXTI2_TSC); };
            (EXTI3_2)   => { $action!(EXTI3_2); };
            (EXTI4_15)  => { $action!(EXTI4_15); };
            (EXTI9_5)   => { $action!(EXTI9_5); };
        );
    };
}

macro_rules! impl_irq {
    ($e:ident) => {
        #[interrupt]
        unsafe fn $e() {
            on_irq()
        }
    };
}

foreach_exti_irq!(impl_irq);

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

macro_rules! enable_irq {
    ($e:ident) => {
        crate::interrupt::$e::steal().enable();
    };
}

/// safety: must be called only once
pub(crate) unsafe fn init() {
    use embassy::interrupt::Interrupt;
    use embassy::interrupt::InterruptExt;

    foreach_exti_irq!(enable_irq);

    #[cfg(not(any(rcc_wb, rcc_wl5)))]
    <crate::peripherals::SYSCFG as crate::rcc::sealed::RccPeripheral>::enable();
}
