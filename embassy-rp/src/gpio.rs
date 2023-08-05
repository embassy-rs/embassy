#![macro_use]
use core::future::Future;
use core::pin::Pin as FuturePin;
use core::task::{Context, Poll};

use embassy_hal_internal::{impl_peripheral, into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::InterruptExt;
use crate::pac::common::{Reg, RW};
use crate::pac::SIO;
use crate::{interrupt, pac, peripherals, Peripheral, RegExt};

const NEW_AW: AtomicWaker = AtomicWaker::new();
const BANK0_PIN_COUNT: usize = 30;
static BANK0_WAKERS: [AtomicWaker; BANK0_PIN_COUNT] = [NEW_AW; BANK0_PIN_COUNT];
#[cfg(feature = "qspi-as-gpio")]
const QSPI_PIN_COUNT: usize = 6;
#[cfg(feature = "qspi-as-gpio")]
static QSPI_WAKERS: [AtomicWaker; QSPI_PIN_COUNT] = [NEW_AW; QSPI_PIN_COUNT];

/// Represents a digital input or output level.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Level {
    Low,
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
            Level::Low => false,
            Level::High => true,
        }
    }
}

/// Represents a pull setting for an input.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Pull {
    None,
    Up,
    Down,
}

/// Drive strength of an output
#[derive(Debug, Eq, PartialEq)]
pub enum Drive {
    _2mA,
    _4mA,
    _8mA,
    _12mA,
}
/// Slew rate of an output
#[derive(Debug, Eq, PartialEq)]
pub enum SlewRate {
    Fast,
    Slow,
}

/// A GPIO bank with up to 32 pins.
#[derive(Debug, Eq, PartialEq)]
pub enum Bank {
    Bank0 = 0,
    #[cfg(feature = "qspi-as-gpio")]
    Qspi = 1,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DormantWakeConfig {
    pub edge_high: bool,
    pub edge_low: bool,
    pub level_high: bool,
    pub level_low: bool,
}

pub struct Input<'d, T: Pin> {
    pin: Flex<'d, T>,
}

impl<'d, T: Pin> Input<'d, T> {
    #[inline]
    pub fn new(pin: impl Peripheral<P = T> + 'd, pull: Pull) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_as_input();
        pin.set_pull(pull);
        Self { pin }
    }

    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Returns current pin level
    #[inline]
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
    }

    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await;
    }

    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await;
    }

    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await;
    }

    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await;
    }

    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await;
    }

    #[inline]
    pub fn dormant_wake(&mut self, cfg: DormantWakeConfig) -> DormantWake<T> {
        self.pin.dormant_wake(cfg)
    }
}

/// Interrupt trigger levels.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InterruptTrigger {
    LevelLow,
    LevelHigh,
    EdgeLow,
    EdgeHigh,
    AnyEdge,
}

pub(crate) unsafe fn init() {
    interrupt::IO_IRQ_BANK0.disable();
    interrupt::IO_IRQ_BANK0.set_priority(interrupt::Priority::P3);
    interrupt::IO_IRQ_BANK0.enable();

    #[cfg(feature = "qspi-as-gpio")]
    {
        interrupt::IO_IRQ_QSPI.disable();
        interrupt::IO_IRQ_QSPI.set_priority(interrupt::Priority::P3);
        interrupt::IO_IRQ_QSPI.enable();
    }
}

