//! GPIO task/event (GPIOTE) driver.

use core::convert::Infallible;
use core::future::{poll_fn, Future};
use core::task::{Context, Poll};

use embassy_hal_internal::{impl_peripheral, Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AnyPin, Flex, Input, Output, Pin as GpioPin, SealedPin as _};
use crate::interrupt::InterruptExt;
#[cfg(not(any(feature = "_nrf51", all(feature = "_nrf54l", feature = "_ns"))))]
use crate::pac::gpio::vals::Detectmode;
use crate::pac::gpio::vals::Sense;
use crate::pac::gpiote::vals::{Mode, Outinit, Polarity};
use crate::ppi::{Event, Task};
use crate::{interrupt, pac, peripherals};

#[cfg(feature = "_nrf51")]
/// Amount of GPIOTE channels in the chip.
const CHANNEL_COUNT: usize = 4;
#[cfg(feature = "_nrf54l")]
/// Amount of GPIOTE channels in the chip.
const CHANNEL_COUNT: usize = 8 /*P1: PERI PD*/ + 4 /*P0: LP PD*/;
#[cfg(not(any(feature = "_nrf51", feature = "_nrf54l")))]
/// Amount of GPIOTE channels in the chip.
const CHANNEL_COUNT: usize = 8;

#[cfg(any(feature = "nrf52833", feature = "nrf52840", feature = "_nrf5340"))]
const PIN_COUNT: usize = 48;
#[cfg(feature = "_nrf54l")]
const PIN_COUNT: usize = 16 /*P1: PERI PD*/ + 11 /*P0: LP PD*/;
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

#[cfg(feature = "_nrf54l")]
#[derive(Copy, Clone)]
/// GPIOTE instance
pub enum GpioteInstance {
    /// GPIOTE_20 instance (GPIO port 1)
    Gpiote20,
    /// GPIOTE_30 instance (GPIO port 0)
    Gpiote30,
}

trait GpioteWakerResolver {
    fn channel_waker(&self, ch: usize) -> &'static AtomicWaker;
    fn port_waker(&self, pin: usize) -> &'static AtomicWaker;
    fn channel_count(&self) -> usize;
}

#[cfg(feature = "_nrf54l")]
type GpioteInstanceType = GpioteInstance;

#[cfg(feature = "_nrf54l")]
impl From<&AnyPin> for GpioteInstance {
    fn from(pin: &AnyPin) -> Self {
        match pin.pin_port() {
            0..=31 => GpioteInstance::Gpiote30,
            32..=63 => GpioteInstance::Gpiote20,
            _ => panic!("Invalid pin_port"),
        }
    }
}

#[cfg(feature = "_nrf54l")]
impl GpioteWakerResolver for GpioteInstance {
    fn channel_waker(&self, ch: usize) -> &'static AtomicWaker {
        match self {
            GpioteInstance::Gpiote20 => &CHANNEL_WAKERS[ch + 0],
            GpioteInstance::Gpiote30 => &CHANNEL_WAKERS[ch + 8],
        }
    }
    fn port_waker(&self, pin: usize) -> &'static AtomicWaker {
        match self {
            GpioteInstance::Gpiote20 => &PORT_WAKERS[pin + 0],
            GpioteInstance::Gpiote30 => &PORT_WAKERS[pin + 16],
        }
    }
    fn channel_count(&self) -> usize {
        match self {
            GpioteInstance::Gpiote20 => 8,
            GpioteInstance::Gpiote30 => 4,
        }
    }
}

#[cfg(not(feature = "_nrf54l"))]
type GpioteInstanceType = ();

#[cfg(not(feature = "_nrf54l"))]
impl From<&AnyPin> for () {
    fn from(_pin: &AnyPin) -> Self {
        ()
    }
}

#[cfg(not(feature = "_nrf54l"))]
impl GpioteWakerResolver for () {
    fn channel_waker(&self, ch: usize) -> &'static AtomicWaker {
        &CHANNEL_WAKERS[ch]
    }
    fn port_waker(&self, pin: usize) -> &'static AtomicWaker {
        &PORT_WAKERS[pin]
    }
    fn channel_count(&self) -> usize {
        CHANNEL_COUNT
    }
}

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

