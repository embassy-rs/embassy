//! GPIO driver.

#![macro_use]
use core::future::Future;
use core::pin::Pin as FuturePin;
use core::task::{Context, Poll};

use cortex_m::interrupt::InterruptNumber;
use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::InterruptExt;
use crate::pac::common::{R, RW, Reg};
use crate::pac::gpio::regs::{Ctrl1, Ctrl2};
use crate::{interrupt, pac, peripherals};

// Each interrupt aggregator block holds a maximum of 31 GPIOs. Some combine a
// considerably smaller number of pins, but we're still allocating 31
// `AtomicWaker`s for those.
pub(crate) const PIN_COUNT: usize = 31;

static GIRQ08_WAKERS: [AtomicWaker; PIN_COUNT] = [const { AtomicWaker::new() }; PIN_COUNT];
static GIRQ09_WAKERS: [AtomicWaker; PIN_COUNT] = [const { AtomicWaker::new() }; PIN_COUNT];
static GIRQ10_WAKERS: [AtomicWaker; PIN_COUNT] = [const { AtomicWaker::new() }; PIN_COUNT];
static GIRQ11_WAKERS: [AtomicWaker; PIN_COUNT] = [const { AtomicWaker::new() }; PIN_COUNT];
static GIRQ12_WAKERS: [AtomicWaker; PIN_COUNT] = [const { AtomicWaker::new() }; PIN_COUNT];
static GIRQ26_WAKERS: [AtomicWaker; PIN_COUNT] = [const { AtomicWaker::new() }; PIN_COUNT];

/// Represents a digital input or output level.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Logical low.
    Low,

    /// Logical high.
    High,
}

impl From<bool> for Level {
    fn from(val: bool) -> Self {
        match val {
            true => Self::High,
            false => Self::Low,
        }
    }
}

impl From<Level> for bool {
    fn from(level: Level) -> bool {
        match level {
            Level::High => true,
            Level::Low => false,
        }
    }
}

/// Represents a pull setting for an input.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Pull {
    /// No pull.
    None,

    /// Internal pull-up resistor.
    Up,

    /// Internal pull-down resistor.
    Down,

    /// Repeater mode. Pin is kept at previous voltage level when no
    /// active driver is present on the pin.
    Repeater,
}

impl From<Pull> for crate::pac::Pull {
    fn from(val: Pull) -> Self {
        match val {
            Pull::None => Self::NONE,
            Pull::Up => Self::UP,
            Pull::Down => Self::DOWN,
            Pull::Repeater => Self::REPEATER,
        }
    }
}

/// Drive strenght of an output.
#[derive(Debug, Eq, PartialEq)]
pub enum Drive {
    /// 2mA for PIO-12, 4mA for PIO-24,
    Weakest,

    /// 4mA for PIO-12, 8mA for PIO-24,
    Weak,

    /// 8mA for PIO-12, 16mA for PIO-24,
    Strong,

    /// 12mA for PIO-12, 24mA for PIO-24,
    Strongest,
}

impl From<Drive> for crate::pac::Strength {
    fn from(val: Drive) -> Self {
        match val {
            Drive::Weakest => Self::LOWEST,
            Drive::Weak => Self::LOW,
            Drive::Strong => Self::MEDIUM,
            Drive::Strongest => Self::FULL,
        }
    }
}

/// Slow rate of an output.
#[derive(Debug, Eq, PartialEq)]
pub enum SlewRate {
    /// Slow (half-frequency) slew rate.
    Slow,

    /// Fast slew rate.
    Fast,
}

impl From<SlewRate> for crate::pac::SlewCtrl {
    fn from(val: SlewRate) -> Self {
        match val {
            SlewRate::Slow => Self::SLOW,
            SlewRate::Fast => Self::FAST,
        }
    }
}

/// GPIO input driber.
pub struct Input<'d> {
    pin: Flex<'d>,
}

impl<'d> Input<'d> {
    /// Create a GPIO input driver for a [Pin] with the provided [Pull] configuration.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_input();
        pin.set_pull(pull);
        Self { pin }
    }

    /// Get wheter the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Returns the current pin level
    #[inline]
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await;
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await;
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await;
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await;
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await;
    }
}

/// Interrupt trigger levels.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InterruptTrigger {
    /// Trigger on pin low.
    LevelLow,

    /// Trigger on pin high.
    LevelHigh,

    /// Trigger on high to low transition.
    EdgeLow,

    /// Trigger on low to high transition.
    EdgeHigh,

    /// Trigger on any transition.
    AnyEdge,
}