#[cfg(feature = "rt")]
fn irq_handler<const N: usize>(bank: pac::io::Io, wakers: &[AtomicWaker; N]) {
    let cpu = SIO.cpuid().read() as usize;
    // There are two sets of interrupt registers, one for cpu0 and one for cpu1
    // and here we are selecting the set that belongs to the currently executing
    // cpu.
    let proc_intx: pac::io::Int = bank.int_proc(cpu);
    for pin in 0..N {
        // There are 4 raw interrupt status registers, PROCx_INTS0, PROCx_INTS1,
        // PROCx_INTS2, and PROCx_INTS3, and we are selecting the one that the
        // current pin belongs to.
        let intsx = proc_intx.ints(pin / 8);
        // The status register is divided into groups of four, one group for
        // each pin. Each group consists of four trigger levels LEVEL_LOW,
        // LEVEL_HIGH, EDGE_LOW, and EDGE_HIGH for each pin.
        let pin_group = (pin % 8) as usize;
        let event = (intsx.read().0 >> pin_group * 4) & 0xf as u32;

        // no more than one event can be awaited per pin at any given time, so
        // we can just clear all interrupt enables for that pin without having
        // to check which event was signalled.
        if event != 0 {
            proc_intx.inte(pin / 8).write_clear(|w| {
                w.set_edge_high(pin_group, true);
                w.set_edge_low(pin_group, true);
                w.set_level_high(pin_group, true);
                w.set_level_low(pin_group, true);
            });
            wakers[pin as usize].wake();
        }
    }
}

#[cfg(feature = "rt")]
#[interrupt]
fn IO_IRQ_BANK0() {
    irq_handler(pac::IO_BANK0, &BANK0_WAKERS);
}

#[cfg(all(feature = "rt", feature = "qspi-as-gpio"))]
#[interrupt]
fn IO_IRQ_QSPI() {
    irq_handler(pac::IO_QSPI, &QSPI_WAKERS);
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct InputFuture<'a, T: Pin> {
    pin: PeripheralRef<'a, T>,
    level: InterruptTrigger,
}

impl<'d, T: Pin> InputFuture<'d, T> {
    pub fn new(pin: impl Peripheral<P = T> + 'd, level: InterruptTrigger) -> Self {
        into_ref!(pin);
        let pin_group = (pin.pin() % 8) as usize;
        // first, clear the INTR register bits. without this INTR will still
        // contain reports of previous edges, causing the IRQ to fire early
        // on stale state. clearing these means that we can only detect edges
        // that occur *after* the clear happened, but since both this and the
        // alternative are fundamentally racy it's probably fine.
        // (the alternative being checking the current level and waiting for
        // its inverse, but that requires reading the current level and thus
        // missing anything that happened before the level was read.)
        pin.io().intr(pin.pin() as usize / 8).write(|w| {
            w.set_edge_high(pin_group, true);
            w.set_edge_low(pin_group, true);
        });

        // Each INTR register is divided into 8 groups, one group for each
        // pin, and each group consists of LEVEL_LOW, LEVEL_HIGH, EDGE_LOW,
        // and EGDE_HIGH.
        pin.int_proc()
            .inte((pin.pin() / 8) as usize)
            .write_set(|w| match level {
                InterruptTrigger::LevelHigh => {
                    trace!("InputFuture::new enable LevelHigh for pin {}", pin.pin());
                    w.set_level_high(pin_group, true);
                }
                InterruptTrigger::LevelLow => {
                    w.set_level_low(pin_group, true);
                }
                InterruptTrigger::EdgeHigh => {
                    w.set_edge_high(pin_group, true);
                }
                InterruptTrigger::EdgeLow => {
                    w.set_edge_low(pin_group, true);
                }
                InterruptTrigger::AnyEdge => {
                    w.set_edge_high(pin_group, true);
                    w.set_edge_low(pin_group, true);
                }
            });

        Self { pin, level }
    }
}

impl<'d, T: Pin> Future for InputFuture<'d, T> {
    type Output = ();

    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // We need to register/re-register the waker for each poll because any
        // calls to wake will deregister the waker.
        let waker = match self.pin.bank() {
            Bank::Bank0 => &BANK0_WAKERS[self.pin.pin() as usize],
            #[cfg(feature = "qspi-as-gpio")]
            Bank::Qspi => &QSPI_WAKERS[self.pin.pin() as usize],
        };
        waker.register(cx.waker());

