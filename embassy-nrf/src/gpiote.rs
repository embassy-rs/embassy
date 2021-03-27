use core::convert::Infallible;
use core::future::Future;
use core::intrinsics::transmute;
use core::marker::PhantomData;
use core::mem::{self, ManuallyDrop};
use core::ops::Deref;
use core::pin::Pin;
use core::ptr;
use core::task::{Context, Poll};
use embassy::interrupt::InterruptExt;
use embassy::traits::gpio::{WaitForHigh, WaitForLow};
use embassy::util::{AtomicWaker, PeripheralBorrow, Signal};
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};

use crate::gpio::sealed::Pin as _;
use crate::gpio::{AnyPin, Input, Pin as GpioPin, Pull};
use crate::pac;
use crate::pac::generic::Reg;
use crate::pac::gpiote::_TASKS_OUT;
use crate::pac::p0 as pac_gpio;
use crate::{interrupt, peripherals};

#[cfg(not(feature = "51"))]
use crate::pac::gpiote::{_TASKS_CLR, _TASKS_SET};

pub const CHANNEL_COUNT: usize = 8;

#[cfg(any(feature = "52833", feature = "52840"))]
pub const PIN_COUNT: usize = 48;
#[cfg(not(any(feature = "52833", feature = "52840")))]
pub const PIN_COUNT: usize = 32;

pub trait ChannelID {
    fn number(&self) -> usize;
}

macro_rules! impl_channel {
    ($ChX:ident, $n:expr) => {
        pub struct $ChX(());
        impl $ChX {
            pub fn degrade(self) -> ChAny {
                ChAny($n)
            }
        }

        impl ChannelID for $ChX {
            fn number(&self) -> usize {
                $n
            }
        }
    };
}

impl_channel!(Ch0, 0);
impl_channel!(Ch1, 1);
impl_channel!(Ch2, 2);
impl_channel!(Ch3, 3);
impl_channel!(Ch4, 4);
impl_channel!(Ch5, 5);
impl_channel!(Ch6, 6);
impl_channel!(Ch7, 7);

pub struct ChAny(u8);

impl ChannelID for ChAny {
    fn number(&self) -> usize {
        self.0 as usize
    }
}

const NEW_AWR: AtomicWaker = AtomicWaker::new();
static CHANNEL_WAKERS: [AtomicWaker; CHANNEL_COUNT] = [NEW_AWR; CHANNEL_COUNT];
static PORT_WAKERS: [AtomicWaker; PIN_COUNT] = [NEW_AWR; PIN_COUNT];

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

/// Token indicating GPIOTE has been correctly initialized.
///
/// This is not an owned singleton, it is Copy. Drivers that make use of GPIOTE require it.
#[derive(Clone, Copy)]
pub struct Initialized {
    _private: (),
}

pub fn initialize(gpiote: peripherals::GPIOTE, irq: interrupt::GPIOTE) -> Initialized {
    #[cfg(any(feature = "52833", feature = "52840"))]
    let ports = unsafe { &[&*pac::P0::ptr(), &*pac::P1::ptr()] };
    #[cfg(not(any(feature = "52833", feature = "52840")))]
    let ports = unsafe { &[&*pac::P0::ptr()] };

    for &p in ports {
        // Enable latched detection
        p.detectmode.write(|w| w.detectmode().ldetect());
        // Clear latch
        p.latch.write(|w| unsafe { w.bits(0xFFFFFFFF) })
    }

    // Enable interrupts
    let g = unsafe { &*pac::GPIOTE::ptr() };
    g.events_port.write(|w| w);
    g.intenset.write(|w| w.port().set());
    irq.set_handler(on_irq);
    irq.unpend();
    irq.enable();

    Initialized { _private: () }
}

