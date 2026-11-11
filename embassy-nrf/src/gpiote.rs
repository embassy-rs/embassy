//! GPIO task/event (GPIOTE) driver.
//!
//! GPIOTE channels are driven by [`InputChannel`] and [`OutputChannel`]. Async waits on a single
//! pin through the PORT event are provided by [`gpio::Input`](crate::gpio::Input) and friends in
//! async mode, see [`gpio::InterruptHandler`](crate::gpio::InterruptHandler).
#![macro_use]

use core::convert::Infallible;
use core::future::{Future, poll_fn};
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{Input, Level, Output, OutputDrive, Pin as GpioPin, Pull};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::mode::{Async, Blocking, Mode};
#[cfg(not(feature = "_nrf51"))]
use crate::pac::gpio::vals::Detectmode;
use crate::pac::gpiote::vals::{Mode as ChannelMode, Outinit, Polarity};
use crate::ppi::{Event, Task};
use crate::{pac, peripherals};

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

#[cfg(all(feature = "_nrf54l", feature = "_ns"))]
const INTNUM: usize = 1;

#[cfg(any(not(feature = "_nrf54l"), feature = "_s"))]
const INTNUM: usize = 0;

/// Set up `T`'s port and GPIOTE for async waits on the PORT event: latched detection, the PORT
/// interrupt enable, and the interrupt in the NVIC.
pub(crate) fn enable_port_interrupt<T: GpiotePin>() {
    #[cfg(not(feature = "_nrf51"))]
    {
        let p = T::PIN_NUMBER.block();
        #[cfg(all(feature = "_s", not(feature = "_nrf54l")))]
        p.detectmode_sec().write(|w| w.set_detectmode(Detectmode::Ldetect));
        #[cfg(any(not(feature = "_s"), all(feature = "_s", feature = "_nrf54l")))]
        p.detectmode().write(|w| w.set_detectmode(Detectmode::Ldetect));
    }

    let g = T::regs();
    #[cfg(not(feature = "_nrf54l"))]
    g.intenset(INTNUM).write(|w| w.set_port(true));
    #[cfg(all(feature = "_nrf54l", feature = "_ns"))]
    g.intenset(INTNUM).write(|w| w.set_port0nonsecure(true));
    #[cfg(all(feature = "_nrf54l", feature = "_s"))]
    g.intenset(INTNUM).write(|w| w.set_port0secure(true));

    unsafe { T::Interrupt::enable() };
}

/// Clear the PORT event of a GPIOTE instance.
pub(crate) fn clear_port_event(g: pac::gpiote::Gpiote) {
    #[cfg(not(feature = "_nrf54l"))]
    g.events_port(0).write_value(0);
    #[cfg(all(feature = "_nrf54l", feature = "_ns"))]
    g.events_port(0).nonsecure().write_value(0);
    #[cfg(all(feature = "_nrf54l", feature = "_s"))]
    g.events_port(0).secure().write_value(0);
}

/// GPIOTE channel interrupt handler.
///
/// Bind one handler per channel used with an async [`InputChannel`], on that channel's GPIOTE
/// interrupt ([`Channel::Interrupt`]):
///
/// ```rust,ignore
/// bind_interrupts!(struct Irqs {
///     GPIOTE => gpiote::InterruptHandler<peripherals::GPIOTE_CH0>, gpiote::InterruptHandler<peripherals::GPIOTE_CH1>;
/// });
/// ```
///
/// The handler only services its own channel, so the same interrupt can also be bound to
/// [`gpio::InterruptHandler`](crate::gpio::InterruptHandler) for async pins, and to your own
/// [`Handler`] for other channels.
pub struct InterruptHandler<C: Channel> {
    _phantom: PhantomData<C>,
}

impl<C: Channel> Handler<C::Interrupt> for InterruptHandler<C> {
    unsafe fn on_interrupt() {
        let g = C::regs();
        let mask = 1 << C::NUMBER;
        // Only act if a wait armed the interrupt. The event stays set after a wait completes.
        if g.events_in(C::NUMBER).read() != 0 && g.intenset(INTNUM).read().0 & mask != 0 {
            g.intenclr(INTNUM).write(|w| w.0 = mask);
            C::waker().wake();
        }
    }
}

/// GPIOTE channel driver in input mode
pub struct InputChannel<'d, M: Mode = Blocking> {
    ch: Peri<'d, AnyChannel>,
    pin: Input<'d>,
    waker: M::IfAsync<&'static AtomicWaker>,
    _mode: PhantomData<M>,
}