        // self.int_proc() will get the register offset for the current cpu,
        // then we want to access the interrupt enable register for our
        // pin (there are 4 of these PROC0_INTE0, PROC0_INTE1, PROC0_INTE2, and
        // PROC0_INTE3 per cpu).
        let inte: pac::io::regs::Int = self.pin.int_proc().inte((self.pin.pin() / 8) as usize).read();
        // The register is divided into groups of four, one group for
        // each pin. Each group consists of four trigger levels LEVEL_LOW,
        // LEVEL_HIGH, EDGE_LOW, and EDGE_HIGH for each pin.
        let pin_group = (self.pin.pin() % 8) as usize;

        // since the interrupt handler clears all INTE flags we'll check that
        // all have been cleared and unconditionally return Ready(()) if so.
        // we don't need further handshaking since only a single event wait
        // is possible for any given pin at any given time.
        if !inte.edge_high(pin_group)
            && !inte.edge_low(pin_group)
            && !inte.level_high(pin_group)
            && !inte.level_low(pin_group)
        {
            trace!(
                "{:?} for pin {} was cleared, return Poll::Ready",
                self.level,
                self.pin.pin()
            );
            return Poll::Ready(());
        }
        trace!("InputFuture::poll return Poll::Pending");
        Poll::Pending
    }
}

pub struct Output<'d, T: Pin> {
    pin: Flex<'d, T>,
}

impl<'d, T: Pin> Output<'d, T> {
    #[inline]
    pub fn new(pin: impl Peripheral<P = T> + 'd, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
        match initial_output {
            Level::High => pin.set_high(),
            Level::Low => pin.set_low(),
        }

        pin.set_as_output();
        Self { pin }
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

    /// Is the output pin set as high?
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
pub struct OutputOpenDrain<'d, T: Pin> {
    pin: Flex<'d, T>,
}

impl<'d, T: Pin> OutputOpenDrain<'d, T> {
    #[inline]
    pub fn new(pin: impl Peripheral<P = T> + 'd, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_low();
        match initial_output {
            Level::High => pin.set_as_input(),
            Level::Low => pin.set_as_output(),
        }
        Self { pin }
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        // For Open Drain High, disable the output pin.
        self.pin.set_as_input()
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        // For Open Drain Low, enable the output pin.
        self.pin.set_as_output()
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
        self.pin.toggle_set_as_output()
    }

    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Returns current pin level
    #[inline]
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }

    #[inline]
    pub async fn wait_for_high(&mut self) {
        self.pin.wait_for_high().await;
    }

    #[inline]
    pub async fn wait_for_low(&mut self) {
        self.pin.wait_for_low().await;
    }

    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        self.pin.wait_for_rising_edge().await;
    }

    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        self.pin.wait_for_falling_edge().await;
    }

    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        self.pin.wait_for_any_edge().await;
    }
}

/// GPIO flexible pin.
///
/// This pin can be either an input or output pin. The output level register bit will remain
/// set while not in output mode, so the pin's level will be 'remembered' when it is not in output
/// mode.
pub struct Flex<'d, T: Pin> {
    pin: PeripheralRef<'d, T>,
}

impl<'d, T: Pin> Flex<'d, T> {
    #[inline]
    pub fn new(pin: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(pin);

        pin.pad_ctrl().write(|w| {
            w.set_ie(true);
        });

        pin.gpio().ctrl().write(|w| {
            w.set_funcsel(pac::io::vals::Gpio0ctrlFuncsel::SIO_0 as _);
        });

        Self { pin }
    }

    #[inline]
    fn bit(&self) -> u32 {
        1 << self.pin.pin()
    }

    /// Set the pin's pull.
    #[inline]
    pub fn set_pull(&mut self, pull: Pull) {
        self.pin.pad_ctrl().modify(|w| {
            w.set_ie(true);
            let (pu, pd) = match pull {
                Pull::Up => (true, false),
                Pull::Down => (false, true),
                Pull::None => (false, false),
            };
            w.set_pue(pu);
            w.set_pde(pd);
        });
    }

