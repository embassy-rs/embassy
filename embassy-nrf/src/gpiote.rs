use core::convert::Infallible;
use core::future::Future;
use core::marker::PhantomData;
use core::task::{Context, Poll};
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unsafe_impl_unborrow;
use futures::future::poll_fn;

use crate::gpio::sealed::Pin as _;
use crate::gpio::{AnyPin, Flex, Input, Output, Pin as GpioPin};
use crate::pac;
use crate::ppi::{Event, Task};
use crate::{interrupt, peripherals};

pub const CHANNEL_COUNT: usize = 8;

#[cfg(any(feature = "nrf52833", feature = "nrf52840"))]
pub const PIN_COUNT: usize = 48;
#[cfg(not(any(feature = "nrf52833", feature = "nrf52840")))]
pub const PIN_COUNT: usize = 32;

#[allow(clippy::declare_interior_mutable_const)]
const NEW_AW: AtomicWaker = AtomicWaker::new();
static CHANNEL_WAKERS: [AtomicWaker; CHANNEL_COUNT] = [NEW_AW; CHANNEL_COUNT];
static PORT_WAKERS: [AtomicWaker; PIN_COUNT] = [NEW_AW; PIN_COUNT];

pub enum InputChannelPolarity {
    None,
    HiToLo,
    LoToHi,
    Toggle,
}

/// Polarity of the `task out` operation.
pub enum OutputChannelPolarity {
    Set,
    Clear,
    Toggle,
}

fn regs() -> &'static pac::gpiote::RegisterBlock {
    cfg_if::cfg_if! {
        if #[cfg(any(feature="nrf5340-app-s", feature="nrf9160-s"))] {
            unsafe { &*pac::GPIOTE0::ptr() }
        } else if #[cfg(any(feature="nrf5340-app-ns", feature="nrf9160-ns"))] {
            unsafe { &*pac::GPIOTE1::ptr() }
        } else {
            unsafe { &*pac::GPIOTE::ptr() }
        }
    }
}

pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    #[cfg(any(feature = "nrf52833", feature = "nrf52840"))]
    let ports = unsafe { &[&*pac::P0::ptr(), &*pac::P1::ptr()] };
    #[cfg(not(any(feature = "nrf52833", feature = "nrf52840")))]
    let ports = unsafe { &[&*pac::P0::ptr()] };

    for &p in ports {
        // Enable latched detection
        p.detectmode.write(|w| w.detectmode().ldetect());
        // Clear latch
        p.latch.write(|w| unsafe { w.bits(0xFFFFFFFF) })
    }

    // Enable interrupts
    cfg_if::cfg_if! {
        if #[cfg(any(feature="nrf5340-app-s", feature="nrf9160-s"))] {
            let irq = unsafe { interrupt::GPIOTE0::steal() };
        } else if #[cfg(any(feature="nrf5340-app-ns", feature="nrf9160-ns"))] {
            let irq = unsafe { interrupt::GPIOTE1::steal() };
        } else {
            let irq = unsafe { interrupt::GPIOTE::steal() };
        }
    }

    irq.unpend();
    irq.set_priority(irq_prio);
    irq.enable();

    let g = regs();
    g.events_port.write(|w| w);
    g.intenset.write(|w| w.port().set());
}

cfg_if::cfg_if! {
    if #[cfg(any(feature="nrf5340-app-s", feature="nrf9160-s"))] {
        #[interrupt]
        fn GPIOTE0() {
            unsafe { handle_gpiote_interrupt() };
        }
    } else if #[cfg(any(feature="nrf5340-app-ns", feature="nrf9160-ns"))] {
        #[interrupt]
        fn GPIOTE1() {
            unsafe { handle_gpiote_interrupt() };
        }
    } else {
        #[interrupt]
        fn GPIOTE() {
            unsafe { handle_gpiote_interrupt() };
        }
    }
}