impl<M: Mode> InputChannel<'static, M> {
    /// Persist the channel's configuration for the rest of the program's lifetime. This method
    /// should be preferred over [`core::mem::forget()`] because the `'static` bound prevents
    /// accidental reuse of the underlying peripheral.
    pub fn persist(self) {
        core::mem::forget(self);
    }
}

impl<'d, M: Mode> Drop for InputChannel<'d, M> {
    fn drop(&mut self) {
        let g = self.ch.regs();
        let num = self.ch.number();
        g.config(num).write(|w| w.set_mode(ChannelMode::Disabled));
        g.intenclr(INTNUM).write(|w| w.0 = 1 << num);
    }
}

impl<'d> InputChannel<'d, Blocking> {
    /// Create a new blocking GPIOTE input channel driver.
    ///
    /// The channel's IN event can be used with PPI through [`event_in`](Self::event_in).
    #[cfg(feature = "_nrf54l")]
    pub fn new<C: Channel, T: GpiotePin<Instance = C::Instance>>(
        ch: Peri<'d, C>,
        pin: Peri<'d, T>,
        pull: Pull,
        polarity: InputChannelPolarity,
    ) -> Self {
        Self::new_inner(ch.into(), Input::new(pin, pull), (), polarity)
    }

    /// Create a new blocking GPIOTE input channel driver.
    ///
    /// The channel's IN event can be used with PPI through [`event_in`](Self::event_in).
    #[cfg(not(feature = "_nrf54l"))]
    pub fn new<C: Channel, T: GpioPin>(
        ch: Peri<'d, C>,
        pin: Peri<'d, T>,
        pull: Pull,
        polarity: InputChannelPolarity,
    ) -> Self {
        Self::new_inner(ch.into(), Input::new(pin, pull), (), polarity)
    }
}

impl<'d> InputChannel<'d, Async> {
    /// Create a new async GPIOTE input channel driver.
    ///
    /// The channel's GPIOTE interrupt must be bound to [`InterruptHandler`] for this channel.
    #[cfg(feature = "_nrf54l")]
    pub fn new_async<C: Channel, T: GpiotePin<Instance = C::Instance>>(
        ch: Peri<'d, C>,
        pin: Peri<'d, T>,
        _irq: impl Binding<C::Interrupt, InterruptHandler<C>> + 'd,
        pull: Pull,
        polarity: InputChannelPolarity,
    ) -> Self {
        unsafe { C::Interrupt::enable() };
        Self::new_inner(ch.into(), Input::new(pin, pull), C::waker(), polarity)
    }

    /// Create a new async GPIOTE input channel driver.
    ///
    /// The channel's GPIOTE interrupt must be bound to [`InterruptHandler`] for this channel.
    #[cfg(not(feature = "_nrf54l"))]
    pub fn new_async<C: Channel, T: GpioPin>(
        ch: Peri<'d, C>,
        pin: Peri<'d, T>,
        _irq: impl Binding<C::Interrupt, InterruptHandler<C>> + 'd,
        pull: Pull,
        polarity: InputChannelPolarity,
    ) -> Self {
        unsafe { C::Interrupt::enable() };
        Self::new_inner(ch.into(), Input::new(pin, pull), C::waker(), polarity)
    }

    /// Asynchronously wait for an event in this channel.
    ///
    /// It is possible to call this function and await the returned future later.
    /// If an even occurs in the mean time, the future will immediately report ready.
    pub fn wait(&mut self) -> impl Future<Output = ()> {
        // NOTE: This is `-> impl Future` and not an `async fn` on purpose.
        // Otherwise, events will only be detected starting at the first poll of the returned future.
        Self::wait_internal(&mut self.ch, self.waker)
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
        let wait = Self::wait_internal(&mut self.ch, self.waker);
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
        let wait = Self::wait_internal(&mut self.ch, self.waker);
        let pin = &self.pin;
        async move {
            if pin.is_low() {
                return;
            }
            wait.await;
        }
    }