    /// Set the pin's drive strength.
    #[inline]
    pub fn set_drive_strength(&mut self, strength: Drive) {
        self.pin.pad_ctrl().modify(|w| {
            w.set_drive(match strength {
                Drive::_2mA => pac::pads::vals::Drive::_2MA,
                Drive::_4mA => pac::pads::vals::Drive::_4MA,
                Drive::_8mA => pac::pads::vals::Drive::_8MA,
                Drive::_12mA => pac::pads::vals::Drive::_12MA,
            });
        });
    }

    // Set the pin's slew rate.
    #[inline]
    pub fn set_slew_rate(&mut self, slew_rate: SlewRate) {
        self.pin.pad_ctrl().modify(|w| {
            w.set_slewfast(slew_rate == SlewRate::Fast);
        });
    }

    /// Put the pin into input mode.
    ///
    /// The pull setting is left unchanged.
    #[inline]
    pub fn set_as_input(&mut self) {
        self.pin.sio_oe().value_clr().write_value(self.bit())
    }

    /// Put the pin into output mode.
    ///
    /// The pin level will be whatever was set before (or low by default). If you want it to begin
    /// at a specific level, call `set_high`/`set_low` on the pin first.
    #[inline]
    pub fn set_as_output(&mut self) {
        self.pin.sio_oe().value_set().write_value(self.bit())
    }

    #[inline]
    fn is_set_as_output(&self) -> bool {
        (self.pin.sio_oe().value().read() & self.bit()) != 0
    }

    #[inline]
    pub fn toggle_set_as_output(&mut self) {
        self.pin.sio_oe().value_xor().write_value(self.bit())
    }

    #[inline]
    pub fn is_high(&self) -> bool {
        !self.is_low()
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.sio_in().read() & self.bit() == 0
    }

    /// Returns current pin level
    #[inline]
    pub fn get_level(&self) -> Level {
        self.is_high().into()
    }

    /// Set the output as high.
    #[inline]
    pub fn set_high(&mut self) {
        self.pin.sio_out().value_set().write_value(self.bit())
    }

    /// Set the output as low.
    #[inline]
    pub fn set_low(&mut self) {
        self.pin.sio_out().value_clr().write_value(self.bit())
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
        !self.is_set_low()
    }

    /// Is the output level low?
    #[inline]
    pub fn is_set_low(&self) -> bool {
        (self.pin.sio_out().value().read() & self.bit()) == 0
    }

    /// What level output is set to
    #[inline]
    pub fn get_output_level(&self) -> Level {
        self.is_set_high().into()
    }

    /// Toggle pin output
    #[inline]
    pub fn toggle(&mut self) {
        self.pin.sio_out().value_xor().write_value(self.bit())
    }

    #[inline]
    pub async fn wait_for_high(&mut self) {
        InputFuture::new(&mut self.pin, InterruptTrigger::LevelHigh).await;
    }

    #[inline]
    pub async fn wait_for_low(&mut self) {
        InputFuture::new(&mut self.pin, InterruptTrigger::LevelLow).await;
    }

    #[inline]
    pub async fn wait_for_rising_edge(&mut self) {
        InputFuture::new(&mut self.pin, InterruptTrigger::EdgeHigh).await;
    }

    #[inline]
    pub async fn wait_for_falling_edge(&mut self) {
        InputFuture::new(&mut self.pin, InterruptTrigger::EdgeLow).await;
    }

    #[inline]
    pub async fn wait_for_any_edge(&mut self) {
        InputFuture::new(&mut self.pin, InterruptTrigger::AnyEdge).await;
    }

    #[inline]
    pub fn dormant_wake(&mut self, cfg: DormantWakeConfig) -> DormantWake<T> {
        let idx = self.pin._pin() as usize;
        self.pin.io().intr(idx / 8).write(|w| {
            w.set_edge_high(idx % 8, cfg.edge_high);
            w.set_edge_low(idx % 8, cfg.edge_low);
        });
        self.pin.io().int_dormant_wake().inte(idx / 8).write_set(|w| {
            w.set_edge_high(idx % 8, cfg.edge_high);
            w.set_edge_low(idx % 8, cfg.edge_low);
            w.set_level_high(idx % 8, cfg.level_high);
            w.set_level_low(idx % 8, cfg.level_low);
        });
        DormantWake {
            pin: self.pin.reborrow(),
            cfg,
        }
    }
}

