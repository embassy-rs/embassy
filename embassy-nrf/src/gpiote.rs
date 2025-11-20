//! GPIO task/event (GPIOTE) driver.
#![macro_use]

use core::convert::Infallible;
use core::future::{Future, poll_fn};
use core::task::{Context, Poll};

use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AnyPin, Flex, Input, Level, Output, OutputDrive, Pin as GpioPin, Pull, SealedPin as _};
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
#[cfg(not(any(feature = "_nrf51", feature = "_nrf54l")))]
/// Amount of GPIOTE channels in the chip.
const CHANNEL_COUNT: usize = 8;
#[cfg(any(feature = "_nrf54l"))]
/// Amount of GPIOTE channels in the chip.
const CHANNEL_COUNT: usize = 12;
/// Max channels per port
const CHANNELS_PER_PORT: usize = 8;

#[cfg(any(
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "_nrf5340",
    feature = "_nrf54l"
))]
const PIN_COUNT: usize = 48;
#[cfg(not(any(
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "_nrf5340",
    feature = "_nrf54l"
)))]
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
            #[cfg(all(feature = "_s", not(feature = "_nrf54l")))]
            p.detectmode_sec().write(|w| w.set_detectmode(Detectmode::LDETECT));
            #[cfg(any(not(feature = "_s"), all(feature = "_s", feature = "_nrf54l")))]
            p.detectmode().write(|w| w.set_detectmode(Detectmode::LDETECT));
            // Clear latch
            p.latch().write(|w| w.0 = 0xFFFFFFFF)
        }
    }

    // Enable interrupts
    #[cfg(any(feature = "nrf5340-app-s", feature = "nrf9160-s", feature = "nrf9120-s"))]
    let irqs = &[(pac::GPIOTE0, interrupt::GPIOTE0)];
    #[cfg(any(feature = "nrf5340-app-ns", feature = "nrf9160-ns", feature = "nrf9120-ns"))]
    let irqs = &[(pac::GPIOTE1, interrupt::GPIOTE1)];
    #[cfg(any(feature = "_nrf51", feature = "_nrf52", feature = "nrf5340-net"))]
    let irqs = &[(pac::GPIOTE, interrupt::GPIOTE)];
    #[cfg(any(feature = "_nrf54l"))]
    let irqs = &[
        #[cfg(feature = "_s")]
        (pac::GPIOTE20, interrupt::GPIOTE20_0),
        #[cfg(feature = "_s")]
        (pac::GPIOTE30, interrupt::GPIOTE30_0),
        #[cfg(feature = "_ns")]
        (pac::GPIOTE20, interrupt::GPIOTE20_1),
        #[cfg(feature = "_ns")]
        (pac::GPIOTE30, interrupt::GPIOTE30_1),
    ];

    for (inst, irq) in irqs {
        irq.unpend();
        irq.set_priority(irq_prio);
        unsafe { irq.enable() };

        let g = inst;
        #[cfg(not(feature = "_nrf54l"))]
        g.intenset(INTNUM).write(|w| w.set_port(true));

        #[cfg(all(feature = "_nrf54l", feature = "_ns"))]
        g.intenset(INTNUM).write(|w| w.set_port0nonsecure(true));

        #[cfg(all(feature = "_nrf54l", feature = "_s"))]
        g.intenset(INTNUM).write(|w| w.set_port0secure(true));
    }
}

#[cfg(all(feature = "_nrf54l", feature = "_ns"))]
const INTNUM: usize = 1;

#[cfg(any(not(feature = "_nrf54l"), feature = "_s"))]
const INTNUM: usize = 0;

#[cfg(any(feature = "nrf5340-app-s", feature = "nrf9160-s", feature = "nrf9120-s"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE0() {
    unsafe { handle_gpiote_interrupt(pac::GPIOTE0) };
}

#[cfg(any(feature = "nrf5340-app-ns", feature = "nrf9160-ns", feature = "nrf9120-ns"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE1() {
    unsafe { handle_gpiote_interrupt(pac::GPIOTE1) };
}

#[cfg(any(feature = "_nrf51", feature = "_nrf52", feature = "nrf5340-net"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE() {
    unsafe { handle_gpiote_interrupt(pac::GPIOTE) };
}

#[cfg(all(feature = "_nrf54l", feature = "_s"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE20_0() {
    unsafe { handle_gpiote_interrupt(pac::GPIOTE20) };
}

#[cfg(all(feature = "_nrf54l", feature = "_s"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE30_0() {
    unsafe { handle_gpiote_interrupt(pac::GPIOTE30) };
}