pub unsafe fn init() {
    // GPIO interrupts must go through the Interrupt Aggregator.
    let blk_en = 1 << (8 + interrupt::GIRQ08.number())
        | 1 << (8 + interrupt::GIRQ09.number())
        | 1 << (8 + interrupt::GIRQ10.number())
        | 1 << (8 + interrupt::GIRQ11.number())
        | 1 << (8 + interrupt::GIRQ12.number())
        | 1 << (8 + interrupt::GIRQ26.number());
    crate::pac::ECIA.blk_en_set().write(|w| w.set_vtor_en_set(blk_en));

    interrupt::GIRQ08.disable();
    interrupt::GIRQ09.disable();
    interrupt::GIRQ10.disable();
    interrupt::GIRQ11.disable();
    interrupt::GIRQ12.disable();
    interrupt::GIRQ26.disable();

    interrupt::GIRQ08.set_priority(interrupt::Priority::P7);
    interrupt::GIRQ09.set_priority(interrupt::Priority::P7);
    interrupt::GIRQ10.set_priority(interrupt::Priority::P7);
    interrupt::GIRQ11.set_priority(interrupt::Priority::P7);
    interrupt::GIRQ12.set_priority(interrupt::Priority::P7);
    interrupt::GIRQ26.set_priority(interrupt::Priority::P7);

    interrupt::GIRQ08.unpend();
    interrupt::GIRQ09.unpend();
    interrupt::GIRQ10.unpend();
    interrupt::GIRQ11.unpend();
    interrupt::GIRQ12.unpend();
    interrupt::GIRQ26.unpend();

    unsafe {
        interrupt::GIRQ08.enable();
        interrupt::GIRQ09.enable();
        interrupt::GIRQ10.enable();
        interrupt::GIRQ11.enable();
        interrupt::GIRQ12.enable();
        interrupt::GIRQ26.enable();
    }
}

#[cfg(feature = "rt")]
#[interrupt]
fn GIRQ08() {
    let regs = InterruptRegs {
        result: crate::pac::ECIA.result8(),
        clr: crate::pac::ECIA.en_clr8(),
    };
    irq_handler(regs, &GIRQ08_WAKERS);
}

#[cfg(feature = "rt")]
#[interrupt]
fn GIRQ09() {
    let regs = InterruptRegs {
        result: crate::pac::ECIA.result9(),
        clr: crate::pac::ECIA.en_clr9(),
    };
    irq_handler(regs, &GIRQ09_WAKERS);
}

#[cfg(feature = "rt")]
#[interrupt]
fn GIRQ10() {
    let regs = InterruptRegs {
        result: crate::pac::ECIA.result10(),
        clr: crate::pac::ECIA.en_clr10(),
    };
    irq_handler(regs, &GIRQ10_WAKERS);
}

#[cfg(feature = "rt")]
#[interrupt]
fn GIRQ11() {
    let regs = InterruptRegs {
        result: crate::pac::ECIA.result11(),
        clr: crate::pac::ECIA.en_clr11(),
    };
    irq_handler(regs, &GIRQ11_WAKERS);
}

#[cfg(feature = "rt")]
#[interrupt]
fn GIRQ12() {
    let regs = InterruptRegs {
        result: crate::pac::ECIA.result12(),
        clr: crate::pac::ECIA.en_clr12(),
    };
    irq_handler(regs, &GIRQ12_WAKERS);
}

#[cfg(feature = "rt")]
#[interrupt]
fn GIRQ26() {
    let regs = InterruptRegs {
        result: crate::pac::ECIA.result26(),
        clr: crate::pac::ECIA.en_clr26(),
    };
    irq_handler(regs, &GIRQ26_WAKERS);
}