// Helper to get the appropriate INTENSET register for the active GPIOTE peripheral.
fn intenset(g: &pac::gpiote::Gpiote) -> pac::common::Reg<pac::gpiote::regs::Int, pac::common::RW> {
    cfg_if::cfg_if! {
        if #[cfg(all(feature = "_nrf54l", feature = "_s"))] {
            g.intenset(0)
        } else if #[cfg(all(feature = "_nrf54l", feature = "_ns"))] {
            g.intenset(1)
        } else {
            g.intenset()
        }
    }
}

// Helper to get the appropriate INTENCLR register for the active GPIOTE peripheral.
fn intenclr(g: &pac::gpiote::Gpiote) -> pac::common::Reg<pac::gpiote::regs::Int, pac::common::RW> {
    cfg_if::cfg_if! {
        if #[cfg(all(feature = "_nrf54l", feature = "_s"))] {
            g.intenclr(0)
        } else if #[cfg(all(feature = "_nrf54l", feature = "_ns"))] {
            g.intenclr(1)
        } else {
            g.intenclr()
        }
    }
}

// Helper to get the appropriate port event register for the active GPIOTE peripheral.
fn events_port(g: &pac::gpiote::Gpiote) -> pac::common::Reg<u32, pac::common::RW> {
    cfg_if::cfg_if! {
        if #[cfg(all(feature = "_nrf54l", feature = "_s"))] {
            g.events_port(0).secure()
        } else if #[cfg(all(feature = "_nrf54l", feature = "_ns"))] {
            g.events_port(0).nonsecure()
        } else {
            g.events_port()
        }
    }
}

// Helper to enable or disabe port event interrupts for the active MCU
fn int_set_port(int: &mut pac::gpiote::regs::Int, val: bool) {
    cfg_if::cfg_if! {
        if #[cfg(all(feature = "_nrf54l", feature = "_s"))] {
            int.set_port0secure(val);
        } else if #[cfg(all(feature = "_nrf54l", feature = "_ns"))] {
            int.set_port0nonsecure(val);
        } else {
            int.set_port(val);
        }
    }
}

// Helper to get the GPIOTE peripheral’s register block for the active MCU and GPIOTE instance.
fn regs(_inst: GpioteInstanceType) -> pac::gpiote::Gpiote {
    cfg_if::cfg_if! {
        if #[cfg(any(feature="nrf5340-app-s", feature="nrf9160-s", feature="nrf9120-s"))] {
            pac::GPIOTE0
        } else if #[cfg(any(feature="nrf5340-app-ns", feature="nrf9160-ns", feature="nrf9120-ns"))] {
            pac::GPIOTE1
        } else if #[cfg(feature="_nrf54l")] {
            match _inst {
                GpioteInstance::Gpiote20 => pac::GPIOTE20,
                GpioteInstance::Gpiote30 => pac::GPIOTE30,
            }
        } else {
            pac::GPIOTE
        }
    }
}

