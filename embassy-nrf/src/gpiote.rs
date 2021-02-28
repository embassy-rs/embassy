use core::future::Future;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::pin::Pin;
use core::ptr;
use core::task::{Context, Poll};
use embassy::gpio::{WaitForHigh, WaitForLow};
use embassy::interrupt::InterruptExt;
use embassy::util::Signal;

use crate::hal::gpio::{Input, Level, Output, Pin as GpioPin, Port};
use crate::interrupt;
use crate::pac;
use crate::pac::generic::Reg;
use crate::pac::gpiote::_TASKS_OUT;
use crate::pac::{p0 as pac_gpio, GPIOTE};

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

#[derive(Clone, Copy)]
pub struct Gpiote(());

const NEW_SIGNAL: Signal<()> = Signal::new();
static CHANNEL_SIGNALS: [Signal<()>; CHANNEL_COUNT] = [NEW_SIGNAL; CHANNEL_COUNT];
static PORT_SIGNALS: [Signal<()>; PIN_COUNT] = [NEW_SIGNAL; PIN_COUNT];

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NewChannelError {
    NoFreeChannels,
}

pub struct Channels {
    pub ch0: Ch0,
    pub ch1: Ch1,
    pub ch2: Ch2,
    pub ch3: Ch3,
    pub ch4: Ch4,
    pub ch5: Ch5,
    pub ch6: Ch6,
    pub ch7: Ch7,
}

impl Gpiote {
    pub fn new(gpiote: GPIOTE, irq: interrupt::GPIOTE) -> (Self, Channels) {
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
        gpiote.events_port.write(|w| w);
        gpiote.intenset.write(|w| w.port().set());
        irq.set_handler(Self::on_irq);
        irq.unpend();
        irq.enable();

        (
            Self(()),
            Channels {
                ch0: Ch0(()),
                ch1: Ch1(()),
                ch2: Ch2(()),
                ch3: Ch3(()),
                ch4: Ch4(()),
                ch5: Ch5(()),
                ch6: Ch6(()),
                ch7: Ch7(()),
            },
        )
    }

    unsafe fn on_irq(_ctx: *mut ()) {
        let g = &*GPIOTE::ptr();

        for (event_in, signal) in g.events_in.iter().zip(CHANNEL_SIGNALS.iter()) {
            if event_in.read().bits() != 0 {
                event_in.write(|w| w);
                signal.signal(());
            }
        }

        if g.events_port.read().bits() != 0 {
            g.events_port.write(|w| w);

            #[cfg(any(feature = "52833", feature = "52840"))]
            let ports = &[&*pac::P0::ptr(), &*pac::P1::ptr()];
            #[cfg(not(any(feature = "52833", feature = "52840")))]
            let ports = &[&*pac::P0::ptr()];

            let mut work = true;
            while work {
                work = false;
                for (port, &p) in ports.iter().enumerate() {
                    for pin in BitIter(p.latch.read().bits()) {
                        work = true;
                        p.pin_cnf[pin as usize].modify(|_, w| w.sense().disabled());
                        p.latch.write(|w| w.bits(1 << pin));
                        PORT_SIGNALS[port * 32 + pin as usize].signal(());
                    }
                }
            }
        }
    }
}

fn pin_num<T>(pin: &GpioPin<T>) -> usize {
    let port = match pin.port() {
        Port::Port0 => 0,
        #[cfg(any(feature = "52833", feature = "52840"))]
        Port::Port1 => 32,
    };

    port + pin.pin() as usize
}

fn pin_block<T>(pin: &GpioPin<T>) -> &pac_gpio::RegisterBlock {
    let ptr = match pin.port() {
        Port::Port0 => pac::P0::ptr(),
        #[cfg(any(feature = "52833", feature = "52840"))]
        Port::Port1 => pac::P1::ptr(),
    };

    unsafe { &*ptr }
}

fn pin_conf<T>(pin: &GpioPin<T>) -> &pac_gpio::PIN_CNF {
    &pin_block(pin).pin_cnf[pin.pin() as usize]
}

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
        _gpiote: Gpiote,
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

        CHANNEL_SIGNALS[index].reset();

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
        CHANNEL_SIGNALS[index].wait().await;
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

pub struct GpiotePin<T> {
    pin: GpioPin<Input<T>>,
}

impl<T> Unpin for GpiotePin<T> {}

impl<T> GpiotePin<T> {
    pub fn new(_gpiote: Gpiote, pin: GpioPin<Input<T>>) -> Self {
        Self { pin }
    }
}

impl<T: 'static> WaitForHigh for GpiotePin<T> {
    type Future<'a> = PortInputFuture<'a, T>;

    fn wait_for_high<'a>(self: Pin<&'a mut Self>) -> Self::Future<'a> {
        PortInputFuture {
            pin: &self.get_mut().pin,
            polarity: PortInputPolarity::High,
        }
    }
}

impl<T: 'static> WaitForLow for GpiotePin<T> {
    type Future<'a> = PortInputFuture<'a, T>;

    fn wait_for_low<'a>(self: Pin<&'a mut Self>) -> Self::Future<'a> {
        PortInputFuture {
            pin: &self.get_mut().pin,
            polarity: PortInputPolarity::Low,
        }
    }
}

impl<T> Deref for GpiotePin<T> {
    type Target = GpioPin<Input<T>>;
    fn deref(&self) -> &Self::Target {
        &self.pin
    }
}

enum PortInputPolarity {
    High,
    Low,
}

pub struct PortInputFuture<'a, T> {
    pin: &'a GpioPin<Input<T>>,
    polarity: PortInputPolarity,
}

impl<'a, T> Drop for PortInputFuture<'a, T> {
    fn drop(&mut self) {
        pin_conf(&self.pin).modify(|_, w| w.sense().disabled());
        PORT_SIGNALS[pin_num(&self.pin)].reset();
    }
}

impl<'a, T> Future for PortInputFuture<'a, T> {
    type Output = ();

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        pin_conf(&self.pin).modify(|_, w| match self.polarity {
            PortInputPolarity::Low => w.sense().low(),
            PortInputPolarity::High => w.sense().high(),
        });
        PORT_SIGNALS[pin_num(&self.pin)].poll_wait(cx)
    }
}
