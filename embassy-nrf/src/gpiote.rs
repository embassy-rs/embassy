//! GPIO task/event (GPIOTE) driver.

use core::convert::Infallible;
use core::future::{poll_fn, Future};
use core::task::{Context, Poll};

use embassy_hal_internal::{impl_peripheral, Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AnyPin, Flex, Input, Output, Pin as GpioPin, SealedPin as _};
use crate::interrupt::InterruptExt;
#[cfg(not(feature = "_nrf51"))]
use crate::pac::gpio::vals::Detectmode;
use crate::pac::gpio::vals::Sense;
use crate::pac::gpiote::vals::{Mode, Outinit, Polarity};
use crate::ppi::{Event, Task};
use crate::{interrupt, pac, peripherals};

#[cfg(feature = "_nrf51")]
/// Amount of GPIOTE channels in the chip.
const CHANNEL_COUNT: usize = 4;
#[cfg(not(feature = "_nrf51"))]
/// Amount of GPIOTE channels in the chip.
const CHANNEL_COUNT: usize = 8;

#[cfg(any(feature = "nrf52833", feature = "nrf52840", feature = "_nrf5340"))]
const PIN_COUNT: usize = 48;
#[cfg(not(any(feature = "nrf52833", feature = "nrf52840", feature = "_nrf5340")))]
const PIN_COUNT: usize = 32;

#[allow(clippy::declare_interior_mutable_const)]
static CHANNEL_WAKERS: [AtomicWaker; CHANNEL_COUNT] = [const { AtomicWaker::new() }; CHANNEL_COUNT];
static PORT_WAKERS: [AtomicWaker; PIN_COUNT] = [const { AtomicWaker::new() }; PIN_COUNT];

/// Polarity for listening to events for GPIOTE input channels.
pub enum InputChannelPolarity {
    /// Don't listen for any pin changes.
    None,
    /// Listen for high to low changes.
    HiToLo,
    /// Listen for low to high changes.
    LoToHi,
    /// Listen for any change, either low to high or high to low.
    Toggle,
}

/// Polarity of the OUT task operation for GPIOTE output channels.
pub enum OutputChannelPolarity {
    /// Set the pin high.
    Set,
    /// Set the pin low.
    Clear,
    /// Toggle the pin.
    Toggle,
}

fn regs() -> pac::gpiote::Gpiote {
    cfg_if::cfg_if! {
        if #[cfg(any(feature="nrf5340-app-s", feature="nrf9160-s", feature="nrf9120-s"))] {
            pac::GPIOTE0
        } else if #[cfg(any(feature="nrf5340-app-ns", feature="nrf9160-ns", feature="nrf9120-ns"))] {
            pac::GPIOTE1
        } else {
            pac::GPIOTE
        }
    }
}

pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    // no latched GPIO detect in nrf51.
    #[cfg(not(feature = "_nrf51"))]
    {
        #[cfg(any(feature = "nrf52833", feature = "nrf52840", feature = "_nrf5340"))]
        let ports = &[pac::P0, pac::P1];
        #[cfg(not(any(feature = "_nrf51", feature = "nrf52833", feature = "nrf52840", feature = "_nrf5340")))]
        let ports = &[pac::P0];

        for &p in ports {
            // Enable latched detection
            p.detectmode().write(|w| w.set_detectmode(Detectmode::LDETECT));
            // Clear latch
            p.latch().write(|w| w.0 = 0xFFFFFFFF)
        }
    }

    // Enable interrupts
    #[cfg(any(feature = "nrf5340-app-s", feature = "nrf9160-s", feature = "nrf9120-s"))]
    let irq = interrupt::GPIOTE0;
    #[cfg(any(feature = "nrf5340-app-ns", feature = "nrf9160-ns", feature = "nrf9120-ns"))]
    let irq = interrupt::GPIOTE1;
    #[cfg(any(feature = "_nrf51", feature = "_nrf52", feature = "nrf5340-net"))]
    let irq = interrupt::GPIOTE;

    irq.unpend();
    irq.set_priority(irq_prio);
    unsafe { irq.enable() };

    let g = regs();
    g.intenset().write(|w| w.set_port(true));
}