pub(crate) fn init(irq_prio: crate::interrupt::Priority) {
    // no latched GPIO detect in nrf51.
    // in the nrf54l-ns, the detectmode register is inaccessible
    #[cfg(not(any(feature = "_nrf51", all(feature = "_nrf54l", feature = "_ns"))))]
    {
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

        for &p in ports {
            // Enable latched detection
            p.detectmode().write(|w| w.set_detectmode(Detectmode::LDETECT));
            // Clear latch
            p.latch().write(|w| w.0 = 0xFFFFFFFF)
        }
    }

    // Enable interrupts
    #[cfg(any(feature = "nrf5340-app-s", feature = "nrf9160-s", feature = "nrf9120-s"))]
    let irqs = &[interrupt::GPIOTE0];
    #[cfg(any(feature = "nrf5340-app-ns", feature = "nrf9160-ns", feature = "nrf9120-ns"))]
    let irqs = &[interrupt::GPIOTE1];
    #[cfg(any(feature = "_nrf51", feature = "_nrf52", feature = "nrf5340-net"))]
    let irqs = &[interrupt::GPIOTE];
    #[cfg(all(feature = "_nrf54l", feature = "_s"))]
    let irqs = &[interrupt::GPIOTE20_0, interrupt::GPIOTE30_0];
    #[cfg(all(feature = "_nrf54l", feature = "_ns"))]
    let irqs = &[interrupt::GPIOTE20_1, interrupt::GPIOTE30_1];

    for &irq in irqs {
        irq.unpend();
        irq.set_priority(irq_prio);
        unsafe { irq.enable() };
    }

    #[cfg(not(feature = "_nrf54l"))]
    let instances = &[()];
    #[cfg(feature = "_nrf54l")]
    let instances = &[GpioteInstance::Gpiote20, GpioteInstance::Gpiote30];

    for &inst in instances {
        let g = regs(inst);
        intenset(&g).write(|w| int_set_port(w, true));
    }
}

#[cfg(any(feature = "nrf5340-app-s", feature = "nrf9160-s", feature = "nrf9120-s"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE0() {
    unsafe { handle_gpiote_interrupt(()) };
}

#[cfg(any(feature = "nrf5340-app-ns", feature = "nrf9160-ns", feature = "nrf9120-ns"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE1() {
    unsafe { handle_gpiote_interrupt(()) };
}

#[cfg(any(feature = "_nrf51", feature = "_nrf52", feature = "nrf5340-net"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE() {
    unsafe { handle_gpiote_interrupt(()) };
}

#[cfg(all(feature = "_nrf54l", feature = "_s"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE20_0() {
    unsafe { handle_gpiote_interrupt(GpioteInstance::Gpiote20) };
}

#[cfg(all(feature = "_nrf54l", feature = "_s"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE30_0() {
    unsafe { handle_gpiote_interrupt(GpioteInstance::Gpiote30) };
}

#[cfg(all(feature = "_nrf54l", feature = "_ns"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE20_1() {
    unsafe { handle_gpiote_interrupt(GpioteInstance::Gpiote20) };
}

#[cfg(all(feature = "_nrf54l", feature = "_ns"))]
#[cfg(feature = "rt")]
#[interrupt]
fn GPIOTE30_1() {
    unsafe { handle_gpiote_interrupt(GpioteInstance::Gpiote30) };
}

unsafe fn handle_gpiote_interrupt(inst: GpioteInstanceType) {
    let g = regs(inst);

    for i in 0..inst.channel_count() {
        if g.events_in(i).read() != 0 {
            intenclr(&g).write(|w| w.0 = 1 << i);
            inst.channel_waker(i).wake();
        }
    }

    if events_port(&g).read() != 0 {
        events_port(&g).write_value(0);

        #[cfg(any(feature = "nrf52833", feature = "nrf52840", feature = "_nrf5340"))]
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
        #[cfg(feature = "_nrf54l")]
        let ports = match inst {
            GpioteInstanceType::Gpiote30 => &[pac::P0],
            GpioteInstanceType::Gpiote20 => &[pac::P1],
        };

        #[cfg(any(feature = "_nrf51", all(feature = "_nrf54l", feature = "_ns")))]
        for (port, &p) in ports.iter().enumerate() {
            let inp = p.in_().read();
            for pin in 0..32 {
                let fired = match p.pin_cnf(pin as usize).read().sense() {
                    Sense::HIGH => inp.pin(pin),
                    Sense::LOW => !inp.pin(pin),
                    _ => false,
                };

                if fired {
                    inst.port_waker(port * 32 + pin as usize).wake();
                    p.pin_cnf(pin as usize).modify(|w| w.set_sense(Sense::DISABLED));
                }
            }
        }

        #[cfg(not(any(feature = "_nrf51", all(feature = "_nrf54l", feature = "_ns"))))]
        for (port, &p) in ports.iter().enumerate() {
            let bits = p.latch().read().0;
            for pin in BitIter(bits) {
                p.pin_cnf(pin as usize).modify(|w| w.set_sense(Sense::DISABLED));
                inst.port_waker(port * 32 + pin as usize).wake();
            }
            p.latch().write(|w| w.0 = bits);
        }
    }
}