unsafe fn handle_gpiote_interrupt() {
    let g = regs();

    for i in 0..CHANNEL_COUNT {
        if g.events_in[i].read().bits() != 0 {
            g.intenclr.write(|w| w.bits(1 << i));
            CHANNEL_WAKERS[i].wake();
        }
    }

    if g.events_port.read().bits() != 0 {
        g.events_port.write(|w| w);

        #[cfg(any(feature = "nrf52833", feature = "nrf52840"))]
        let ports = &[&*pac::P0::ptr(), &*pac::P1::ptr()];
        #[cfg(not(any(feature = "nrf52833", feature = "nrf52840")))]
        let ports = &[&*pac::P0::ptr()];

        for (port, &p) in ports.iter().enumerate() {
            let bits = p.latch.read().bits();
            for pin in BitIter(bits) {
                p.pin_cnf[pin as usize].modify(|_, w| w.sense().disabled());
                PORT_WAKERS[port * 32 + pin as usize].wake();
            }
            p.latch.write(|w| w.bits(bits));
        }
    }
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

/// GPIOTE channel driver in input mode
pub struct InputChannel<'d, C: Channel, T: GpioPin> {
    ch: C,
    pin: Input<'d, T>,
}

impl<'d, C: Channel, T: GpioPin> Drop for InputChannel<'d, C, T> {
    fn drop(&mut self) {
        let g = regs();
        let num = self.ch.number();
        g.config[num].write(|w| w.mode().disabled());
        g.intenclr.write(|w| unsafe { w.bits(1 << num) });
    }
}

impl<'d, C: Channel, T: GpioPin> InputChannel<'d, C, T> {
    pub fn new(ch: C, pin: Input<'d, T>, polarity: InputChannelPolarity) -> Self {
        let g = regs();
        let num = ch.number();

        g.config[num].write(|w| {
            match polarity {
                InputChannelPolarity::HiToLo => w.mode().event().polarity().hi_to_lo(),
                InputChannelPolarity::LoToHi => w.mode().event().polarity().lo_to_hi(),
                InputChannelPolarity::None => w.mode().event().polarity().none(),
                InputChannelPolarity::Toggle => w.mode().event().polarity().toggle(),
            };
            #[cfg(any(feature = "nrf52833", feature = "nrf52840"))]
            w.port().bit(match pin.pin.pin.port() {
                crate::gpio::Port::Port0 => false,
                crate::gpio::Port::Port1 => true,
            });
            unsafe { w.psel().bits(pin.pin.pin.pin()) }
        });

        g.events_in[num].reset();

        InputChannel { ch, pin }
    }