impl<'d, T: Pin> Drop for Flex<'d, T> {
    #[inline]
    fn drop(&mut self) {
        let idx = self.pin._pin() as usize;
        self.pin.pad_ctrl().write(|_| {});
        self.pin.gpio().ctrl().write(|w| {
            w.set_funcsel(pac::io::vals::Gpio0ctrlFuncsel::NULL as _);
        });
        self.pin.io().int_dormant_wake().inte(idx / 8).write_clear(|w| {
            w.set_edge_high(idx % 8, true);
            w.set_edge_low(idx % 8, true);
            w.set_level_high(idx % 8, true);
            w.set_level_low(idx % 8, true);
        });
    }
}

pub struct DormantWake<'w, T: Pin> {
    pin: PeripheralRef<'w, T>,
    cfg: DormantWakeConfig,
}

impl<'w, T: Pin> Drop for DormantWake<'w, T> {
    fn drop(&mut self) {
        let idx = self.pin._pin() as usize;
        self.pin.io().intr(idx / 8).write(|w| {
            w.set_edge_high(idx % 8, self.cfg.edge_high);
            w.set_edge_low(idx % 8, self.cfg.edge_low);
        });
        self.pin.io().int_dormant_wake().inte(idx / 8).write_clear(|w| {
            w.set_edge_high(idx % 8, true);
            w.set_edge_low(idx % 8, true);
            w.set_level_high(idx % 8, true);
            w.set_level_low(idx % 8, true);
        });
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Pin: Sized {
        fn pin_bank(&self) -> u8;

        #[inline]
        fn _pin(&self) -> u8 {
            self.pin_bank() & 0x1f
        }

        #[inline]
        fn _bank(&self) -> Bank {
            match self.pin_bank() & 0x20 {
                #[cfg(feature = "qspi-as-gpio")]
                1 => Bank::Qspi,
                _ => Bank::Bank0,
            }
        }

        fn io(&self) -> pac::io::Io {
            match self._bank() {
                Bank::Bank0 => crate::pac::IO_BANK0,
                #[cfg(feature = "qspi-as-gpio")]
                Bank::Qspi => crate::pac::IO_QSPI,
            }
        }

        fn gpio(&self) -> pac::io::Gpio {
            self.io().gpio(self._pin() as _)
        }

        fn pad_ctrl(&self) -> Reg<pac::pads::regs::GpioCtrl, RW> {
            let block = match self._bank() {
                Bank::Bank0 => crate::pac::PADS_BANK0,
                #[cfg(feature = "qspi-as-gpio")]
                Bank::Qspi => crate::pac::PADS_QSPI,
            };
            block.gpio(self._pin() as _)
        }

        fn sio_out(&self) -> pac::sio::Gpio {
            SIO.gpio_out(self._bank() as _)
        }

        fn sio_oe(&self) -> pac::sio::Gpio {
            SIO.gpio_oe(self._bank() as _)
        }

        fn sio_in(&self) -> Reg<u32, RW> {
            SIO.gpio_in(self._bank() as _)
        }

        fn int_proc(&self) -> pac::io::Int {
            let proc = SIO.cpuid().read();
            self.io().int_proc(proc as _)
        }
    }
}

pub trait Pin: Peripheral<P = Self> + Into<AnyPin> + sealed::Pin + Sized + 'static {
    /// Degrade to a generic pin struct
    fn degrade(self) -> AnyPin {
        AnyPin {
            pin_bank: self.pin_bank(),
        }
    }

    /// Returns the pin number within a bank
    #[inline]
    fn pin(&self) -> u8 {
        self._pin()
    }

