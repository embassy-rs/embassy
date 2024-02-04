//! GPIO task/event (GPIOTE) driver.

use core::convert::Infallible;
use core::future::{poll_fn, Future};
use core::task::{Context, Poll};

use embassy_hal_internal::{impl_peripheral, into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::sealed::Pin as _;
use crate::gpio::{AnyPin, Flex, Input, Output, Pin as GpioPin};
use crate::interrupt::InterruptExt;
use crate::ppi::{Event, Task};
use crate::{interrupt, pac, peripherals};

#[cfg(feature = "nrf51")]
/// Amount of GPIOTE channels in the chip.
const CHANNEL_COUNT: usize = 4;
#[cfg(not(feature = "_nrf51"))]
/// Amount of GPIOTE channels in the chip.
const CHANNEL_COUNT: usize = 8;

#[cfg(any(feature = "nrf52833", feature = "nrf52840"))]
const PIN_COUNT: usize = 48;
#[cfg(not(any(feature = "nrf52833", feature = "nrf52840")))]
const PIN_COUNT: usize = 32;

#[allow(clippy::declare_interior_mutable_const)]
const NEW_AW: AtomicWaker = AtomicWaker::new();
static CHANNEL_WAKERS: [AtomicWaker; CHANNEL_COUNT] = [NEW_AW; CHANNEL_COUNT];
static PORT_WAKERS: [AtomicWaker; PIN_COUNT] = [NEW_AW; PIN_COUNT];

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
    // no latched GPIO detect in nrf51.
    #[cfg(not(feature = "_nrf51"))]
    {
        #[cfg(any(feature = "nrf52833", feature = "nrf52840"))]
        let ports = unsafe { &[&*pac::P0::ptr(), &*pac::P1::ptr()] };
        #[cfg(not(any(feature = "_nrf51", feature = "nrf52833", feature = "nrf52840")))]
        let ports = unsafe { &[&*pac::P0::ptr()] };

        for &p in ports {
            // Enable latched detection
            p.detectmode.write(|w| w.detectmode().ldetect());
            // Clear latch
            p.latch.write(|w| unsafe { w.bits(0xFFFFFFFF) })
        }
    }

    // Enable interrupts
    #[cfg(any(feature = "nrf5340-app-s", feature = "nrf9160-s"))]
    let irq = interrupt::GPIOTE0;
    #[cfg(any(feature = "nrf5340-app-ns", feature = "nrf9160-ns"))]
    let irq = interrupt::GPIOTE1;
    #[cfg(any(feature = "_nrf51", feature = "_nrf52", feature = "nrf5340-net"))]
    let irq = interrupt::GPIOTE;

    irq.unpend();
    irq.set_priority(irq_prio);
    unsafe { irq.enable() };

    let g = regs();
    g.intenset.write(|w| w.port().set());
}

#[cfg(any(feature = "nrf5340-app-s", feature = "nrf9160-s"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE0() {
    unsafe { handle_gpiote_interrupt() };
}

#[cfg(any(feature = "nrf5340-app-ns", feature = "nrf9160-ns"))]
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
        if g.events_in[i].read().bits() != 0 {
            g.intenclr.write(|w| w.bits(1 << i));
            CHANNEL_WAKERS[i].wake();
        }
    }

    if g.events_port.read().bits() != 0 {
        g.events_port.write(|w| w);

        #[cfg(any(feature = "nrf52833", feature = "nrf52840"))]
        let ports = &[&*pac::P0::ptr(), &*pac::P1::ptr()];
        #[cfg(not(any(feature = "_nrf51", feature = "nrf52833", feature = "nrf52840")))]
        let ports = &[&*pac::P0::ptr()];
        #[cfg(feature = "_nrf51")]
        let ports = unsafe { &[&*pac::GPIO::ptr()] };

        #[cfg(feature = "_nrf51")]
        for (port, &p) in ports.iter().enumerate() {
            let inp = p.in_.read().bits();
            for pin in 0..32 {
                let fired = match p.pin_cnf[pin as usize].read().sense().variant() {
                    Some(pac::gpio::pin_cnf::SENSE_A::HIGH) => inp & (1 << pin) != 0,
                    Some(pac::gpio::pin_cnf::SENSE_A::LOW) => inp & (1 << pin) == 0,
                    _ => false,
                };

                if fired {
                    PORT_WAKERS[port * 32 + pin as usize].wake();
                    p.pin_cnf[pin as usize].modify(|_, w| w.sense().disabled());
                }
            }
        }

        #[cfg(not(feature = "_nrf51"))]
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
pub struct InputChannel<'d> {
    ch: PeripheralRef<'d, AnyChannel>,
    pin: Input<'d>,
}

impl<'d> Drop for InputChannel<'d> {
    fn drop(&mut self) {
        let g = regs();
        let num = self.ch.number();
        g.config[num].write(|w| w.mode().disabled());
        g.intenclr.write(|w| unsafe { w.bits(1 << num) });
    }
}

impl<'d> InputChannel<'d> {
    /// Create a new GPIOTE input channel driver.
    pub fn new(ch: impl Peripheral<P = impl Channel> + 'd, pin: Input<'d>, polarity: InputChannelPolarity) -> Self {
        into_ref!(ch);

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

        InputChannel { ch: ch.map_into(), pin }
    }