#[cfg(any(feature = "nrf5340-app-s", feature = "nrf9160-s", feature = "nrf9120-s"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE0() {
    unsafe { handle_gpiote_interrupt() };
}

#[cfg(any(feature = "nrf5340-app-ns", feature = "nrf9160-ns", feature = "nrf9120-ns"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE1() {
    unsafe { handle_gpiote_interrupt() };
}

#[cfg(any(feature = "_nrf51", feature = "_nrf52", feature = "nrf5340-net"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE() {
    unsafe { handle_gpiote_interrupt() };
}

unsafe fn handle_gpiote_interrupt() {
    let g = regs();

    for i in 0..CHANNEL_COUNT {
        if g.events_in(i).read() != 0 {
            g.intenclr().write(|w| w.0 = 1 << i);
            CHANNEL_WAKERS[i].wake();
        }
    }

    if g.events_port().read() != 0 {
        g.events_port().write_value(0);

        #[cfg(any(feature = "nrf52833", feature = "nrf52840", feature = "_nrf5340"))]
        let ports = &[pac::P0, pac::P1];
        #[cfg(not(any(feature = "_nrf51", feature = "nrf52833", feature = "nrf52840", feature = "_nrf5340")))]
        let ports = &[pac::P0];
        #[cfg(feature = "_nrf51")]
        let ports = &[pac::GPIO];

        #[cfg(feature = "_nrf51")]
        for (port, &p) in ports.iter().enumerate() {
            let inp = p.in_().read();
            for pin in 0..32 {
                let fired = match p.pin_cnf(pin as usize).read().sense() {
                    Sense::HIGH => inp.pin(pin),
                    Sense::LOW => !inp.pin(pin),
                    _ => false,
                };

                if fired {
                    PORT_WAKERS[port * 32 + pin as usize].wake();
                    p.pin_cnf(pin as usize).modify(|w| w.set_sense(Sense::DISABLED));
                }
            }
        }

        #[cfg(not(feature = "_nrf51"))]
        for (port, &p) in ports.iter().enumerate() {
            let bits = p.latch().read().0;
            for pin in BitIter(bits) {
                p.pin_cnf(pin as usize).modify(|w| w.set_sense(Sense::DISABLED));
                PORT_WAKERS[port * 32 + pin as usize].wake();
            }
            p.latch().write(|w| w.0 = bits);
        }
    }
}

#[cfg(not(feature = "_nrf51"))]
struct BitIter(u32);

#[cfg(not(feature = "_nrf51"))]
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
pub struct InputChannel<'d> {
    ch: Peri<'d, AnyChannel>,
    pin: Input<'d>,
}

impl<'d> Drop for InputChannel<'d> {
    fn drop(&mut self) {
        let g = regs();
        let num = self.ch.number();
        g.config(num).write(|w| w.set_mode(Mode::DISABLED));
        g.intenclr().write(|w| w.0 = 1 << num);
    }
}

impl<'d> InputChannel<'d> {
    /// Create a new GPIOTE input channel driver.
    pub fn new(ch: Peri<'d, impl Channel>, pin: Input<'d>, polarity: InputChannelPolarity) -> Self {
        let g = regs();
        let num = ch.number();

        g.config(num).write(|w| {
            w.set_mode(Mode::EVENT);
            match polarity {
                InputChannelPolarity::HiToLo => w.set_polarity(Polarity::HI_TO_LO),
                InputChannelPolarity::LoToHi => w.set_polarity(Polarity::LO_TO_HI),
                InputChannelPolarity::None => w.set_polarity(Polarity::NONE),
                InputChannelPolarity::Toggle => w.set_polarity(Polarity::TOGGLE),
            };
            #[cfg(any(feature = "nrf52833", feature = "nrf52840", feature = "_nrf5340"))]
            w.set_port(match pin.pin.pin.port() {
                crate::gpio::Port::Port0 => false,
                crate::gpio::Port::Port1 => true,
            });
            w.set_psel(pin.pin.pin.pin());
        });

        g.events_in(num).write_value(0);

        InputChannel { ch: ch.into(), pin }
    }