    pub async fn wait(&self) {
        let g = regs();
        let num = self.ch.number();

        // Enable interrupt
        g.events_in[num].reset();
        g.intenset.write(|w| unsafe { w.bits(1 << num) });

        poll_fn(|cx| {
            CHANNEL_WAKERS[num].register(cx.waker());

            if g.events_in[num].read().bits() != 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    /// Returns the IN event, for use with PPI.
    pub fn event_in(&self) -> Event {
        let g = regs();
        Event::from_reg(&g.events_in[self.ch.number()])
    }
}

/// GPIOTE channel driver in output mode
pub struct OutputChannel<'d, C: Channel, T: GpioPin> {
    ch: C,
    _pin: Output<'d, T>,
}

impl<'d, C: Channel, T: GpioPin> Drop for OutputChannel<'d, C, T> {
    fn drop(&mut self) {
        let g = regs();
        let num = self.ch.number();
        g.config[num].write(|w| w.mode().disabled());
        g.intenclr.write(|w| unsafe { w.bits(1 << num) });
    }
}

impl<'d, C: Channel, T: GpioPin> OutputChannel<'d, C, T> {
    pub fn new(ch: C, pin: Output<'d, T>, polarity: OutputChannelPolarity) -> Self {
        let g = regs();
        let num = ch.number();

        g.config[num].write(|w| {
            w.mode().task();
            match pin.is_set_high() {
                true => w.outinit().high(),
                false => w.outinit().low(),
            };
            match polarity {
                OutputChannelPolarity::Set => w.polarity().lo_to_hi(),
                OutputChannelPolarity::Clear => w.polarity().hi_to_lo(),
                OutputChannelPolarity::Toggle => w.polarity().toggle(),
            };
            #[cfg(any(feature = "nrf52833", feature = "nrf52840"))]
            w.port().bit(match pin.pin.pin.port() {
                crate::gpio::Port::Port0 => false,
                crate::gpio::Port::Port1 => true,
            });
            unsafe { w.psel().bits(pin.pin.pin.pin()) }
        });

        OutputChannel { ch, _pin: pin }
    }

    /// Triggers `task out` (as configured with task_out_polarity, defaults to Toggle).
    pub fn out(&self) {
        let g = regs();
        g.tasks_out[self.ch.number()].write(|w| unsafe { w.bits(1) });
    }

    /// Triggers `task set` (set associated pin high).
    #[cfg(not(feature = "nrf51"))]
    pub fn set(&self) {
        let g = regs();
        g.tasks_set[self.ch.number()].write(|w| unsafe { w.bits(1) });
    }

    /// Triggers `task clear` (set associated pin low).
    #[cfg(not(feature = "nrf51"))]
    pub fn clear(&self) {
        let g = regs();
        g.tasks_clr[self.ch.number()].write(|w| unsafe { w.bits(1) });
    }

    /// Returns the OUT task, for use with PPI.
    pub fn task_out(&self) -> Task {
        let g = regs();
        Task::from_reg(&g.tasks_out[self.ch.number()])
    }

    /// Returns the CLR task, for use with PPI.
    #[cfg(not(feature = "nrf51"))]
    pub fn task_clr(&self) -> Task {
        let g = regs();
        Task::from_reg(&g.tasks_clr[self.ch.number()])
    }

    /// Returns the SET task, for use with PPI.
    #[cfg(not(feature = "nrf51"))]
    pub fn task_set(&self) -> Task {
        let g = regs();
        Task::from_reg(&g.tasks_set[self.ch.number()])
    }
}

// =======================

pub(crate) struct PortInputFuture<'a> {
    pin_port: u8,
    phantom: PhantomData<&'a mut AnyPin>,
}

impl<'a> Unpin for PortInputFuture<'a> {}

impl<'a> Drop for PortInputFuture<'a> {
    fn drop(&mut self) {
        let pin = unsafe { AnyPin::steal(self.pin_port) };
        pin.conf().modify(|_, w| w.sense().disabled());
    }
}

impl<'a> Future for PortInputFuture<'a> {
    type Output = ();

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        PORT_WAKERS[self.pin_port as usize].register(cx.waker());

        let pin = unsafe { AnyPin::steal(self.pin_port) };
        if pin.conf().read().sense().is_disabled() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl<'d, T: GpioPin> Input<'d, T> {
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await
    }

    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await
    }

    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await
    }

    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await
    }

    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await
    }
}

impl<'d, T: GpioPin> Flex<'d, T> {
    pub async fn wait_for_high(&mut self) {
        self.pin.conf().modify(|_, w| w.sense().high());

        PortInputFuture {
            pin_port: self.pin.pin_port(),
            phantom: PhantomData,
        }
        .await
    }

    pub async fn wait_for_low(&mut self) {
        self.pin.conf().modify(|_, w| w.sense().low());

        PortInputFuture {
            pin_port: self.pin.pin_port(),
            phantom: PhantomData,
        }
        .await
    }

    pub async fn wait_for_rising_edge(&mut self) {
        self.wait_for_low().await;
        self.wait_for_high().await;
    }

    pub async fn wait_for_falling_edge(&mut self) {
        self.wait_for_high().await;
        self.wait_for_low().await;
    }