#[cfg(feature = "rt")]
#[inline]
fn irq_handler(regs: InterruptRegs, wakers: &[AtomicWaker; PIN_COUNT]) {
    let result = regs.result.read();

    for bit in 0..PIN_COUNT {
        if result & (1 << bit) != 0 {
            // mask event
            regs.clr.write_value(1 << bit);
            wakers[bit].wake();
        }
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct InputFuture<'d> {
    pin: Peri<'d, AnyPin>,
    trigger: InterruptTrigger,
}

impl<'d> InputFuture<'d> {
    fn new(pin: Peri<'d, AnyPin>, trigger: InterruptTrigger) -> Self {
        pin.enable_interrupts(trigger);
        Self { pin, trigger }
    }
}

impl<'d> Unpin for InputFuture<'d> {}
impl<'d> Future for InputFuture<'d> {
    type Output = ();

    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = self.pin.waker();
        waker.register(cx.waker());

        // If user requested a level change, then we must check the
        // current level of the pin. If it matches the requested
        // level, we're done. Just produce a `Poll::Ready`. If,
        // however, an edge change was requested, we must wait for the
        // actual interrupt.
        let level_change = match self.trigger {
            InterruptTrigger::LevelLow => !self.pin.regs().ctrl1.read().inp(),
            InterruptTrigger::LevelHigh => self.pin.regs().ctrl1.read().inp(),
            _ => false,
        };

        // IRQ handler will mask the interrupt if the event has occurred. If the
        // bit for $this event is cleared, that means we triggered an interrupt
        // and an return control to the application.
        let set = self.pin.regs().set.read();

        if level_change || (set & (1 << self.pin.irq_bit()) == 0) {
            Poll::Ready(())
        } else {
            // unmask event so other events can be sampled
            self.pin.regs().set.write_value(1 << self.pin.irq_bit());
            Poll::Pending
        }
    }
}

impl<'d> Drop for InputFuture<'d> {
    fn drop(&mut self) {
        self.pin.disable_interrupts();
    }
}

/// GPIO output driver.
pub struct Output<'d> {
    pin: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Create GPIO output driver for a [Pin] with the provided [Level].
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);

        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }

        pin.set_as_output();

        Self { pin }
    }

    /// Set the pin's drive strength.
    #[inline]
    pub fn set_drive_strength(&mut self, strength: Drive) {
        self.pin.set_drive_strength(strength)
    }

    /// Set the pin's slew rate.
    #[inline]
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        self.pin.set_slew_rate(slew_rate)
    }

    /// Set the outpt as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high()
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low()
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level)
    }

    /// Is the outpt pin set as high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.is_set_high()
    }

    /// Is the output pin set as low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_low()
    }

    /// What level output is set to
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.pin.get_output_level()
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle()
    }
}

/// GPIO output open-drain.
pub struct OutputOpenDrain<'d> {
    pin: Flex<'d>,
}

impl<'d> OutputOpenDrain<'d> {
    /// Create GPIO output driver for a [Pin] in open drain mode with the
    /// provided [Level].
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);

        pin.set_as_output_open_drain();
        pin.set_level(initial_output);

        Self { pin }
    }

    /// Set the pin's pull-up.
    #[inline]
    pub fn set_pullup(&mut self, enable: bool) {
        if enable {
            self.pin.set_pull(Pull::Up)
        } else {
            self.pin.set_pull(Pull::None)
        }
    }

    /// Set the pin's drive strength.
    #[inline]
    pub fn set_drive_strength(&mut self, strength: Drive) {
        self.pin.set_drive_strength(strength)
    }

    /// Set the pin's slew rate.
    #[inline]
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        self.pin.set_slew_rate(slew_rate)
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high()
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low()
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level)
    }

    /// Is the output level high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    /// Is the output level low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_as_output()
    }

    /// What level output is set to
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.is_set_high().into()
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle()
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Returns current pin level
    #[inline]
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await;
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await;
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await;
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await;
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to
    /// low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await;
    }
}