#[cfg(not(any(feature = "_nrf51", all(feature = "_nrf54l", feature = "_ns"))))]
struct BitIter(u32);

#[cfg(not(any(feature = "_nrf51", all(feature = "_nrf54l", feature = "_ns"))))]
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
        let g = regs(self.ch.inst());
        let num = self.ch.number();
        g.config(num).write(|w| w.set_mode(Mode::DISABLED));
        intenclr(&g).write(|w| w.0 = 1 << num);
    }
}

impl<'d> InputChannel<'d> {
    /// Create a new GPIOTE input channel driver.
    pub fn new(ch: Peri<'d, impl Channel>, pin: Input<'d>, polarity: InputChannelPolarity) -> Self {
        let g = regs(ch.inst());
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
            #[cfg(feature = "_nrf54l")]
            w.set_port(match pin.pin.pin.port() {
                crate::gpio::Port::Port0 => 0,
                crate::gpio::Port::Port1 => 1,
                crate::gpio::Port::Port2 => 2,
            });
            w.set_psel(pin.pin.pin.pin());
        });

        g.events_in(num).write_value(0);

        InputChannel { ch: ch.into(), pin }
    }

    /// Asynchronously wait for an event in this channel.
    pub async fn wait(&self) {
        let inst = self.ch.inst();
        let g = regs(inst);
        let num = self.ch.number();

        // Enable interrupt
        g.events_in(num).write_value(0);
        intenset(&g).write(|w| w.0 = 1 << num);

        poll_fn(|cx| {
            inst.channel_waker(num).register(cx.waker());

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
        let g = regs(self.ch.inst());
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
        let g = regs(self.ch.inst());
        let num = self.ch.number();
        g.config(num).write(|w| w.set_mode(Mode::DISABLED));
        intenclr(&g).write(|w| w.0 = 1 << num);
    }
}

impl<'d> OutputChannel<'d> {
    /// Create a new GPIOTE output channel driver.
    pub fn new(ch: Peri<'d, impl Channel>, pin: Output<'d>, polarity: OutputChannelPolarity) -> Self {
        let g = regs(ch.inst());
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
            #[cfg(feature = "_nrf54l")]
            w.set_port(match pin.pin.pin.port() {
                crate::gpio::Port::Port0 => 0,
                crate::gpio::Port::Port1 => 1,
                crate::gpio::Port::Port2 => 2,
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
        let g = regs(self.ch.inst());
        g.tasks_out(self.ch.number()).write_value(1);
    }

    /// Triggers the SET task (set associated pin high).
    #[cfg(not(feature = "_nrf51"))]
    pub fn set(&self) {
        let g = regs(self.ch.inst());
        g.tasks_set(self.ch.number()).write_value(1);
    }

    /// Triggers the CLEAR task (set associated pin low).
    #[cfg(not(feature = "_nrf51"))]
    pub fn clear(&self) {
        let g = regs(self.ch.inst());
        g.tasks_clr(self.ch.number()).write_value(1);
    }

    /// Returns the OUT task, for use with PPI.
    pub fn task_out(&self) -> Task<'d> {
        let g = regs(self.ch.inst());
        Task::from_reg(g.tasks_out(self.ch.number()))
    }

    /// Returns the CLR task, for use with PPI.
    #[cfg(not(feature = "_nrf51"))]
    pub fn task_clr(&self) -> Task<'d> {
        let g = regs(self.ch.inst());
        Task::from_reg(g.tasks_clr(self.ch.number()))
    }

    /// Returns the SET task, for use with PPI.
    #[cfg(not(feature = "_nrf51"))]
    pub fn task_set(&self) -> Task<'d> {
        let g = regs(self.ch.inst());
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
        let pin: &AnyPin = &self.pin;
        let inst: GpioteInstanceType = pin.into();
        inst.port_waker(pin._pin() as usize).register(cx.waker());

        if pin.conf().read().sense() == Sense::DISABLED {
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
    /// Get the controller instance id.
    fn inst(&self) -> GpioteInstanceType;
}

/// Type-erased channel.
///
/// Obtained by calling `Channel::into()`.
///
/// This allows using several channels in situations that might require
/// them to be the same type, like putting them in an array.
pub struct AnyChannel {
    number: u8,
    inst: GpioteInstanceType,
}
impl_peripheral!(AnyChannel);
impl SealedChannel for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> usize {
        self.number as usize
    }
    fn inst(&self) -> GpioteInstanceType {
        self.inst
    }
}

macro_rules! impl_channel {
    ($type:ident, $inst:expr, $number:expr) => {
        impl SealedChannel for peripherals::$type {}
        impl Channel for peripherals::$type {
            fn number(&self) -> usize {
                $number as usize
            }
            fn inst(&self) -> GpioteInstanceType {
                $inst
            }
        }

        impl From<peripherals::$type> for AnyChannel {
            fn from(val: peripherals::$type) -> Self {
                Self {
                    number: val.number() as u8,
                    inst: val.inst(),
                }
            }
        }
    };
}

#[cfg(not(feature = "_nrf54l"))]
impl_channel!(GPIOTE_CH0, (), 0);
#[cfg(not(feature = "_nrf54l"))]
impl_channel!(GPIOTE_CH1, (), 1);
#[cfg(not(feature = "_nrf54l"))]
impl_channel!(GPIOTE_CH2, (), 2);
#[cfg(not(feature = "_nrf54l"))]
impl_channel!(GPIOTE_CH3, (), 3);
#[cfg(not(feature = "_nrf51"))]
#[cfg(not(feature = "_nrf54l"))]
impl_channel!(GPIOTE_CH4, (), 4);
#[cfg(not(feature = "_nrf51"))]
#[cfg(not(feature = "_nrf54l"))]
impl_channel!(GPIOTE_CH5, (), 5);
#[cfg(not(feature = "_nrf51"))]
#[cfg(not(feature = "_nrf54l"))]
impl_channel!(GPIOTE_CH6, (), 6);
#[cfg(not(feature = "_nrf51"))]
#[cfg(not(feature = "_nrf54l"))]
impl_channel!(GPIOTE_CH7, (), 7);

#[cfg(feature = "_nrf54l")]
impl_channel!(GPIOTE20_CH0, GpioteInstance::Gpiote20, 0);
#[cfg(feature = "_nrf54l")]
impl_channel!(GPIOTE20_CH1, GpioteInstance::Gpiote20, 1);
#[cfg(feature = "_nrf54l")]
impl_channel!(GPIOTE20_CH2, GpioteInstance::Gpiote20, 2);
#[cfg(feature = "_nrf54l")]
impl_channel!(GPIOTE20_CH3, GpioteInstance::Gpiote20, 3);
#[cfg(feature = "_nrf54l")]
impl_channel!(GPIOTE20_CH4, GpioteInstance::Gpiote20, 4);
#[cfg(feature = "_nrf54l")]
impl_channel!(GPIOTE20_CH5, GpioteInstance::Gpiote20, 5);
#[cfg(feature = "_nrf54l")]
impl_channel!(GPIOTE20_CH6, GpioteInstance::Gpiote20, 6);
#[cfg(feature = "_nrf54l")]
impl_channel!(GPIOTE20_CH7, GpioteInstance::Gpiote20, 7);

#[cfg(feature = "_nrf54l")]
impl_channel!(GPIOTE30_CH0, GpioteInstance::Gpiote30, 0);
#[cfg(feature = "_nrf54l")]
impl_channel!(GPIOTE30_CH1, GpioteInstance::Gpiote30, 1);
#[cfg(feature = "_nrf54l")]
impl_channel!(GPIOTE30_CH2, GpioteInstance::Gpiote30, 2);
#[cfg(feature = "_nrf54l")]
impl_channel!(GPIOTE30_CH3, GpioteInstance::Gpiote30, 3);

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