    pub async fn wait_for_any_edge(&mut self) {
        if self.is_high() {
            self.pin.conf().modify(|_, w| w.sense().low());
        } else {
            self.pin.conf().modify(|_, w| w.sense().high());
        }
        PortInputFuture {
            pin_port: self.pin.pin_port(),
            phantom: PhantomData,
        }
        .await
    }
}

// =======================

mod sealed {
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

macro_rules! impl_channel {
    ($type:ident, $number:expr) => {
        impl sealed::Channel for peripherals::$type {}
        impl Channel for peripherals::$type {
            fn number(&self) -> usize {
                $number as usize
            }
        }
    };
}

impl_channel!(GPIOTE_CH0, 0);
impl_channel!(GPIOTE_CH1, 1);
impl_channel!(GPIOTE_CH2, 2);
impl_channel!(GPIOTE_CH3, 3);
impl_channel!(GPIOTE_CH4, 4);
impl_channel!(GPIOTE_CH5, 5);
impl_channel!(GPIOTE_CH6, 6);
impl_channel!(GPIOTE_CH7, 7);

// ====================

mod eh02 {
    use super::*;

    impl<'d, C: Channel, T: GpioPin> embedded_hal_02::digital::v2::InputPin for InputChannel<'d, C, T> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.pin.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.pin.is_low())
        }
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl<'d, C: Channel, T: GpioPin> embedded_hal_1::digital::ErrorType for InputChannel<'d, C, T> {
        type Error = Infallible;
    }

    impl<'d, C: Channel, T: GpioPin> embedded_hal_1::digital::blocking::InputPin
        for InputChannel<'d, C, T>
    {
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.pin.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.pin.is_low())
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "unstable-traits", feature = "nightly"))] {
        use futures::FutureExt;

        impl<'d, T: GpioPin> embedded_hal_async::digital::Wait for Input<'d, T> {
            type WaitForHighFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn wait_for_high<'a>(&'a mut self) -> Self::WaitForHighFuture<'a> {
                self.wait_for_high().map(Ok)
            }

            type WaitForLowFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn wait_for_low<'a>(&'a mut self) -> Self::WaitForLowFuture<'a> {
                self.wait_for_low().map(Ok)
            }

            type WaitForRisingEdgeFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn wait_for_rising_edge<'a>(&'a mut self) -> Self::WaitForRisingEdgeFuture<'a> {
                self.wait_for_rising_edge().map(Ok)
            }

            type WaitForFallingEdgeFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn wait_for_falling_edge<'a>(&'a mut self) -> Self::WaitForFallingEdgeFuture<'a> {
                self.wait_for_falling_edge().map(Ok)
            }

            type WaitForAnyEdgeFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn wait_for_any_edge<'a>(&'a mut self) -> Self::WaitForAnyEdgeFuture<'a> {
                self.wait_for_any_edge().map(Ok)
            }
        }

        impl<'d, T: GpioPin> embedded_hal_async::digital::Wait for Flex<'d, T> {
            type WaitForHighFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn wait_for_high<'a>(&'a mut self) -> Self::WaitForHighFuture<'a> {
                self.wait_for_high().map(Ok)
            }

            type WaitForLowFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn wait_for_low<'a>(&'a mut self) -> Self::WaitForLowFuture<'a> {
                self.wait_for_low().map(Ok)
            }

            type WaitForRisingEdgeFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn wait_for_rising_edge<'a>(&'a mut self) -> Self::WaitForRisingEdgeFuture<'a> {
                self.wait_for_rising_edge().map(Ok)
            }

            type WaitForFallingEdgeFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn wait_for_falling_edge<'a>(&'a mut self) -> Self::WaitForFallingEdgeFuture<'a> {
                self.wait_for_falling_edge().map(Ok)
            }

            type WaitForAnyEdgeFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn wait_for_any_edge<'a>(&'a mut self) -> Self::WaitForAnyEdgeFuture<'a> {
                self.wait_for_any_edge().map(Ok)
            }
        }
    }
}