    /// Returns the bank of this pin
    #[inline]
    fn bank(&self) -> Bank {
        self._bank()
    }
}

pub struct AnyPin {
    pin_bank: u8,
}

impl_peripheral!(AnyPin);

impl Pin for AnyPin {}
impl sealed::Pin for AnyPin {
    fn pin_bank(&self) -> u8 {
        self.pin_bank
    }
}

// ==========================

macro_rules! impl_pin {
    ($name:ident, $bank:expr, $pin_num:expr) => {
        impl Pin for peripherals::$name {}
        impl sealed::Pin for peripherals::$name {
            #[inline]
            fn pin_bank(&self) -> u8 {
                ($bank as u8) * 32 + $pin_num
            }
        }

        impl From<peripherals::$name> for crate::gpio::AnyPin {
            fn from(val: peripherals::$name) -> Self {
                crate::gpio::Pin::degrade(val)
            }
        }
    };
}

impl_pin!(PIN_0, Bank::Bank0, 0);
impl_pin!(PIN_1, Bank::Bank0, 1);
impl_pin!(PIN_2, Bank::Bank0, 2);
impl_pin!(PIN_3, Bank::Bank0, 3);
impl_pin!(PIN_4, Bank::Bank0, 4);
impl_pin!(PIN_5, Bank::Bank0, 5);
impl_pin!(PIN_6, Bank::Bank0, 6);
impl_pin!(PIN_7, Bank::Bank0, 7);
impl_pin!(PIN_8, Bank::Bank0, 8);
impl_pin!(PIN_9, Bank::Bank0, 9);
impl_pin!(PIN_10, Bank::Bank0, 10);
impl_pin!(PIN_11, Bank::Bank0, 11);
impl_pin!(PIN_12, Bank::Bank0, 12);
impl_pin!(PIN_13, Bank::Bank0, 13);
impl_pin!(PIN_14, Bank::Bank0, 14);
impl_pin!(PIN_15, Bank::Bank0, 15);
impl_pin!(PIN_16, Bank::Bank0, 16);
impl_pin!(PIN_17, Bank::Bank0, 17);
impl_pin!(PIN_18, Bank::Bank0, 18);
impl_pin!(PIN_19, Bank::Bank0, 19);
impl_pin!(PIN_20, Bank::Bank0, 20);
impl_pin!(PIN_21, Bank::Bank0, 21);
impl_pin!(PIN_22, Bank::Bank0, 22);
impl_pin!(PIN_23, Bank::Bank0, 23);
impl_pin!(PIN_24, Bank::Bank0, 24);
impl_pin!(PIN_25, Bank::Bank0, 25);
impl_pin!(PIN_26, Bank::Bank0, 26);
impl_pin!(PIN_27, Bank::Bank0, 27);
impl_pin!(PIN_28, Bank::Bank0, 28);
impl_pin!(PIN_29, Bank::Bank0, 29);

#[cfg(feature = "qspi-as-gpio")]
impl_pin!(PIN_QSPI_SCLK, Bank::Qspi, 0);
#[cfg(feature = "qspi-as-gpio")]
impl_pin!(PIN_QSPI_SS, Bank::Qspi, 1);
#[cfg(feature = "qspi-as-gpio")]
impl_pin!(PIN_QSPI_SD0, Bank::Qspi, 2);
#[cfg(feature = "qspi-as-gpio")]
impl_pin!(PIN_QSPI_SD1, Bank::Qspi, 3);
#[cfg(feature = "qspi-as-gpio")]
impl_pin!(PIN_QSPI_SD2, Bank::Qspi, 4);
#[cfg(feature = "qspi-as-gpio")]
impl_pin!(PIN_QSPI_SD3, Bank::Qspi, 5);

// ====================

mod eh02 {
    use core::convert::Infallible;

    use super::*;