#[cfg(all(feature = "_nrf54l", feature = "_ns"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE20_1() {
    unsafe { handle_gpiote_interrupt(pac::GPIOTE20) };
}

#[cfg(all(feature = "_nrf54l", feature = "_ns"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE30_1() {
    unsafe { handle_gpiote_interrupt(pac::GPIOTE30) };
}

unsafe fn handle_gpiote_interrupt(g: pac::gpiote::Gpiote) {
    for c in 0..CHANNEL_COUNT {
        let i = c % CHANNELS_PER_PORT;
        if g.events_in(i).read() != 0 {
            g.intenclr(INTNUM).write(|w| w.0 = 1 << i);
            CHANNEL_WAKERS[c].wake();
        }
    }

    #[cfg(not(feature = "_nrf54l"))]
    let eport = g.events_port(0);

    #[cfg(all(feature = "_nrf54l", feature = "_ns"))]
    let eport = g.events_port(0).nonsecure();

    #[cfg(all(feature = "_nrf54l", feature = "_s"))]
    let eport = g.events_port(0).secure();

    if eport.read() != 0 {
        eport.write_value(0);

        #[cfg(any(
            feature = "nrf52833",
            feature = "nrf52840",
            feature = "_nrf5340",
            feature = "_nrf54l"
        ))]
        let ports = &[pac::P0, pac::P1];
        #[cfg(not(any(
            feature = "_nrf51",
            feature = "nrf52833",
            feature = "nrf52840",
            feature = "_nrf5340",
            feature = "_nrf54l"
        )))]
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

impl InputChannel<'static> {
    /// Persist the channel's configuration for the rest of the program's lifetime. This method
    /// should be preferred over [`core::mem::forget()`] because the `'static` bound prevents
    /// accidental reuse of the underlying peripheral.
    pub fn persist(self) {
        core::mem::forget(self);
    }
}

impl<'d> Drop for InputChannel<'d> {
    fn drop(&mut self) {
        let g = self.ch.regs();
        let num = self.ch.number();
        g.config(num).write(|w| w.set_mode(Mode::DISABLED));
        g.intenclr(INTNUM).write(|w| w.0 = 1 << num);
    }
}

impl<'d> InputChannel<'d> {
    /// Create a new GPIOTE input channel driver.
    #[cfg(feature = "_nrf54l")]
    pub fn new<C: Channel, T: GpiotePin<Instance = C::Instance>>(
        ch: Peri<'d, C>,
        pin: Peri<'d, T>,
        pull: Pull,
        polarity: InputChannelPolarity,
    ) -> Self {
        let pin = Input::new(pin, pull);
        let ch = ch.into();
        Self::new_inner(ch, pin, polarity)
    }

    /// Create a new GPIOTE output channel driver.
    #[cfg(not(feature = "_nrf54l"))]
    pub fn new<C: Channel, T: GpioPin>(
        ch: Peri<'d, C>,
        pin: Peri<'d, T>,
        pull: Pull,
        polarity: InputChannelPolarity,
    ) -> Self {
        let pin = Input::new(pin, pull);
        let ch = ch.into();
        Self::new_inner(ch, pin, polarity)
    }

    fn new_inner(ch: Peri<'d, AnyChannel>, pin: Input<'d>, polarity: InputChannelPolarity) -> Self {
        let g = ch.regs();
        let num = ch.number();
        g.config(num).write(|w| {
            w.set_mode(Mode::EVENT);
            match polarity {
                InputChannelPolarity::HiToLo => w.set_polarity(Polarity::HI_TO_LO),
                InputChannelPolarity::LoToHi => w.set_polarity(Polarity::LO_TO_HI),
                InputChannelPolarity::None => w.set_polarity(Polarity::NONE),
                InputChannelPolarity::Toggle => w.set_polarity(Polarity::TOGGLE),
            };
            #[cfg(any(feature = "nrf52833", feature = "nrf52840", feature = "_nrf5340",))]
            w.set_port(match pin.pin.pin.port() {
                crate::gpio::Port::Port0 => false,
                crate::gpio::Port::Port1 => true,
            });
            #[cfg(any(feature = "_nrf54l"))]
            w.set_port(match pin.pin.pin.port() {
                crate::gpio::Port::Port0 => 0,
                crate::gpio::Port::Port1 => 1,
                crate::gpio::Port::Port2 => 2,
            });
            w.set_psel(pin.pin.pin.pin());
        });

        g.events_in(num).write_value(0);

        InputChannel { ch, pin }
    }