    /// Internal implementation for `wait()` and friends.
    fn wait_internal(channel: &mut Peri<'_, AnyChannel>, waker: &'static AtomicWaker) -> impl Future<Output = ()> {
        // NOTE: This is `-> impl Future` and not an `async fn` on purpose.
        // Otherwise, events will only be detected starting at the first poll of the returned future.

        let g = channel.regs();
        let num = channel.number();

        // Enable interrupt
        g.events_in(num).write_value(0);
        g.intenset(INTNUM).write(|w| w.0 = 1 << num);

        poll_fn(move |cx| {
            waker.register(cx.waker());

            if g.events_in(num).read() != 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
    }
}

impl<'d, M: Mode> InputChannel<'d, M> {
    fn new_inner(
        ch: Peri<'d, AnyChannel>,
        pin: Input<'d>,
        waker: M::IfAsync<&'static AtomicWaker>,
        polarity: InputChannelPolarity,
    ) -> Self {
        let g = ch.regs();
        let num = ch.number();
        g.config(num).write(|w| {
            w.set_mode(ChannelMode::Event);
            match polarity {
                InputChannelPolarity::HiToLo => w.set_polarity(Polarity::HiToLo),
                InputChannelPolarity::LoToHi => w.set_polarity(Polarity::LoToHi),
                InputChannelPolarity::None => w.set_polarity(Polarity::None),
                InputChannelPolarity::Toggle => w.set_polarity(Polarity::Toggle),
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
                #[cfg(feature = "_gpio-p3")]
                crate::gpio::Port::Port3 => 3,
            });
            w.set_psel(pin.pin.pin.pin());
        });

        g.events_in(num).write_value(0);

        InputChannel {
            ch,
            pin,
            waker,
            _mode: PhantomData,
        }
    }

    /// Get the associated input pin.
    pub fn pin(&self) -> &Input<'d> {
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
        g.config(num).write(|w| w.set_mode(ChannelMode::Disabled));
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
            w.set_mode(ChannelMode::Task);
            match pin.is_set_high() {
                true => w.set_outinit(Outinit::High),
                false => w.set_outinit(Outinit::Low),
            };
            match polarity {
                OutputChannelPolarity::Set => w.set_polarity(Polarity::HiToLo),
                OutputChannelPolarity::Clear => w.set_polarity(Polarity::LoToHi),
                OutputChannelPolarity::Toggle => w.set_polarity(Polarity::Toggle),
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
                #[cfg(feature = "_gpio-p3")]
                crate::gpio::Port::Port3 => 3,
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

pub(crate) trait SealedChannel {
    /// Channel number within its GPIOTE instance.
    const NUMBER: usize;

    /// Registers of the GPIOTE instance this channel belongs to.
    fn regs() -> pac::gpiote::Gpiote;

    /// Waker of the async driver for this channel.
    ///
    /// Each channel has its own `static`, so wakers only take RAM for channels used in async mode.
    fn waker() -> &'static AtomicWaker;
}

/// GPIOTE channel trait.
///
/// Implemented by all GPIOTE channels.
#[allow(private_bounds)]
pub trait Channel: PeripheralType + SealedChannel + Into<AnyChannel> + Sized + 'static {
    #[cfg(feature = "_nrf54l")]
    /// GPIOTE instance this channel belongs to.
    type Instance: GpioteInstance;

    /// Interrupt of the GPIOTE instance this channel belongs to.
    type Interrupt: Interrupt;

    /// Get the channel number.
    fn number(&self) -> usize {
        Self::NUMBER
    }
}

pub(crate) struct AnyChannel {
    number: u8,
    regs: pac::gpiote::Gpiote,
}

impl_peripheral!(AnyChannel);

impl AnyChannel {
    fn number(&self) -> usize {
        self.number as usize
    }

    fn regs(&self) -> pac::gpiote::Gpiote {
        self.regs
    }
}

macro_rules! impl_channel {
    ($type:ident, $inst:ident, $number:expr) => {
        impl SealedChannel for peripherals::$type {
            const NUMBER: usize = $number;

            fn regs() -> pac::gpiote::Gpiote {
                pac::$inst
            }

            fn waker() -> &'static AtomicWaker {
                static WAKER: AtomicWaker = AtomicWaker::new();
                &WAKER
            }
        }
        impl Channel for peripherals::$type {
            #[cfg(feature = "_nrf54l")]
            type Instance = peripherals::$inst;
            #[cfg(feature = "_nrf54l")]
            type Interrupt = <peripherals::$inst as GpioteInstance>::Interrupt;
            #[cfg(not(feature = "_nrf54l"))]
            type Interrupt = crate::interrupt::typelevel::$inst;
        }

        impl From<peripherals::$type> for AnyChannel {
            fn from(_val: peripherals::$type) -> Self {
                Self {
                    number: $number,
                    regs: pac::$inst,
                }
            }
        }
    };
}

pub(crate) trait SealedGpiotePin {
    /// Number of this pin, as returned by [`SealedPin::pin_number()`](crate::gpio::SealedPin::pin_number).
    const PIN_NUMBER: crate::gpio::PinNumber;