    /// Asynchronously wait for an event in this channel.
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
    pub fn event_in(&self) -> Event<'d> {
        let g = regs();
        Event::from_reg(&g.events_in[self.ch.number()])
    }
}

/// GPIOTE channel driver in output mode
pub struct OutputChannel<'d> {
    ch: PeripheralRef<'d, AnyChannel>,
    _pin: Output<'d>,
}

impl<'d> Drop for OutputChannel<'d> {
    fn drop(&mut self) {
        let g = regs();
        let num = self.ch.number();
        g.config[num].write(|w| w.mode().disabled());
        g.intenclr.write(|w| unsafe { w.bits(1 << num) });
    }
}

impl<'d> OutputChannel<'d> {
    /// Create a new GPIOTE output channel driver.
    pub fn new(ch: impl Peripheral<P = impl Channel> + 'd, pin: Output<'d>, polarity: OutputChannelPolarity) -> Self {
        into_ref!(ch);
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

        OutputChannel {
            ch: ch.map_into(),
            _pin: pin,
        }
    }

    /// Triggers the OUT task (does the action as configured with task_out_polarity, defaults to Toggle).
    pub fn out(&self) {
        let g = regs();
        g.tasks_out[self.ch.number()].write(|w| unsafe { w.bits(1) });
    }

    /// Triggers the SET task (set associated pin high).
    #[cfg(not(feature = "nrf51"))]
    pub fn set(&self) {
        let g = regs();
        g.tasks_set[self.ch.number()].write(|w| unsafe { w.bits(1) });
    }

    /// Triggers the CLEAR task (set associated pin low).
    #[cfg(not(feature = "nrf51"))]
    pub fn clear(&self) {
        let g = regs();
        g.tasks_clr[self.ch.number()].write(|w| unsafe { w.bits(1) });
    }

    /// Returns the OUT task, for use with PPI.
    pub fn task_out(&self) -> Task<'d> {
        let g = regs();
        Task::from_reg(&g.tasks_out[self.ch.number()])
    }

    /// Returns the CLR task, for use with PPI.
    #[cfg(not(feature = "nrf51"))]
    pub fn task_clr(&self) -> Task<'d> {
        let g = regs();
        Task::from_reg(&g.tasks_clr[self.ch.number()])
    }

    /// Returns the SET task, for use with PPI.
    #[cfg(not(feature = "nrf51"))]
    pub fn task_set(&self) -> Task<'d> {
        let g = regs();
        Task::from_reg(&g.tasks_set[self.ch.number()])
    }
}

// =======================

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub(crate) struct PortInputFuture<'a> {
    pin: PeripheralRef<'a, AnyPin>,
}

impl<'a> PortInputFuture<'a> {
    fn new(pin: impl Peripheral<P = impl GpioPin> + 'a) -> Self {
        Self {
            pin: pin.into_ref().map_into(),
        }
    }
}

impl<'a> Unpin for PortInputFuture<'a> {}

impl<'a> Drop for PortInputFuture<'a> {
    fn drop(&mut self) {
        self.pin.conf().modify(|_, w| w.sense().disabled());
    }
}

impl<'a> Future for PortInputFuture<'a> {
    type Output = ();

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        PORT_WAKERS[self.pin.pin_port() as usize].register(cx.waker());

        if self.pin.conf().read().sense().is_disabled() {
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
        self.pin.conf().modify(|_, w| w.sense().high());
        PortInputFuture::new(&mut self.pin).await
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    pub async fn wait_for_low(&mut self) {
        self.pin.conf().modify(|_, w| w.sense().low());
        PortInputFuture::new(&mut self.pin).await
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
            self.pin.conf().modify(|_, w| w.sense().low());
        } else {
            self.pin.conf().modify(|_, w| w.sense().high());
        }
        PortInputFuture::new(&mut self.pin).await
    }
}

// =======================

mod sealed {
    pub trait Channel {}
}

/// GPIOTE channel trait.
///
/// Implemented by all GPIOTE channels.
pub trait Channel: sealed::Channel + Into<AnyChannel> + Sized + 'static {
    /// Get the channel number.
    fn number(&self) -> usize;

    /// Convert this channel to a type-erased `AnyChannel`.
    ///
    /// This allows using several channels in situations that might require
    /// them to be the same type, like putting them in an array.
    fn degrade(self) -> AnyChannel {
        AnyChannel {
            number: self.number() as u8,
        }
    }
}

/// Type-erased channel.
///
/// Obtained by calling `Channel::degrade`.
///
/// This allows using several channels in situations that might require
/// them to be the same type, like putting them in an array.
pub struct AnyChannel {
    number: u8,
}
impl_peripheral!(AnyChannel);
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

        impl From<peripherals::$type> for AnyChannel {
            fn from(val: peripherals::$type) -> Self {
                Channel::degrade(val)
            }
        }
    };
}

impl_channel!(GPIOTE_CH0, 0);
impl_channel!(GPIOTE_CH1, 1);
impl_channel!(GPIOTE_CH2, 2);
impl_channel!(GPIOTE_CH3, 3);
#[cfg(not(feature = "nrf51"))]
impl_channel!(GPIOTE_CH4, 4);
#[cfg(not(feature = "nrf51"))]
impl_channel!(GPIOTE_CH5, 5);
#[cfg(not(feature = "nrf51"))]
impl_channel!(GPIOTE_CH6, 6);
#[cfg(not(feature = "nrf51"))]
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