    impl<'d, T: Pin> embedded_hal_02::digital::v2::InputPin for Input<'d, T> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::OutputPin for Output<'d, T> {
        type Error = Infallible;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::StatefulOutputPin for Output<'d, T> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::ToggleableOutputPin for Output<'d, T> {
        type Error = Infallible;
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Ok(self.toggle())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::InputPin for OutputOpenDrain<'d, T> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::OutputPin for OutputOpenDrain<'d, T> {
        type Error = Infallible;

        #[inline]
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        #[inline]
        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::StatefulOutputPin for OutputOpenDrain<'d, T> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::ToggleableOutputPin for OutputOpenDrain<'d, T> {
        type Error = Infallible;
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Ok(self.toggle())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::InputPin for Flex<'d, T> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::OutputPin for Flex<'d, T> {
        type Error = Infallible;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::StatefulOutputPin for Flex<'d, T> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_02::digital::v2::ToggleableOutputPin for Flex<'d, T> {
        type Error = Infallible;
        #[inline]
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Ok(self.toggle())
        }
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use core::convert::Infallible;

    use super::*;

    impl<'d, T: Pin> embedded_hal_1::digital::ErrorType for Input<'d, T> {
        type Error = Infallible;
    }

    impl<'d, T: Pin> embedded_hal_1::digital::InputPin for Input<'d, T> {
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::ErrorType for Output<'d, T> {
        type Error = Infallible;
    }

    impl<'d, T: Pin> embedded_hal_1::digital::OutputPin for Output<'d, T> {
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::StatefulOutputPin for Output<'d, T> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::ToggleableOutputPin for Output<'d, T> {
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Ok(self.toggle())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::ErrorType for OutputOpenDrain<'d, T> {
        type Error = Infallible;
    }

    impl<'d, T: Pin> embedded_hal_1::digital::OutputPin for OutputOpenDrain<'d, T> {
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::StatefulOutputPin for OutputOpenDrain<'d, T> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::ToggleableOutputPin for OutputOpenDrain<'d, T> {
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Ok(self.toggle())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::InputPin for OutputOpenDrain<'d, T> {
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::ErrorType for Flex<'d, T> {
        type Error = Infallible;
    }

    impl<'d, T: Pin> embedded_hal_1::digital::InputPin for Flex<'d, T> {
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::OutputPin for Flex<'d, T> {
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_high())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(self.set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::StatefulOutputPin for Flex<'d, T> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }

        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d, T: Pin> embedded_hal_1::digital::ToggleableOutputPin for Flex<'d, T> {
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Ok(self.toggle())
        }
    }

    #[cfg(feature = "nightly")]
    impl<'d, T: Pin> embedded_hal_async::digital::Wait for Flex<'d, T> {
        async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
            self.wait_for_high().await;
            Ok(())
        }

        async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
            self.wait_for_low().await;
            Ok(())
        }

        async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
            self.wait_for_rising_edge().await;
            Ok(())
        }

        async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
            self.wait_for_falling_edge().await;
            Ok(())
        }

        async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
            self.wait_for_any_edge().await;
            Ok(())
        }
    }

    #[cfg(feature = "nightly")]
    impl<'d, T: Pin> embedded_hal_async::digital::Wait for Input<'d, T> {
        async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
            self.wait_for_high().await;
            Ok(())
        }

        async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
            self.wait_for_low().await;
            Ok(())
        }

        async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
            self.wait_for_rising_edge().await;
            Ok(())
        }

        async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
            self.wait_for_falling_edge().await;
            Ok(())
        }

        async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
            self.wait_for_any_edge().await;
            Ok(())
        }
    }

    #[cfg(feature = "nightly")]
    impl<'d, T: Pin> embedded_hal_async::digital::Wait for OutputOpenDrain<'d, T> {
        async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
            self.wait_for_high().await;
            Ok(())
        }

        async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
            self.wait_for_low().await;
            Ok(())
        }

        async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
            self.wait_for_rising_edge().await;
            Ok(())
        }

        async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
            self.wait_for_falling_edge().await;
            Ok(())
        }

        async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
            self.wait_for_any_edge().await;
            Ok(())
        }
    }
}