/// GPIO flexible pin.
///
/// This pin can be either an input or output pin. The output level register bit
/// will remain set while not in output mode, so the pin's level will be
/// 'remembered' when it is not in output mode.
pub struct Flex<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> Flex<'d> {
    /// Wrap the pin in a `Flex`.
    ///
    /// The pin remains disconnected. The initial output level is unspecified,
    /// but can be changed before the pin is put into output mode.
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        critical_section::with(|_| {
            pin.regs().ctrl1.modify(|w| {
                w.set_mux_ctrl(crate::pac::Function::GPIO);
                w.set_out_sel(crate::pac::Sel::PIN);
            })
        });

        let pin = pin.into();
        pin.disable_interrupts();

        Self { pin }
    }

    /// Set the pin's pull.
    #[inline]
    pub fn set_pull(&mut self, pull: Pull) {
        critical_section::with(|_| {
            self.pin.regs().ctrl1.modify(|w| w.set_pu_pd(pull.into()));
        });
    }

    /// Set the pin's drive stength
    #[inline]
    pub fn set_drive_strength(&mut self, strength: Drive) {
        critical_section::with(|_| {
            self.pin.regs().ctrl2.modify(|w| w.set_driv_stren(strength.into()));
        });
    }

    /// Set the pin's slew rate.
    #[inline]
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        critical_section::with(|_| {
            self.pin.regs().ctrl2.modify(|w| w.set_slew_ctrl(slew_rate.into()));
        });
    }

    /// Put the pin into input mode.
    ///
    /// The pull setting is left unchanged.
    #[inline]
    pub fn set_as_input(&mut self) {
        critical_section::with(|_| {
            self.pin.regs().ctrl1.modify(|w| {
                w.set_dir(crate::pac::Dir::INPUT);
                w.set_inp_dis(false);
            })
        });
    }

    /// Put the pin into output mode.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_output(&mut self) {
        critical_section::with(|_| self.pin.regs().ctrl1.modify(|w| w.set_dir(crate::pac::Dir::OUTPUT)));
    }

    /// Put the pin into output open drain mode.
    ///
    /// The pin level will be whatever was set before (or low by
    /// default). If you want it to begin at a specific level, call
    /// `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_output_open_drain(&mut self) {
        critical_section::with(|_| {
            self.pin.regs().ctrl1.modify(|w| {
                w.set_dir(crate::pac::Dir::OUTPUT);
                w.set_out_buff_type(crate::pac::BufferType::OPEN_DRAIN);
            })
        });
    }

    /// Set as output pin.
    #[inline]
    fn is_set_as_output(&self) -> bool {
        self.pin.regs().ctrl1.read().dir() == crate::pac::Dir::OUTPUT
    }

    /// Get whether the pin input level is high.
    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.regs().ctrl1.read().inp()
    }

    /// Get whether the pin input level is low.
    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Returns current pin level
    #[inline]
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        critical_section::with(|_| {
            self.pin.regs().ctrl1.modify(|w| w.set_alt_data(true));
        });
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        critical_section::with(|_| {
            self.pin.regs().ctrl1.modify(|w| w.set_alt_data(false));
        });
    }

    /// Set the output level.
    #[inline]
    pub fn set_level(&mut self, level: Level) {
        match level {
            Level::Low => self.set_low(),
            Level::High => self.set_high(),
        }
    }

    /// Is the output level high?
    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.regs().ctrl1.read().alt_data()
    }

    /// Is the output level low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    /// What level output is set to
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.is_set_high().into()
    }

    /// Toggle the output pin
    #[inline]
    pub fn toggle(&mut self) {
        critical_section::with(|_| {
            let val = self.pin.regs().ctrl1.read().alt_data();
            self.pin.regs().ctrl1.modify(|w| w.set_alt_data(!val));
        });
    }

    /// Wait until the pin is high. If it is already high, return immediately.
    #[inline]
    pub async fn wait_for_high(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptTrigger::LevelHigh).await;
    }

    /// Wait until the pin is low. If it is already low, return immediately.
    #[inline]
    pub async fn wait_for_low(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptTrigger::LevelLow).await;
    }

    /// Wait for the pin to undergo a transition from low to high.
    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptTrigger::EdgeHigh).await;
    }

    /// Wait for the pin to undergo a transition from high to low.
    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptTrigger::EdgeLow).await;
    }

    /// Wait for the pin to undergo any transition, i.e low to high OR high to low.
    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        InputFuture::new(self.pin.reborrow(), InterruptTrigger::AnyEdge).await;
    }
}

impl<'d> Drop for Flex<'d> {
    #[inline]
    fn drop(&mut self) {
        todo!()
    }
}

pub(crate) trait SealedPin: Sized {
    fn _pin(&self) -> u8;
    fn _port(&self) -> u8;
    fn regs(&self) -> Registers;
    fn irq_id(&self) -> usize;
    fn irq_bit(&self) -> usize;