unsafe fn on_irq(_ctx: *mut ()) {
    let g = &*pac::GPIOTE::ptr();

    for i in 0..CHANNEL_COUNT {
        if g.events_in[i].read().bits() != 0 {
            g.events_in[i].write(|w| w);
            CHANNEL_WAKERS[i].wake();
        }
    }

    if g.events_port.read().bits() != 0 {
        g.events_port.write(|w| w);

        #[cfg(any(feature = "52833", feature = "52840"))]
        let ports = &[&*pac::P0::ptr(), &*pac::P1::ptr()];
        #[cfg(not(any(feature = "52833", feature = "52840")))]
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

/*
pub struct InputChannel<C: ChannelID, T> {
    ch: C,
    pin: GpioPin<Input<T>>,
}

impl<C: ChannelID, T> Drop for InputChannel<C, T> {
    fn drop(&mut self) {
        let g = unsafe { &*GPIOTE::ptr() };
        let index = self.ch.number();
        g.config[index].write(|w| w.mode().disabled());
        g.intenclr.write(|w| unsafe { w.bits(1 << index) });
    }
}

impl<C: ChannelID, T> InputChannel<C, T> {
    pub fn new(
        _init: Initialized,
        ch: C,
        pin: GpioPin<Input<T>>,
        polarity: InputChannelPolarity,
    ) -> Self {
        let g = unsafe { &*GPIOTE::ptr() };
        let index = ch.number();

        g.config[index].write(|w| {
            match polarity {
                InputChannelPolarity::HiToLo => w.mode().event().polarity().hi_to_lo(),
                InputChannelPolarity::LoToHi => w.mode().event().polarity().lo_to_hi(),
                InputChannelPolarity::None => w.mode().event().polarity().none(),
                InputChannelPolarity::Toggle => w.mode().event().polarity().toggle(),
            };
            #[cfg(any(feature = "52833", feature = "52840"))]
            w.port().bit(match pin.port() {
                Port::Port0 => false,
                Port::Port1 => true,
            });
            unsafe { w.psel().bits(pin.pin()) }
        });

        CHANNEL_WAKERS[index].reset();

        // Enable interrupt
        g.intenset.write(|w| unsafe { w.bits(1 << index) });

        InputChannel { ch, pin }
    }

    pub fn free(self) -> (C, GpioPin<Input<T>>) {
        let m = ManuallyDrop::new(self);
        let ch = unsafe { ptr::read(&m.ch) };
        let pin = unsafe { ptr::read(&m.pin) };
        (ch, pin)
    }

    pub async fn wait(&self) {
        let index = self.ch.number();
        CHANNEL_WAKERS[index].wait().await;
    }

    pub fn pin(&self) -> &GpioPin<Input<T>> {
        &self.pin
    }
}

pub struct OutputChannel<C: ChannelID, T> {
    ch: C,
    pin: GpioPin<Output<T>>,
}

impl<C: ChannelID, T> Drop for OutputChannel<C, T> {
    fn drop(&mut self) {
        let g = unsafe { &*GPIOTE::ptr() };
        let index = self.ch.number();
        g.config[index].write(|w| w.mode().disabled());
        g.intenclr.write(|w| unsafe { w.bits(1 << index) });
    }
}

impl<C: ChannelID, T> OutputChannel<C, T> {
    pub fn new(
        _gpiote: Gpiote,
        ch: C,
        pin: GpioPin<Output<T>>,
        level: Level,
        polarity: OutputChannelPolarity,
    ) -> Self {
        let g = unsafe { &*GPIOTE::ptr() };
        let index = ch.number();

        g.config[index].write(|w| {
            w.mode().task();
            match level {
                Level::High => w.outinit().high(),
                Level::Low => w.outinit().low(),
            };
            match polarity {
                OutputChannelPolarity::Set => w.polarity().lo_to_hi(),
                OutputChannelPolarity::Clear => w.polarity().hi_to_lo(),
                OutputChannelPolarity::Toggle => w.polarity().toggle(),
            };
            #[cfg(any(feature = "52833", feature = "52840"))]
            w.port().bit(match pin.port() {
                Port::Port0 => false,
                Port::Port1 => true,
            });
            unsafe { w.psel().bits(pin.pin()) }
        });

        // Enable interrupt
        g.intenset.write(|w| unsafe { w.bits(1 << index) });

        OutputChannel { ch, pin }
    }

    pub fn free(self) -> (C, GpioPin<Output<T>>) {
        let m = ManuallyDrop::new(self);
        let ch = unsafe { ptr::read(&m.ch) };
        let pin = unsafe { ptr::read(&m.pin) };
        (ch, pin)
    }

    /// Triggers `task out` (as configured with task_out_polarity, defaults to Toggle).
    pub fn out(&self) {
        let g = unsafe { &*GPIOTE::ptr() };
        let index = self.ch.number();

        g.tasks_out[index].write(|w| unsafe { w.bits(1) });
    }
    /// Triggers `task set` (set associated pin high).
    #[cfg(not(feature = "51"))]
    pub fn set(&self) {
        let g = unsafe { &*GPIOTE::ptr() };
        let index = self.ch.number();

        g.tasks_set[index].write(|w| unsafe { w.bits(1) });
    }
    /// Triggers `task clear` (set associated pin low).
    #[cfg(not(feature = "51"))]
    pub fn clear(&self) {
        let g = unsafe { &*GPIOTE::ptr() };
        let index = self.ch.number();

        g.tasks_clr[index].write(|w| unsafe { w.bits(1) });
    }

    /// Returns reference to task_out endpoint for PPI.
    pub fn task_out(&self) -> &Reg<u32, _TASKS_OUT> {
        let g = unsafe { &*GPIOTE::ptr() };
        let index = self.ch.number();

        &g.tasks_out[index]
    }

    /// Returns reference to task_clr endpoint for PPI.
    #[cfg(not(feature = "51"))]
    pub fn task_clr(&self) -> &Reg<u32, _TASKS_CLR> {
        let g = unsafe { &*GPIOTE::ptr() };
        let index = self.ch.number();

        &g.tasks_clr[index]
    }

    /// Returns reference to task_set endpoint for PPI.
    #[cfg(not(feature = "51"))]
    pub fn task_set(&self) -> &Reg<u32, _TASKS_SET> {
        let g = unsafe { &*GPIOTE::ptr() };
        let index = self.ch.number();

        &g.tasks_set[index]
    }
}
 */

/// GPIO input driver with support
pub struct PortInput<'d, T: GpioPin> {
    pin: Input<'d, T>,
}

impl<'d, T: GpioPin> Unpin for PortInput<'d, T> {}

impl<'d, T: GpioPin> PortInput<'d, T> {
    pub fn new(_init: Initialized, pin: Input<'d, T>) -> Self {
        Self { pin }
    }
}

impl<'d, T: GpioPin> InputPin for PortInput<'d, T> {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.pin.is_high()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.pin.is_low()
    }
}

impl<'d, T: GpioPin> WaitForHigh for PortInput<'d, T> {
    type Future<'a> = PortInputFuture<'a>;

    fn wait_for_high<'a>(self: Pin<&'a mut Self>) -> Self::Future<'a> {
        self.pin.pin.conf().modify(|_, w| w.sense().high());

        PortInputFuture {
            pin_port: self.pin.pin.pin_port(),
            phantom: PhantomData,
        }
    }
}

impl<'d, T: GpioPin> WaitForLow for PortInput<'d, T> {
    type Future<'a> = PortInputFuture<'a>;

    fn wait_for_low<'a>(self: Pin<&'a mut Self>) -> Self::Future<'a> {
        self.pin.pin.conf().modify(|_, w| w.sense().low());

        PortInputFuture {
            pin_port: self.pin.pin.pin_port(),
            phantom: PhantomData,
        }
    }
}

pub struct PortInputFuture<'a> {
    pin_port: u8,
    phantom: PhantomData<&'a mut AnyPin>,
}

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