    /// Registers of the GPIOTE instance serving this pin's port.
    fn regs() -> pac::gpiote::Gpiote;

    /// Waker of the async GPIO driver for this pin.
    ///
    /// Each pin has its own `static`, so wakers only take RAM for pins used in async mode.
    fn waker() -> &'static AtomicWaker;
}

/// A GPIO pin served by a GPIOTE instance.
///
/// These pins can be used with GPIOTE channels and with the async GPIO drivers. On nRF54L, only
/// some ports are served by a GPIOTE instance.
#[allow(private_bounds)]
pub trait GpiotePin: GpioPin + SealedGpiotePin {
    /// The GPIOTE instance this pin belongs to.
    #[cfg(feature = "_nrf54l")]
    type Instance: GpioteInstance;

    /// Interrupt of the GPIOTE instance serving this pin's port.
    type Interrupt: Interrupt;
}

cfg_select! {
    feature = "_nrf54l" => {
        pub(crate) trait SealedGpioteInstance {
            fn regs() -> pac::gpiote::Gpiote;
        }

        /// Represents a GPIOTE instance.
        #[allow(private_bounds)]
        pub trait GpioteInstance: PeripheralType + SealedGpioteInstance + Sized + 'static {
            /// Interrupt of this GPIOTE instance.
            type Interrupt: Interrupt;
        }

        macro_rules! impl_gpiote {
            ($type:ident, $irq_s:ident, $irq_ns:ident) => {
                impl SealedGpioteInstance for peripherals::$type {
                    fn regs() -> pac::gpiote::Gpiote {
                        pac::$type
                    }
                }
                impl GpioteInstance for peripherals::$type {
                    #[cfg(feature = "_s")]
                    type Interrupt = crate::interrupt::typelevel::$irq_s;
                    #[cfg(feature = "_ns")]
                    type Interrupt = crate::interrupt::typelevel::$irq_ns;
                }
            };
        }

        macro_rules! impl_gpiote_pin {
            ($type:ident, $port_num:expr, $pin_num:expr, $inst:ident) => {
                impl crate::gpiote::SealedGpiotePin for peripherals::$type {
                    const PIN_NUMBER: crate::gpio::PinNumber = crate::gpio::PinNumber::new($port_num * 32 + $pin_num);

                    fn regs() -> crate::pac::gpiote::Gpiote {
                        <peripherals::$inst as crate::gpiote::SealedGpioteInstance>::regs()
                    }

                    fn waker() -> &'static embassy_sync::waitqueue::AtomicWaker {
                        static WAKER: embassy_sync::waitqueue::AtomicWaker =
                            embassy_sync::waitqueue::AtomicWaker::new();
                        &WAKER
                    }
                }
                impl crate::gpiote::GpiotePin for peripherals::$type {
                    type Instance = peripherals::$inst;
                    type Interrupt = <peripherals::$inst as crate::gpiote::GpioteInstance>::Interrupt;
                }
            };
        }

        impl_gpiote!(GPIOTE20, GPIOTE20_0, GPIOTE20_1);
        impl_gpiote!(GPIOTE30, GPIOTE30_0, GPIOTE30_1);
        impl_channel!(GPIOTE20_CH0, GPIOTE20, 0);
        impl_channel!(GPIOTE20_CH1, GPIOTE20, 1);
        impl_channel!(GPIOTE20_CH2, GPIOTE20, 2);
        impl_channel!(GPIOTE20_CH3, GPIOTE20, 3);
        impl_channel!(GPIOTE20_CH4, GPIOTE20, 4);
        impl_channel!(GPIOTE20_CH5, GPIOTE20, 5);
        impl_channel!(GPIOTE20_CH6, GPIOTE20, 6);
        impl_channel!(GPIOTE20_CH7, GPIOTE20, 7);