    fn waker(&self) -> &AtomicWaker {
        match self.irq_id() {
            0 => &GIRQ08_WAKERS[self.irq_bit()],
            1 => &GIRQ09_WAKERS[self.irq_bit()],
            2 => &GIRQ10_WAKERS[self.irq_bit()],
            3 => &GIRQ11_WAKERS[self.irq_bit()],
            4 => &GIRQ12_WAKERS[self.irq_bit()],
            17 => &GIRQ26_WAKERS[self.irq_bit()],
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
struct InterruptRegs {
    result: Reg<u32, R>,
    clr: Reg<u32, RW>,
}

#[derive(Clone, Copy)]
pub(crate) struct Registers {
    pub(crate) ctrl1: Reg<Ctrl1, RW>,
    pub(crate) ctrl2: Reg<Ctrl2, RW>,
    pub(crate) src: Reg<u32, RW>,
    pub(crate) set: Reg<u32, RW>,
    pub(crate) clr: Reg<u32, RW>,
}

/// Interface for a [Pin] that can be configured by an [Input] or [Output]
/// driver, or converted to an [AnyPin].
#[allow(private_bounds)]
pub trait Pin: PeripheralType + Into<AnyPin> + SealedPin + Sized + 'static {
    /// Returns the pin number within a port
    #[inline]
    fn pin(&self) -> u8 {
        self._pin()
    }

    #[inline]
    fn port(&self) -> u8 {
        self._port()
    }
}

/// Type-erased GPIO pin
pub struct AnyPin {
    pin: u8,
    port: u8,
    regs: Registers,
    irq_id: usize,
    irq_bit: usize,
}

impl AnyPin {
    fn enable_interrupts(&self, trigger: InterruptTrigger) {
        critical_section::with(|_| {
            self.regs().clr.write_value(1 << self.irq_bit);

            self.regs().ctrl1.modify(|w| match trigger {
                InterruptTrigger::LevelLow => {
                    w.set_edge_en(false);
                    w.set_intr_det(0);
                }
                InterruptTrigger::LevelHigh => {
                    w.set_edge_en(false);
                    w.set_intr_det(1);
                }
                InterruptTrigger::EdgeLow => {
                    w.set_edge_en(true);
                    w.set_intr_det(6);
                }
                InterruptTrigger::EdgeHigh => {
                    w.set_edge_en(true);
                    w.set_intr_det(5);
                }
                InterruptTrigger::AnyEdge => {
                    w.set_edge_en(true);
                    w.set_intr_det(7);
                }
            });

            self.regs().src.write_value(1 << self.irq_bit);
            self.regs().set.write_value(1 << self.irq_bit);
        });
    }

    fn disable_interrupts(&self) {
        critical_section::with(|_| {
            self.regs().ctrl1.modify(|w| {
                w.set_edge_en(false);
                w.set_intr_det(4);
            });

            self.regs().clr.write_value(1 << self.irq_bit);
        });
    }
}

impl_peripheral!(AnyPin);

impl Pin for AnyPin {}
impl SealedPin for AnyPin {
    #[inline]
    fn _pin(&self) -> u8 {
        self.pin
    }

    #[inline]
    fn _port(&self) -> u8 {
        self.port
    }

    #[inline]
    fn regs(&self) -> Registers {
        self.regs
    }

    #[inline]
    fn irq_id(&self) -> usize {
        self.irq_id
    }

    #[inline]
    fn irq_bit(&self) -> usize {
        self.irq_bit
    }
}

macro_rules! impl_pin {
    ($name:ident, $port:expr, $pin:expr, $irq_id:expr, $irq_bit:expr, $src:ident, $set:ident, $result:ident, $clr:ident) => {
        impl Pin for peripherals::$name {}
        impl SealedPin for peripherals::$name {
            #[inline]
            fn _pin(&self) -> u8 {
                $pin
            }

            #[inline]
            fn _port(&self) -> u8 {
                $port
            }

            #[inline]
            fn irq_id(&self) -> usize {
                $irq_id
            }

            #[inline]
            fn irq_bit(&self) -> usize {
                $irq_bit
            }

            #[inline]
            fn regs(&self) -> Registers {
                let ptr = pac::GPIO.as_ptr();
                let ctrl1 =
                    unsafe { Reg::from_ptr(ptr.byte_add((($port / 10) << 8) + (($port % 10) << 5) + ($pin * 4)) as _) };
                let ctrl2 = unsafe {
                    Reg::from_ptr(ptr.byte_add(0x500 + (($port / 10) << 8) + (($port % 10) << 5) + ($pin * 4)) as _)
                };

                Registers {
                    ctrl1,
                    ctrl2,
                    src: crate::pac::ECIA.$src(),
                    set: crate::pac::ECIA.$set(),
                    clr: crate::pac::ECIA.$clr(),
                }
            }
        }

        impl From<peripherals::$name> for AnyPin {
            fn from(val: peripherals::$name) -> Self {
                Self {
                    pin: val._pin(),
                    port: val._port(),
                    regs: val.regs(),
                    irq_bit: val.irq_bit(),
                    irq_id: val.irq_id(),
                }
            }
        }
    };
}

impl_pin!(GPIO0, 0, 0, 3, 0, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO1, 0, 1, 3, 1, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO2, 0, 2, 3, 2, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO3, 0, 3, 3, 3, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO4, 0, 4, 3, 4, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO5, 0, 5, 3, 5, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO6, 0, 6, 3, 6, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO7, 0, 7, 3, 7, src11, en_set11, result11, en_clr11);

impl_pin!(GPIO10, 1, 0, 3, 8, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO11, 1, 1, 3, 9, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO12, 1, 2, 3, 10, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO13, 1, 3, 3, 11, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO14, 1, 4, 3, 12, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO15, 1, 5, 3, 13, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO16, 1, 6, 3, 14, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO17, 1, 7, 3, 15, src11, en_set11, result11, en_clr11);

impl_pin!(GPIO20, 2, 0, 3, 16, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO21, 2, 1, 3, 17, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO22, 2, 2, 3, 18, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO23, 2, 3, 3, 19, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO24, 2, 4, 3, 20, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO25, 2, 5, 3, 21, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO26, 2, 6, 3, 22, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO27, 2, 7, 3, 23, src11, en_set11, result11, en_clr11);

impl_pin!(GPIO30, 3, 0, 3, 24, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO31, 3, 1, 3, 25, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO32, 3, 2, 3, 26, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO33, 3, 3, 3, 27, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO34, 3, 4, 3, 28, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO35, 3, 5, 3, 29, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO36, 3, 6, 3, 30, src11, en_set11, result11, en_clr11);

impl_pin!(GPIO40, 4, 0, 2, 0, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO41, 4, 1, 2, 1, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO42, 4, 2, 2, 2, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO43, 4, 3, 2, 3, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO44, 4, 4, 2, 4, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO45, 4, 5, 2, 5, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO46, 4, 6, 2, 6, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO47, 4, 7, 2, 7, src10, en_set10, result10, en_clr10);

impl_pin!(GPIO50, 5, 0, 2, 8, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO51, 5, 1, 2, 9, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO52, 5, 2, 2, 10, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO53, 5, 3, 2, 11, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO54, 5, 4, 2, 12, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO55, 5, 5, 2, 13, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO56, 5, 6, 2, 14, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO57, 5, 7, 2, 15, src10, en_set10, result10, en_clr10);

impl_pin!(GPIO60, 6, 0, 2, 16, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO61, 6, 1, 2, 17, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO62, 6, 2, 2, 18, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO63, 6, 3, 2, 19, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO64, 6, 4, 2, 20, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO65, 6, 5, 2, 21, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO66, 6, 6, 2, 22, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO67, 6, 7, 2, 23, src10, en_set10, result10, en_clr10);

impl_pin!(GPIO70, 7, 0, 2, 24, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO71, 7, 1, 2, 25, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO72, 7, 2, 2, 26, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO73, 7, 3, 2, 27, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO74, 7, 4, 2, 28, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO75, 7, 5, 2, 29, src10, en_set10, result10, en_clr10);
impl_pin!(GPIO76, 7, 6, 2, 30, src10, en_set10, result10, en_clr10);

impl_pin!(GPIO100, 10, 0, 1, 0, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO101, 10, 1, 1, 1, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO102, 10, 2, 1, 2, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO103, 10, 3, 1, 3, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO104, 10, 4, 1, 4, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO105, 10, 5, 1, 5, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO106, 10, 6, 1, 6, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO107, 10, 7, 1, 7, src9, en_set9, result9, en_clr9);

impl_pin!(GPIO112, 11, 2, 1, 10, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO113, 11, 3, 1, 11, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO114, 11, 4, 1, 12, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO115, 11, 5, 1, 13, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO116, 11, 6, 1, 14, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO117, 11, 7, 1, 15, src9, en_set9, result9, en_clr9);

impl_pin!(GPIO120, 12, 0, 1, 16, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO121, 12, 1, 1, 17, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO122, 12, 2, 1, 18, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO123, 12, 3, 1, 19, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO124, 12, 4, 1, 20, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO125, 12, 5, 1, 21, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO126, 12, 6, 1, 22, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO127, 12, 7, 1, 23, src9, en_set9, result9, en_clr9);

impl_pin!(GPIO130, 13, 0, 1, 24, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO131, 13, 1, 1, 25, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO132, 13, 2, 1, 26, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO133, 13, 3, 1, 27, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO134, 13, 4, 1, 28, src9, en_set9, result9, en_clr9);
impl_pin!(GPIO135, 13, 5, 1, 29, src9, en_set9, result9, en_clr9);

impl_pin!(GPIO140, 14, 0, 0, 0, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO141, 14, 1, 0, 1, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO142, 14, 2, 0, 2, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO143, 14, 3, 0, 3, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO144, 14, 4, 0, 4, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO145, 14, 5, 0, 5, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO146, 14, 6, 0, 6, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO147, 14, 7, 0, 7, src8, en_set8, result8, en_clr8);

impl_pin!(GPIO150, 15, 0, 0, 8, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO151, 15, 1, 0, 9, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO152, 15, 2, 0, 10, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO153, 15, 3, 0, 11, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO154, 15, 4, 0, 12, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO155, 15, 5, 0, 13, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO156, 15, 6, 0, 14, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO157, 15, 7, 0, 15, src8, en_set8, result8, en_clr8);

impl_pin!(GPIO160, 16, 0, 0, 16, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO161, 16, 1, 0, 17, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO162, 16, 2, 0, 18, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO165, 16, 5, 0, 21, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO166, 16, 6, 0, 22, src8, en_set8, result8, en_clr8);

impl_pin!(GPIO170, 17, 0, 0, 24, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO171, 17, 1, 0, 25, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO172, 17, 2, 0, 26, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO173, 17, 3, 0, 27, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO174, 17, 4, 0, 28, src8, en_set8, result8, en_clr8);
impl_pin!(GPIO175, 17, 5, 0, 29, src8, en_set8, result8, en_clr8);

impl_pin!(GPIO200, 20, 0, 4, 0, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO201, 20, 1, 4, 1, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO202, 20, 2, 4, 2, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO203, 20, 3, 4, 3, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO204, 20, 4, 4, 4, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO205, 20, 5, 4, 5, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO206, 20, 6, 4, 6, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO207, 20, 7, 4, 7, src12, en_set12, result12, en_clr12);

impl_pin!(GPIO210, 21, 0, 4, 8, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO211, 21, 1, 4, 9, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO212, 21, 2, 4, 10, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO213, 21, 3, 4, 11, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO214, 21, 4, 4, 12, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO215, 21, 5, 4, 13, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO216, 21, 6, 4, 14, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO217, 21, 7, 4, 15, src12, en_set12, result12, en_clr12);

impl_pin!(GPIO221, 22, 1, 4, 17, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO222, 22, 2, 4, 18, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO223, 22, 3, 4, 19, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO224, 22, 4, 4, 20, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO225, 22, 5, 4, 21, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO226, 22, 6, 4, 22, src12, en_set12, result12, en_clr12);
impl_pin!(GPIO227, 22, 7, 4, 23, src12, en_set12, result12, en_clr12);

impl_pin!(GPIO230, 23, 0, 4, 23, src11, en_set11, result11, en_clr11);
impl_pin!(GPIO231, 23, 0, 4, 23, src11, en_set11, result11, en_clr11);

impl_pin!(GPIO240, 24, 0, 17, 0, src26, en_set26, result26, en_clr26);
impl_pin!(GPIO241, 24, 1, 17, 1, src26, en_set26, result26, en_clr26);
impl_pin!(GPIO242, 24, 2, 17, 2, src26, en_set26, result26, en_clr26);
impl_pin!(GPIO243, 24, 3, 17, 3, src26, en_set26, result26, en_clr26);
impl_pin!(GPIO244, 24, 4, 17, 4, src26, en_set26, result26, en_clr26);
impl_pin!(GPIO245, 24, 5, 17, 5, src26, en_set26, result26, en_clr26);
impl_pin!(GPIO246, 24, 6, 17, 6, src26, en_set26, result26, en_clr26);
impl_pin!(GPIO247, 24, 7, 17, 7, src26, en_set26, result26, en_clr26);

impl_pin!(GPIO254, 25, 4, 17, 12, src26, en_set26, result26, en_clr26);
impl_pin!(GPIO255, 25, 5, 17, 13, src26, en_set26, result26, en_clr26);