    /// Asynchronously wait for an event in this channel.
    pub async fn wait(&self) {
        let g = regs();
        let num = self.ch.number();

        // Enable interrupt
        g.events_in(num).write_value(0);
        g.intenset().write(|w| w.0 = 1 << num);

        poll_fn(|cx| {
            CHANNEL_WAKERS[num].register(cx.waker());

            if g.events_in(num).read() != 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    /// Returns the IN event, for use with PPI.
    pub fn event_in(&self) -> Event<'d> {
        let g = regs();
        Event::from_reg(g.events_in(self.ch.number()))
    }
}

/// GPIOTE channel driver in output mode
pub struct OutputChannel<'d> {
    ch: Peri<'d, AnyChannel>,
    _pin: Output<'d>,
}

impl<'d> Drop for OutputChannel<'d> {
    fn drop(&mut self) {
        let g = regs();
        let num = self.ch.number();
        g.config(num).write(|w| w.set_mode(Mode::DISABLED));
        g.intenclr().write(|w| w.0 = 1 << num);
    }
}

impl<'d> OutputChannel<'d> {
    /// Create a new GPIOTE output channel driver.
    pub fn new(ch: Peri<'d, impl Channel>, pin: Output<'d>, polarity: OutputChannelPolarity) -> Self {
        let g = regs();
        let num = ch.number();

        g.config(num).write(|w| {
            w.set_mode(Mode::TASK);
            match pin.is_set_high() {
                true => w.set_outinit(Outinit::HIGH),
                false => w.set_outinit(Outinit::LOW),
            };
            match polarity {
                OutputChannelPolarity::Set => w.set_polarity(Polarity::HI_TO_LO),
                OutputChannelPolarity::Clear => w.set_polarity(Polarity::LO_TO_HI),
                OutputChannelPolarity::Toggle => w.set_polarity(Polarity::TOGGLE),
            };
            #[cfg(any(feature = "nrf52833", feature = "nrf52840", feature = "_nrf5340"))]
            w.set_port(match pin.pin.pin.port() {
                crate::gpio::Port::Port0 => false,
                crate::gpio::Port::Port1 => true,
            });
            w.set_psel(pin.pin.pin.pin());
        });

        OutputChannel {
            ch: ch.into(),
            _pin: pin,
        }
    }

    /// Triggers the OUT task (does the action as configured with task_out_polarity, defaults to Toggle).
    pub fn out(&self) {
        let g = regs();
        g.tasks_out(self.ch.number()).write_value(1);
    }

    /// Triggers the SET task (set associated pin high).
    #[cfg(not(feature = "_nrf51"))]
    pub fn set(&self) {
        let g = regs();
        g.tasks_set(self.ch.number()).write_value(1);
    }

    /// Triggers the CLEAR task (set associated pin low).
    #[cfg(not(feature = "_nrf51"))]
    pub fn clear(&self) {
        let g = regs();
        g.tasks_clr(self.ch.number()).write_value(1);
    }

    /// Returns the OUT task, for use with PPI.
    pub fn task_out(&self) -> Task<'d> {
        let g = regs();
        Task::from_reg(g.tasks_out(self.ch.number()))
    }

    /// Returns the CLR task, for use with PPI.
    #[cfg(not(feature = "_nrf51"))]
    pub fn task_clr(&self) -> Task<'d> {
        let g = regs();
        Task::from_reg(g.tasks_clr(self.ch.number()))
    }

    /// Returns the SET task, for use with PPI.
    #[cfg(not(feature = "_nrf51"))]
    pub fn task_set(&self) -> Task<'d> {
        let g = regs();
        Task::from_reg(g.tasks_set(self.ch.number()))
    }
}

// =======================

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub(crate) struct PortInputFuture<'a> {
    pin: Peri<'a, AnyPin>,
}

impl<'a> PortInputFuture<'a> {
    fn new(pin: Peri<'a, impl GpioPin>) -> Self {
        Self { pin: pin.into() }
    }
}

impl<'a> Unpin for PortInputFuture<'a> {}

impl<'a> Drop for PortInputFuture<'a> {
    fn drop(&mut self) {
        self.pin.conf().modify(|w| w.set_sense(Sense::DISABLED));
    }
}

impl<'a> Future for PortInputFuture<'a> {
    type Output = ();

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        PORT_WAKERS[self.pin.pin_port() as usize].register(cx.waker());