    /// Asynchronously wait for an event in this channel.
    ///
    /// It is possible to call this function and await the returned future later.
    /// If an even occurs in the mean time, the future will immediately report ready.
    pub fn wait(&mut self) -> impl Future<Output = ()> {
        // NOTE: This is `-> impl Future` and not an `async fn` on purpose.
        // Otherwise, events will only be detected starting at the first poll of the returned future.
        Self::wait_internal(&mut self.ch)
    }

    /// Asynchronously wait for the pin to become high.
    ///
    /// The channel must be configured with [`InputChannelPolarity::LoToHi`] or [`InputChannelPolarity::Toggle`].
    /// If the channel is not configured to detect rising edges, it is unspecified when the returned future completes.
    ///
    /// It is possible to call this function and await the returned future later.
    /// If an even occurs in the mean time, the future will immediately report ready.
    pub fn wait_for_high(&mut self) -> impl Future<Output = ()> {
        // NOTE: This is `-> impl Future` and not an `async fn` on purpose.
        // Otherwise, events will only be detected starting at the first poll of the returned future.

        // Subscribe to the event before checking the pin level.
        let wait = Self::wait_internal(&mut self.ch);
        let pin = &self.pin;
        async move {
            if pin.is_high() {
                return;
            }
            wait.await;
        }
    }

    /// Asynchronously wait for the pin to become low.
    ///
    /// The channel must be configured with [`InputChannelPolarity::HiToLo`] or [`InputChannelPolarity::Toggle`].
    /// If the channel is not configured to detect falling edges, it is unspecified when the returned future completes.
    ///
    /// It is possible to call this function and await the returned future later.
    /// If an even occurs in the mean time, the future will immediately report ready.
    pub fn wait_for_low(&mut self) -> impl Future<Output = ()> {
        // NOTE: This is `-> impl Future` and not an `async fn` on purpose.
        // Otherwise, events will only be detected starting at the first poll of the returned future.

        // Subscribe to the event before checking the pin level.
        let wait = Self::wait_internal(&mut self.ch);
        let pin = &self.pin;
        async move {
            if pin.is_low() {
                return;
            }
            wait.await;
        }
    }

    /// Internal implementation for `wait()` and friends.
    fn wait_internal(channel: &mut Peri<'_, AnyChannel>) -> impl Future<Output = ()> {
        // NOTE: This is `-> impl Future` and not an `async fn` on purpose.
        // Otherwise, events will only be detected starting at the first poll of the returned future.

        let g = channel.regs();
        let num = channel.number();
        let waker = channel.waker();

        // Enable interrupt
        g.events_in(num).write_value(0);
        g.intenset(INTNUM).write(|w| w.0 = 1 << num);

        poll_fn(move |cx| {
            CHANNEL_WAKERS[waker].register(cx.waker());

            if g.events_in(num).read() != 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
    }

    /// Get the associated input pin.
    pub fn pin(&self) -> &Input<'_> {
        &self.pin
    }

    /// Returns the IN event, for use with PPI.
    pub fn event_in(&self) -> Event<'d> {
        let g = self.ch.regs();
        Event::from_reg(g.events_in(self.ch.number()))
    }
}

/// GPIOTE channel driver in output mode
pub struct OutputChannel<'d> {
    ch: Peri<'d, AnyChannel>,
    _pin: Output<'d>,
}

impl OutputChannel<'static> {
    /// Persist the channel's configuration for the rest of the program's lifetime. This method
    /// should be preferred over [`core::mem::forget()`] because the `'static` bound prevents
    /// accidental reuse of the underlying peripheral.
    pub fn persist(self) {
        core::mem::forget(self);
    }
}

impl<'d> Drop for OutputChannel<'d> {
    fn drop(&mut self) {
        let g = self.ch.regs();
        let num = self.ch.number();
        g.config(num).write(|w| w.set_mode(Mode::DISABLED));
        g.intenclr(INTNUM).write(|w| w.0 = 1 << num);
    }
}

impl<'d> OutputChannel<'d> {
    /// Create a new GPIOTE output channel driver.
    #[cfg(feature = "_nrf54l")]
    pub fn new<C: Channel, T: GpiotePin<Instance = C::Instance>>(
        ch: Peri<'d, C>,
        pin: Peri<'d, T>,
        initial_output: Level,
        drive: OutputDrive,
        polarity: OutputChannelPolarity,
    ) -> Self {
        let pin = Output::new(pin, initial_output, drive);
        let ch = ch.into();
        Self::new_inner(ch, pin, polarity)
    }