        impl_channel!(GPIOTE30_CH0, GPIOTE30, 0);
        impl_channel!(GPIOTE30_CH1, GPIOTE30, 1);
        impl_channel!(GPIOTE30_CH2, GPIOTE30, 2);
        impl_channel!(GPIOTE30_CH3, GPIOTE30, 3);
    }
    _ => {
        // Outside nRF54L, there's one GPIOTE instance, serving all pins.
        cfg_select! {
            all(feature = "_s", any(feature = "_nrf91", feature = "_nrf5340")) => {
                pub(crate) const PIN_REGS: pac::gpiote::Gpiote = pac::GPIOTE0;
                pub(crate) type PinInterrupt = crate::interrupt::typelevel::GPIOTE0;
            }
            all(feature = "_ns", any(feature = "_nrf91", feature = "_nrf5340")) => {
                pub(crate) const PIN_REGS: pac::gpiote::Gpiote = pac::GPIOTE1;
                pub(crate) type PinInterrupt = crate::interrupt::typelevel::GPIOTE1;
            }
            _ => {
                pub(crate) const PIN_REGS: pac::gpiote::Gpiote = pac::GPIOTE;
                pub(crate) type PinInterrupt = crate::interrupt::typelevel::GPIOTE;
            }
        }

        macro_rules! impl_gpiote_pin {
            ($type:ident, $port_num:expr, $pin_num:expr) => {
                impl crate::gpiote::SealedGpiotePin for peripherals::$type {
                    const PIN_NUMBER: crate::gpio::PinNumber = crate::gpio::PinNumber::new($port_num * 32 + $pin_num);

                    fn regs() -> crate::pac::gpiote::Gpiote {
                        crate::gpiote::PIN_REGS
                    }

                    fn waker() -> &'static embassy_sync::waitqueue::AtomicWaker {
                        static WAKER: embassy_sync::waitqueue::AtomicWaker =
                            embassy_sync::waitqueue::AtomicWaker::new();
                        &WAKER
                    }
                }
                impl crate::gpiote::GpiotePin for peripherals::$type {
                    type Interrupt = crate::gpiote::PinInterrupt;
                }
            };
        }
    }
}

cfg_select! {
    feature = "_nrf54l" => {}
    feature = "_nrf51" => {
        impl_channel!(GPIOTE_CH0, GPIOTE, 0);
        impl_channel!(GPIOTE_CH1, GPIOTE, 1);
        impl_channel!(GPIOTE_CH2, GPIOTE, 2);
        impl_channel!(GPIOTE_CH3, GPIOTE, 3);
    }
    all(feature = "_s", any(feature = "_nrf91", feature = "_nrf5340")) => {
        impl_channel!(GPIOTE_CH0, GPIOTE0, 0);
        impl_channel!(GPIOTE_CH1, GPIOTE0, 1);
        impl_channel!(GPIOTE_CH2, GPIOTE0, 2);
        impl_channel!(GPIOTE_CH3, GPIOTE0, 3);
        impl_channel!(GPIOTE_CH4, GPIOTE0, 4);
        impl_channel!(GPIOTE_CH5, GPIOTE0, 5);
        impl_channel!(GPIOTE_CH6, GPIOTE0, 6);
        impl_channel!(GPIOTE_CH7, GPIOTE0, 7);
    }
    all(feature = "_ns", any(feature = "_nrf91", feature = "_nrf5340")) => {
        impl_channel!(GPIOTE_CH0, GPIOTE1, 0);
        impl_channel!(GPIOTE_CH1, GPIOTE1, 1);
        impl_channel!(GPIOTE_CH2, GPIOTE1, 2);
        impl_channel!(GPIOTE_CH3, GPIOTE1, 3);
        impl_channel!(GPIOTE_CH4, GPIOTE1, 4);
        impl_channel!(GPIOTE_CH5, GPIOTE1, 5);
        impl_channel!(GPIOTE_CH6, GPIOTE1, 6);
        impl_channel!(GPIOTE_CH7, GPIOTE1, 7);
    }
    _ => {
        impl_channel!(GPIOTE_CH0, GPIOTE, 0);
        impl_channel!(GPIOTE_CH1, GPIOTE, 1);
        impl_channel!(GPIOTE_CH2, GPIOTE, 2);
        impl_channel!(GPIOTE_CH3, GPIOTE, 3);
        impl_channel!(GPIOTE_CH4, GPIOTE, 4);
        impl_channel!(GPIOTE_CH5, GPIOTE, 5);
        impl_channel!(GPIOTE_CH6, GPIOTE, 6);
        impl_channel!(GPIOTE_CH7, GPIOTE, 7);
    }
}

// ====================

mod eh02 {
    use super::*;

    impl<'d, M: Mode> embedded_hal_02::digital::v2::InputPin for InputChannel<'d, M> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.pin.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.pin.is_low())
        }
    }
}

impl<'d, M: Mode> embedded_hal_1::digital::ErrorType for InputChannel<'d, M> {
    type Error = Infallible;
}

impl<'d, M: Mode> embedded_hal_1::digital::InputPin for InputChannel<'d, M> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.pin.is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.pin.is_low())
    }
}