        if self.pin.conf().read().sense() == Sense::DISABLED {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl<'d> Input<'d> {
    /// Wait until the pin is high. If it is already high, return immediately.
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await
    }

    /// Wait for the pin to undergo a transition from low to high.
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await
    }

    /// Wait for the pin to undergo a transition from high to low.
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await
    }
}

impl<'d> Flex<'d> {
    /// Wait until the pin is high. If it is already high, return immediately.
    pub async fn wait_for_high(&mut self) {
        self.pin.conf().modify(|w| w.set_sense(Sense::HIGH));
        PortInputFuture::new(self.pin.reborrow()).await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    pub async fn wait_for_low(&mut self) {
        self.pin.conf().modify(|w| w.set_sense(Sense::LOW));
        PortInputFuture::new(self.pin.reborrow()).await
    }

    /// Wait for the pin to undergo a transition from low to high.
    pub async fn wait_for_rising_edge(&mut self) {
        self.wait_for_low().await;
        self.wait_for_high().await;
    }

    /// Wait for the pin to undergo a transition from high to low.
    pub async fn wait_for_falling_edge(&mut self) {
        self.wait_for_high().await;
        self.wait_for_low().await;
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    pub async fn wait_for_any_edge(&mut self) {
        if self.is_high() {
            self.pin.conf().modify(|w| w.set_sense(Sense::LOW));
        } else {
            self.pin.conf().modify(|w| w.set_sense(Sense::HIGH));
        }
        PortInputFuture::new(self.pin.reborrow()).await
    }
}

// =======================

trait SealedChannel {}

/// GPIOTE channel trait.
///
/// Implemented by all GPIOTE channels.
#[allow(private_bounds)]
pub trait Channel: PeripheralType + SealedChannel + Into<AnyChannel> + Sized + 'static {
    /// Get the channel number.
    fn number(&self) -> usize;
}

/// Type-erased channel.
///
/// Obtained by calling `Channel::into()`.
///
/// This allows using several channels in situations that might require
/// them to be the same type, like putting them in an array.
pub struct AnyChannel {
    number: u8,
}
impl_peripheral!(AnyChannel);
impl SealedChannel for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> usize {
        self.number as usize
    }
}

macro_rules! impl_channel {
    ($type:ident, $number:expr) => {
        impl SealedChannel for peripherals::$type {}
        impl Channel for peripherals::$type {
            fn number(&self) -> usize {
                $number as usize
            }
        }

        impl From<peripherals::$type> for AnyChannel {
            fn from(val: peripherals::$type) -> Self {
                Self {
                    number: val.number() as u8,
                }
            }
        }
    };
}

impl_channel!(GPIOTE_CH0, 0);
impl_channel!(GPIOTE_CH1, 1);
impl_channel!(GPIOTE_CH2, 2);
impl_channel!(GPIOTE_CH3, 3);
#[cfg(not(feature = "_nrf51"))]
impl_channel!(GPIOTE_CH4, 4);
#[cfg(not(feature = "_nrf51"))]
impl_channel!(GPIOTE_CH5, 5);
#[cfg(not(feature = "_nrf51"))]
impl_channel!(GPIOTE_CH6, 6);
#[cfg(not(feature = "_nrf51"))]
impl_channel!(GPIOTE_CH7, 7);

// ====================

mod eh02 {
    use super::*;

    impl<'d> embedded_hal_02::digital::v2::InputPin for InputChannel<'d> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.pin.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.pin.is_low())
        }
    }
}

impl<'d> embedded_hal_1::digital::ErrorType for InputChannel<'d> {
    type Error = Infallible;
}

impl<'d> embedded_hal_1::digital::InputPin for InputChannel<'d> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.pin.is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.pin.is_low())
    }
}

impl<'d> embedded_hal_async::digital::Wait for Input<'d> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_high().await)
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_low().await)
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_rising_edge().await)
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_falling_edge().await)
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_any_edge().await)
    }
}

impl<'d> embedded_hal_async::digital::Wait for Flex<'d> {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_high().await)
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_low().await)
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_rising_edge().await)
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_falling_edge().await)
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        Ok(self.wait_for_any_edge().await)
    }
}