    /// Create a new GPIOTE output channel driver.
    #[cfg(not(feature = "_nrf54l"))]
    pub fn new<C: Channel, T: GpioPin>(
        ch: Peri<'d, C>,
        pin: Peri<'d, T>,
        initial_output: Level,
        drive: OutputDrive,
        polarity: OutputChannelPolarity,
    ) -> Self {
        let pin = Output::new(pin, initial_output, drive);
        let ch = ch.into();
        Self::new_inner(ch, pin, polarity)
    }

    fn new_inner(ch: Peri<'d, AnyChannel>, pin: Output<'d>, polarity: OutputChannelPolarity) -> Self {
        let g = ch.regs();
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
            #[cfg(any(feature = "_nrf54l"))]
            w.set_port(match pin.pin.pin.port() {
                crate::gpio::Port::Port0 => 0,
                crate::gpio::Port::Port1 => 1,
                crate::gpio::Port::Port2 => 2,
            });
            w.set_psel(pin.pin.pin.pin());
        });

        OutputChannel { ch, _pin: pin }
    }

    /// Triggers the OUT task (does the action as configured with task_out_polarity, defaults to Toggle).
    pub fn out(&self) {
        let g = self.ch.regs();
        g.tasks_out(self.ch.number()).write_value(1);
    }

    /// Triggers the SET task (set associated pin high).
    #[cfg(not(feature = "_nrf51"))]
    pub fn set(&self) {
        let g = self.ch.regs();
        g.tasks_set(self.ch.number()).write_value(1);
    }

    /// Triggers the CLEAR task (set associated pin low).
    #[cfg(not(feature = "_nrf51"))]
    pub fn clear(&self) {
        let g = self.ch.regs();
        g.tasks_clr(self.ch.number()).write_value(1);
    }

    /// Returns the OUT task, for use with PPI.
    pub fn task_out(&self) -> Task<'d> {
        let g = self.ch.regs();
        Task::from_reg(g.tasks_out(self.ch.number()))
    }

    /// Returns the CLR task, for use with PPI.
    #[cfg(not(feature = "_nrf51"))]
    pub fn task_clr(&self) -> Task<'d> {
        let g = self.ch.regs();
        Task::from_reg(g.tasks_clr(self.ch.number()))
    }

    /// Returns the SET task, for use with PPI.
    #[cfg(not(feature = "_nrf51"))]
    pub fn task_set(&self) -> Task<'d> {
        let g = self.ch.regs();
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
//

trait SealedChannel {
    fn waker(&self) -> usize;
    fn regs(&self) -> pac::gpiote::Gpiote;
}

/// GPIOTE channel trait.
///
/// Implemented by all GPIOTE channels.
#[allow(private_bounds)]
pub trait Channel: PeripheralType + SealedChannel + Into<AnyChannel> + Sized + 'static {
    #[cfg(feature = "_nrf54l")]
    /// GPIOTE instance this channel belongs to.
    type Instance: GpioteInstance;
    /// Get the channel number.
    fn number(&self) -> usize;
}

struct AnyChannel {
    number: u8,
    regs: pac::gpiote::Gpiote,
    waker: u8,
}

impl_peripheral!(AnyChannel);

impl SealedChannel for AnyChannel {
    fn waker(&self) -> usize {
        self.waker as usize
    }

    fn regs(&self) -> pac::gpiote::Gpiote {
        self.regs
    }
}

#[cfg(feature = "_nrf54l")]
impl AnyChannel {
    fn number(&self) -> usize {
        self.number as usize
    }
}

#[cfg(not(feature = "_nrf54l"))]
impl Channel for AnyChannel {
    fn number(&self) -> usize {
        self.number as usize
    }
}

macro_rules! impl_channel {
    ($type:ident, $inst:ident, $number:expr, $waker:expr) => {
        impl SealedChannel for peripherals::$type {
            fn waker(&self) -> usize {
                $waker as usize
            }

            fn regs(&self) -> pac::gpiote::Gpiote {
                pac::$inst
            }
        }
        impl Channel for peripherals::$type {
            #[cfg(feature = "_nrf54l")]
            type Instance = peripherals::$inst;
            fn number(&self) -> usize {
                $number as usize
            }
        }

        impl From<peripherals::$type> for AnyChannel {
            fn from(val: peripherals::$type) -> Self {
                Self {
                    number: val.number() as u8,
                    waker: val.waker() as u8,
                    regs: val.regs(),
                }
            }
        }
    };
}

cfg_if::cfg_if! {
    if #[cfg(feature = "_nrf54l")] {
        trait SealedGpioteInstance {}
        /// Represents a GPIOTE instance.
        #[allow(private_bounds)]
        pub trait GpioteInstance: PeripheralType + SealedGpioteInstance + Sized + 'static {}

        macro_rules! impl_gpiote {
            ($type:ident) => {
                impl SealedGpioteInstance for peripherals::$type {}
                impl GpioteInstance for peripherals::$type {}
            };
        }

        pub(crate) trait SealedGpiotePin {}

        /// Represents a GPIO pin that can be used with GPIOTE.
        #[allow(private_bounds)]
        pub trait GpiotePin: GpioPin + SealedGpiotePin {
            /// The GPIOTE instance this pin belongs to.
            type Instance: GpioteInstance;
        }

        macro_rules! impl_gpiote_pin {
            ($type:ident, $inst:ident) => {
                impl crate::gpiote::SealedGpiotePin for peripherals::$type {}
                impl crate::gpiote::GpiotePin for peripherals::$type {
                    type Instance = peripherals::$inst;
                }
            };
        }

        impl_gpiote!(GPIOTE20);
        impl_gpiote!(GPIOTE30);
        impl_channel!(GPIOTE20_CH0, GPIOTE20, 0, 0);
        impl_channel!(GPIOTE20_CH1, GPIOTE20, 1, 1);
        impl_channel!(GPIOTE20_CH2, GPIOTE20, 2, 2);
        impl_channel!(GPIOTE20_CH3, GPIOTE20, 3, 3);
        impl_channel!(GPIOTE20_CH4, GPIOTE20, 4, 4);
        impl_channel!(GPIOTE20_CH5, GPIOTE20, 5, 5);
        impl_channel!(GPIOTE20_CH6, GPIOTE20, 6, 6);
        impl_channel!(GPIOTE20_CH7, GPIOTE20, 7, 7);

        impl_channel!(GPIOTE30_CH0, GPIOTE30, 0, 8);
        impl_channel!(GPIOTE30_CH1, GPIOTE30, 1, 9);
        impl_channel!(GPIOTE30_CH2, GPIOTE30, 2, 10);
        impl_channel!(GPIOTE30_CH3, GPIOTE30, 3, 11);
    } else if #[cfg(feature = "_nrf51")] {
        impl_channel!(GPIOTE_CH0, GPIOTE, 0, 0);
        impl_channel!(GPIOTE_CH1, GPIOTE, 1, 1);
        impl_channel!(GPIOTE_CH2, GPIOTE, 2, 2);
        impl_channel!(GPIOTE_CH3, GPIOTE, 3, 3);
    } else if #[cfg(all(feature = "_s", any(feature = "_nrf91", feature = "_nrf5340")))] {
        impl_channel!(GPIOTE_CH0, GPIOTE0, 0, 0);
        impl_channel!(GPIOTE_CH1, GPIOTE0, 1, 1);
        impl_channel!(GPIOTE_CH2, GPIOTE0, 2, 2);
        impl_channel!(GPIOTE_CH3, GPIOTE0, 3, 3);
        impl_channel!(GPIOTE_CH4, GPIOTE0, 4, 4);
        impl_channel!(GPIOTE_CH5, GPIOTE0, 5, 5);
        impl_channel!(GPIOTE_CH6, GPIOTE0, 6, 6);
        impl_channel!(GPIOTE_CH7, GPIOTE0, 7, 7);
    } else if #[cfg(all(feature = "_ns", any(feature = "_nrf91", feature = "_nrf5340")))] {
        impl_channel!(GPIOTE_CH0, GPIOTE1, 0, 0);
        impl_channel!(GPIOTE_CH1, GPIOTE1, 1, 1);
        impl_channel!(GPIOTE_CH2, GPIOTE1, 2, 2);
        impl_channel!(GPIOTE_CH3, GPIOTE1, 3, 3);
        impl_channel!(GPIOTE_CH4, GPIOTE1, 4, 4);
        impl_channel!(GPIOTE_CH5, GPIOTE1, 5, 5);
        impl_channel!(GPIOTE_CH6, GPIOTE1, 6, 6);
        impl_channel!(GPIOTE_CH7, GPIOTE1, 7, 7);
    } else {
        impl_channel!(GPIOTE_CH0, GPIOTE, 0, 0);
        impl_channel!(GPIOTE_CH1, GPIOTE, 1, 1);
        impl_channel!(GPIOTE_CH2, GPIOTE, 2, 2);
        impl_channel!(GPIOTE_CH3, GPIOTE, 3, 3);
        impl_channel!(GPIOTE_CH4, GPIOTE, 4, 4);
        impl_channel!(GPIOTE_CH5, GPIOTE, 5, 5);
        impl_channel!(GPIOTE_CH6, GPIOTE, 6, 6);
        impl_channel!(GPIOTE_CH7, GPIOTE, 7, 7);
    }
}

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
