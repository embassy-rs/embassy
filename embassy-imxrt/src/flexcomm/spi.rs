//! Serial Peripheral Interface (SPI) driver.

use core::future::{Future, poll_fn};
use core::marker::PhantomData;
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
pub use embedded_hal_1::spi::{MODE_0, MODE_1, MODE_2, MODE_3, Mode, Phase, Polarity};
use paste::paste;

use crate::flexcomm::Clock;
use crate::gpio::{AnyPin, GpioPin as Pin};
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::iopctl::{DriveMode, DriveStrength, Inverter, IopctlPin, Pull, SlewRate};
use crate::pac::spi0::cfg::{Cpha, Cpol};

/// Driver move trait.
#[allow(private_bounds)]
pub trait IoMode: sealed::Sealed {}

/// Blocking mode.
pub struct Blocking;
impl sealed::Sealed for Blocking {}
impl IoMode for Blocking {}

/// Async mode.
pub struct Async;
impl sealed::Sealed for Async {}
impl IoMode for Async {}

/// Spi errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    // No errors for now.
}

/// Spi driver.
pub struct Spi<'a, M: IoMode> {
    info: Info,
    _phantom: PhantomData<&'a M>,
}

impl<'a> Spi<'a, Blocking> {
    /// Create a SPI driver in blocking mode.
    pub fn new_blocking<T: Instance>(
        _inner: Peri<'a, T>,
        sck: Peri<'a, impl SckPin<T> + 'a>,
        mosi: Peri<'a, impl MosiPin<T> + 'a>,
        miso: Peri<'a, impl MisoPin<T> + 'a>,
        config: Config,
    ) -> Self {
        sck.as_sck();
        mosi.as_mosi();
        miso.as_miso();

        Self::new_inner(_inner, Some(sck.into()), Some(mosi.into()), Some(miso.into()), config)
    }

    /// Create a TX-only SPI driver in blocking mode.
    pub fn new_blocking_txonly<T: Instance>(
        _inner: Peri<'a, T>,
        sck: Peri<'a, impl SckPin<T> + 'a>,
        mosi: Peri<'a, impl MosiPin<T> + 'a>,
        config: Config,
    ) -> Self {
        sck.as_sck();
        mosi.as_mosi();

        Self::new_inner(_inner, Some(sck.into()), Some(mosi.into()), None, config)
    }

    /// Create an RX-only SPI driver in blocking mode.
    pub fn new_blocking_rxonly<T: Instance>(
        _inner: Peri<'a, T>,
        sck: Peri<'a, impl SckPin<T> + 'a>,
        miso: Peri<'a, impl MisoPin<T> + 'a>,
        config: Config,
    ) -> Self {
        sck.as_sck();
        miso.as_miso();

        Self::new_inner(_inner, Some(sck.into()), None, Some(miso.into()), config)
    }

    /// Create an internal-loopback SPI driver in blocking mode.
    ///
    /// WARNING: This is only useful for testing as it doesn't use any
    /// external pins.
    pub fn new_blocking_loopback<T: Instance>(_inner: Peri<'a, T>, config: Config) -> Self {
        Self::new_inner(_inner, None, None, None, config)
    }
}

impl<'a, M: IoMode> Spi<'a, M> {
    /// Read data from Spi blocking execution until done.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        critical_section::with(|_| {
            self.info
                .regs
                .fifostat()
                .modify(|_, w| w.txerr().set_bit().rxerr().set_bit());

            for word in data.iter_mut() {
                // wait until we have data in the RxFIFO.
                while self.info.regs.fifostat().read().rxnotempty().bit_is_clear() {}

                self.info
                    .regs
                    .fifowr()
                    .write(|w| unsafe { w.txdata().bits(*word as u16).len().bits(7) });

                *word = self.info.regs.fiford().read().rxdata().bits() as u8;
            }
        });

        self.flush()
    }

    /// Write data to Spi blocking execution until done.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        critical_section::with(|_| {
            self.info
                .regs
                .fifostat()
                .modify(|_, w| w.txerr().set_bit().rxerr().set_bit());

            for (i, word) in data.iter().enumerate() {
                // wait until we have space in the TxFIFO.
                while self.info.regs.fifostat().read().txnotfull().bit_is_clear() {}

                self.info.regs.fifowr().write(|w| {
                    unsafe { w.txdata().bits(*word as u16).len().bits(7) }
                        .rxignore()
                        .set_bit();

                    if i == data.len() - 1 {
                        w.eot().set_bit();
                    }

                    w
                });
            }
        });

        self.flush()
    }

    /// Transfer data to SPI blocking execution until done.
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        let len = read.len().max(write.len());

        critical_section::with(|_| {
            self.info
                .regs
                .fifostat()
                .modify(|_, w| w.txerr().set_bit().rxerr().set_bit());

            for i in 0..len {
                let wb = write.get(i).copied().unwrap_or(0);

                // wait until we have space in the TxFIFO.
                while self.info.regs.fifostat().read().txnotfull().bit_is_clear() {}

                self.info.regs.fifowr().write(|w| {
                    unsafe { w.txdata().bits(wb as u16).len().bits(7) };

                    if i == len - 1 {
                        w.eot().set_bit();
                    }

                    w
                });

                // wait until we have data in the RxFIFO.
                while self.info.regs.fifostat().read().rxnotempty().bit_is_clear() {}

                let rb = self.info.regs.fiford().read().rxdata().bits() as u8;

                if let Some(r) = read.get_mut(i) {
                    *r = rb;
                }
            }
        });

        self.flush()
    }

    /// Transfer data in place to SPI blocking execution until done.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        critical_section::with(|_| {
            self.info
                .regs
                .fifostat()
                .modify(|_, w| w.txerr().set_bit().rxerr().set_bit());

            for word in data {
                // wait until we have space in the TxFIFO.
                while self.info.regs.fifostat().read().txnotfull().bit_is_clear() {}
                self.info
                    .regs
                    .fifowr()
                    .write(|w| unsafe { w.txdata().bits(*word as u16) });

                // wait until we have data in the RxFIFO.
                while self.info.regs.fifostat().read().rxnotempty().bit_is_clear() {}
                *word = self.info.regs.fiford().read().rxdata().bits() as u8;
            }
        });

        self.flush()
    }

    /// Block execution until Spi is done.
    pub fn flush(&mut self) -> Result<(), Error> {
        let regs = self.info.regs;
        while regs.stat().read().mstidle().bit_is_clear() {}
        Ok(())
    }
}

impl<'a> Spi<'a, Async> {
    /// Create a SPI driver in async mode.
    pub fn new_async<T: Instance>(
        _inner: Peri<'a, T>,
        sck: Peri<'a, impl SckPin<T> + 'a>,
        mosi: Peri<'a, impl MosiPin<T> + 'a>,
        miso: Peri<'a, impl MisoPin<T> + 'a>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        config: Config,
    ) -> Self {
        sck.as_sck();
        mosi.as_mosi();
        miso.as_miso();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self::new_inner(_inner, Some(sck.into()), Some(mosi.into()), Some(miso.into()), config)
    }

    /// Create a TX-only SPI driver in async mode.
    pub fn new_async_txonly<T: Instance>(
        _inner: Peri<'a, T>,
        sck: Peri<'a, impl SckPin<T> + 'a>,
        mosi: Peri<'a, impl MosiPin<T> + 'a>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        config: Config,
    ) -> Self {
        sck.as_sck();
        mosi.as_mosi();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self::new_inner(_inner, Some(sck.into()), Some(mosi.into()), None, config)
    }

    /// Create an RX-only SPI driver in async mode.
    pub fn new_async_rxonly<T: Instance>(
        _inner: Peri<'a, T>,
        sck: Peri<'a, impl SckPin<T> + 'a>,
        miso: Peri<'a, impl MisoPin<T> + 'a>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        config: Config,
    ) -> Self {
        sck.as_sck();
        miso.as_miso();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self::new_inner(_inner, Some(sck.into()), None, Some(miso.into()), config)
    }

    /// Create an internal-loopback SPI driver in async mode.
    ///
    /// WARNING: This is only useful for testing as it doesn't use any
    /// external pins.
    pub fn new_async_loopback<T: Instance>(
        _inner: Peri<'a, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
        config: Config,
    ) -> Self {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self::new_inner(_inner, None, None, None, config)
    }

    /// Read data from Spi async execution until done.
    pub async fn async_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        critical_section::with(|_| {
            self.info
                .regs
                .fifostat()
                .modify(|_, w| w.txerr().set_bit().rxerr().set_bit());
        });

        for word in data.iter_mut() {
            // wait until we have data in the RxFIFO.
            self.wait_for(
                |me| {
                    if me.info.regs.fifostat().read().rxnotempty().bit_is_set() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                },
                |me| {
                    me.info
                        .regs
                        .fifointenset()
                        .write(|w| w.rxlvl().set_bit().rxerr().set_bit());
                },
            )
            .await;

            self.info
                .regs
                .fifowr()
                .write(|w| unsafe { w.txdata().bits(*word as u16).len().bits(7) });

            *word = self.info.regs.fiford().read().rxdata().bits() as u8;
        }

        self.async_flush().await;

        Ok(())
    }

    /// Write data to Spi async execution until done.
    pub async fn async_write(&mut self, data: &[u8]) -> Result<(), Error> {
        critical_section::with(|_| {
            self.info
                .regs
                .fifostat()
                .modify(|_, w| w.txerr().set_bit().rxerr().set_bit());
        });

        for (i, word) in data.iter().enumerate() {
            // wait until we have space in the TxFIFO.
            self.wait_for(
                |me| {
                    if me.info.regs.fifostat().read().txnotfull().bit_is_set() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                },
                |me| {
                    me.info
                        .regs
                        .fifointenset()
                        .write(|w| w.txlvl().set_bit().txerr().set_bit());
                },
            )
            .await;

            self.info.regs.fifowr().write(|w| {
                unsafe { w.txdata().bits(*word as u16).len().bits(7) }
                    .rxignore()
                    .set_bit();

                if i == data.len() - 1 {
                    w.eot().set_bit();
                }

                w
            });
        }

        self.async_flush().await;

        Ok(())
    }

    /// Transfer data to SPI async execution until done.
    pub async fn async_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        let len = read.len().max(write.len());

        critical_section::with(|_| {
            self.info
                .regs
                .fifostat()
                .modify(|_, w| w.txerr().set_bit().rxerr().set_bit());
        });

        for i in 0..len {
            let wb = write.get(i).copied().unwrap_or(0);

            // wait until we have space in the TxFIFO.
            self.wait_for(
                |me| {
                    if me.info.regs.fifostat().read().txnotfull().bit_is_set() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                },
                |me| {
                    me.info.regs.fifotrig().write(|w| w.txlvlena().set_bit());
                    me.info
                        .regs
                        .fifointenset()
                        .write(|w| w.txlvl().set_bit().txerr().set_bit());
                },
            )
            .await;

            self.info.regs.fifowr().write(|w| {
                unsafe { w.txdata().bits(wb as u16).len().bits(7) };

                if i == len - 1 {
                    w.eot().set_bit();
                }

                w
            });

            // wait until we have data in the RxFIFO.
            self.wait_for(
                |me| {
                    if me.info.regs.fifostat().read().rxnotempty().bit_is_set() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                },
                |me| {
                    me.info.regs.fifotrig().write(|w| w.rxlvlena().set_bit());
                    me.info
                        .regs
                        .fifointenset()
                        .write(|w| w.rxlvl().set_bit().rxerr().set_bit());
                },
            )
            .await;

            let rb = self.info.regs.fiford().read().rxdata().bits() as u8;

            if let Some(r) = read.get_mut(i) {
                *r = rb;
            }
        }

        self.async_flush().await;

        Ok(())
    }

    /// Transfer data in place to SPI async execution until done.
    pub async fn async_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        critical_section::with(|_| {
            self.info
                .regs
                .fifostat()
                .modify(|_, w| w.txerr().set_bit().rxerr().set_bit());
        });

        for word in data {
            // wait until we have space in the TxFIFO.
            self.wait_for(
                |me| {
                    if me.info.regs.fifostat().read().txnotfull().bit_is_set() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                },
                |me| {
                    me.info
                        .regs
                        .fifointenset()
                        .write(|w| w.txlvl().set_bit().txerr().set_bit());
                },
            )
            .await;

            self.info
                .regs
                .fifowr()
                .write(|w| unsafe { w.txdata().bits(*word as u16) });

            // wait until we have data in the RxFIFO.
            self.wait_for(
                |me| {
                    if me.info.regs.fifostat().read().rxnotempty().bit_is_set() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                },
                |me| {
                    me.info
                        .regs
                        .fifointenset()
                        .write(|w| w.rxlvl().set_bit().rxerr().set_bit());
                },
            )
            .await;

            *word = self.info.regs.fiford().read().rxdata().bits() as u8;
        }

        self.async_flush().await;

        Ok(())
    }

    /// Async flush.
    pub fn async_flush(&mut self) -> impl Future<Output = ()> + use<'_, 'a> {
        self.wait_for(
            |me| {
                if me.info.regs.stat().read().mstidle().bit_is_set() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            },
            |me| {
                me.info.regs.intenset().write(|w| w.mstidleen().set_bit());
            },
        )
    }

    /// Calls `f` to check if we are ready or not.
    /// If not, `g` is called once the waker is set (to eg enable the required interrupts).
    fn wait_for<F, U, G>(&mut self, mut f: F, mut g: G) -> impl Future<Output = U> + use<'_, 'a, F, U, G>
    where
        F: FnMut(&mut Self) -> Poll<U>,
        G: FnMut(&mut Self),
    {
        poll_fn(move |cx| {
            // Register waker before checking condition, to ensure that wakes/interrupts
            // aren't lost between f() and g()
            self.info.waker.register(cx.waker());
            let r = f(self);

            if r.is_pending() {
                g(self);
            }

            r
        })
    }
}

impl<'a, M: IoMode> Spi<'a, M> {
    fn new_inner<T: Instance>(
        _inner: Peri<'a, T>,
        sck: Option<Peri<'a, AnyPin>>,
        mosi: Option<Peri<'a, AnyPin>>,
        miso: Option<Peri<'a, AnyPin>>,
        config: Config,
    ) -> Self {
        // REVISIT: allow selecting from multiple clocks.
        let clk = Self::clock(&config);

        T::enable(clk);
        T::into_spi();

        Self::apply_config(T::info().regs, &config);

        let info = T::info();
        let regs = info.regs;

        critical_section::with(|_| match (sck.is_some(), mosi.is_some(), miso.is_some()) {
            (true, true, true) => {
                regs.fifocfg().modify(|_, w| {
                    w.enabletx()
                        .set_bit()
                        .emptytx()
                        .set_bit()
                        .enablerx()
                        .set_bit()
                        .emptyrx()
                        .set_bit()
                });
            }
            (true, false, true) => {
                regs.fifocfg().modify(|_, w| {
                    w.enabletx()
                        .set_bit()
                        .emptytx()
                        .clear_bit()
                        .enablerx()
                        .set_bit()
                        .emptyrx()
                        .set_bit()
                });
            }
            (true, true, false) => {
                regs.fifocfg().modify(|_, w| {
                    w.enabletx()
                        .set_bit()
                        .emptytx()
                        .set_bit()
                        .enablerx()
                        .clear_bit()
                        .emptyrx()
                        .set_bit()
                });
            }
            (false, _, _) => {
                regs.fifocfg().modify(|_, w| {
                    w.enabletx()
                        .set_bit()
                        .emptytx()
                        .set_bit()
                        .enablerx()
                        .set_bit()
                        .emptyrx()
                        .set_bit()
                });
                regs.cfg().modify(|_, w| w.loop_().enabled());
            }
            _ => {}
        });

        Self {
            info,
            _phantom: PhantomData,
        }
    }

    fn set_config(&mut self, config: &Config) {
        Self::apply_config(self.info.regs, config);
    }

    fn clock(config: &Config) -> Clock {
        const SFRO_CLOCK_SPEED_HZ: u32 = 16_000_000;

        if config.frequency > SFRO_CLOCK_SPEED_HZ {
            Clock::Ffro
        } else {
            Clock::Sfro
        }
    }

    fn clock_frequency(clock: Clock) -> u32 {
        match clock {
            Clock::Sfro => 16_000_000,
            Clock::Ffro => 48_000_000,
            _ => unreachable!(),
        }
    }

    fn apply_config(regs: &'static crate::pac::spi0::RegisterBlock, config: &Config) {
        let polarity = if config.mode.polarity == Polarity::IdleLow {
            Cpol::Low
        } else {
            Cpol::High
        };

        let phase = if config.mode.phase == Phase::CaptureOnFirstTransition {
            Cpha::Change
        } else {
            Cpha::Capture
        };

        let clk = Self::clock(config);
        let div = Self::clock_frequency(clk) / config.frequency - 1;

        critical_section::with(|_| {
            // disable SPI every time we need to modify configuration.
            regs.cfg().modify(|_, w| w.enable().disabled());

            regs.cfg().modify(|_, w| {
                w.cpha()
                    .variant(phase)
                    .cpol()
                    .variant(polarity)
                    .loop_()
                    .disabled()
                    .master()
                    .master_mode()
            });

            regs.div().write(|w| unsafe { w.divval().bits(div as u16) });

            regs.cfg().modify(|_, w| w.enable().enabled());
        });
    }
}

/// Spi config.
#[derive(Clone)]
pub struct Config {
    /// Frequency in Hertz.
    pub frequency: u32,
    /// SPI operating mode.
    pub mode: Mode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: 1_000_000,
            mode: MODE_0,
        }
    }
}

struct Info {
    regs: &'static crate::pac::spi0::RegisterBlock,
    waker: &'static AtomicWaker,
}

// SAFETY: safety for Send here is the same as the other accessors to
// unsafe blocks: it must be done from a single executor context.
//
// This is a temporary workaround -- a better solution might be to
// refactor Info to no longer maintain a reference to regs, but
// instead look up the correct register set and then perform
// operations within an unsafe block as we do for other peripherals
unsafe impl Send for Info {}

trait SealedInstance {
    fn info() -> Info;
}

/// Spi interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let waker = T::info().waker;
        let stat = T::info().regs.fifointstat().read();

        if stat.perint().bit_is_set() {
            T::info().regs.intenclr().write(|w| w.mstidle().clear_bit_by_one());
        }

        if stat.txlvl().bit_is_set() {
            T::info().regs.fifointenclr().write(|w| w.txlvl().set_bit());
        }

        if stat.txerr().bit_is_set() {
            T::info().regs.fifointenclr().write(|w| w.txerr().set_bit());
        }

        if stat.rxlvl().bit_is_set() {
            T::info().regs.fifointenclr().write(|w| w.rxlvl().set_bit());
        }

        if stat.rxerr().bit_is_set() {
            T::info().regs.fifointenclr().write(|w| w.rxerr().set_bit());
        }

        waker.wake();
    }
}

/// Spi instance trait.
#[allow(private_bounds)]
pub trait Instance: crate::flexcomm::IntoSpi + SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this Spi instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_instance {
    ($($n:expr),*) => {
        $(
            paste!{
                impl SealedInstance for crate::peripherals::[<FLEXCOMM $n>] {
                    #[inline]
                    fn info() -> Info {
                        static WAKER: AtomicWaker = AtomicWaker::new();

                        Info {
                            regs: unsafe { &*crate::pac::[<Spi $n>]::ptr() },
                            waker: &WAKER,
                        }
                    }
                }

                impl Instance for crate::peripherals::[<FLEXCOMM $n>] {
                    type Interrupt = crate::interrupt::typelevel::[<FLEXCOMM $n>];
                }
            }
        )*
    }
}

impl_instance!(0, 1, 2, 3, 4, 5, 6, 7, 14);

mod sealed {
    /// Seal a trait
    pub trait Sealed {}
}

impl<T: Pin> sealed::Sealed for T {}

/// IO configuration trait for Spi clk
pub trait SckPin<T: Instance>: Pin + sealed::Sealed + PeripheralType {
    /// convert the pin to appropriate function for Spi clk usage.
    fn as_sck(&self);
}

/// IO configuration trait for Spi mosi
pub trait MosiPin<T: Instance>: Pin + sealed::Sealed + PeripheralType {
    /// convert the pin to appropriate function for Spi mosi usage.
    fn as_mosi(&self);
}

/// IO configuration trait for Spi miso
pub trait MisoPin<T: Instance>: Pin + sealed::Sealed + PeripheralType {
    /// convert the pin to appropriate function for Spi miso usage.
    fn as_miso(&self);
}

macro_rules! impl_pin_trait {
    ($fcn:ident, $mode:ident, $($pin:ident, $fn:ident),*) => {
        paste! {
            $(
                impl [<$mode:camel Pin>]<crate::peripherals::$fcn> for crate::peripherals::$pin {
                    fn [<as_ $mode>](&self) {
                        // UM11147 table 530 pg 518
                        self.set_function(crate::iopctl::Function::$fn)
                            .set_pull(Pull::None)
                            .enable_input_buffer()
                            .set_slew_rate(SlewRate::Standard)
                            .set_drive_strength(DriveStrength::Normal)
                            .disable_analog_multiplex()
                            .set_drive_mode(DriveMode::PushPull)
                            .set_input_inverter(Inverter::Disabled);
                    }
                }
            )*
        }
    }
}

// FLEXCOMM0
impl_pin_trait!(FLEXCOMM0, sck, PIO0_0, F1, PIO3_0, F5);
impl_pin_trait!(FLEXCOMM0, miso, PIO0_1, F1, PIO3_1, F5);
impl_pin_trait!(FLEXCOMM0, mosi, PIO0_2, F1, PIO3_2, F5);

// FLEXCOMM1
impl_pin_trait!(FLEXCOMM1, sck, PIO0_7, F1, PIO7_25, F1);
impl_pin_trait!(FLEXCOMM1, miso, PIO0_8, F1, PIO7_26, F1);
impl_pin_trait!(FLEXCOMM1, mosi, PIO0_9, F1, PIO7_28, F1);

// FLEXCOMM2
impl_pin_trait!(FLEXCOMM2, sck, PIO0_14, F1, PIO7_29, F5);
impl_pin_trait!(FLEXCOMM2, miso, PIO0_15, F1, PIO7_30, F5);
impl_pin_trait!(FLEXCOMM2, mosi, PIO0_16, F1, PIO7_31, F5);

// FLEXCOMM3
impl_pin_trait!(FLEXCOMM3, sck, PIO0_21, F1);
impl_pin_trait!(FLEXCOMM3, miso, PIO0_22, F1);
impl_pin_trait!(FLEXCOMM3, mosi, PIO0_23, F1);

// FLEXCOMM4
impl_pin_trait!(FLEXCOMM4, sck, PIO0_28, F1);
impl_pin_trait!(FLEXCOMM4, miso, PIO0_29, F1);
impl_pin_trait!(FLEXCOMM4, mosi, PIO0_30, F1);

// FLEXCOMM5
impl_pin_trait!(FLEXCOMM5, sck, PIO1_3, F1, PIO3_15, F5);
impl_pin_trait!(FLEXCOMM5, miso, PIO1_4, F1, PIO3_16, F5);
impl_pin_trait!(FLEXCOMM5, mosi, PIO1_5, F1, PIO3_17, F5);

// FLEXCOMM6
impl_pin_trait!(FLEXCOMM6, sck, PIO3_25, F1);
impl_pin_trait!(FLEXCOMM6, miso, PIO3_26, F1);
impl_pin_trait!(FLEXCOMM6, mosi, PIO3_27, F1);

// FLEXCOMM7
impl_pin_trait!(FLEXCOMM7, sck, PIO4_0, F1);
impl_pin_trait!(FLEXCOMM7, miso, PIO4_1, F1);
impl_pin_trait!(FLEXCOMM7, mosi, PIO4_2, F1);

// FLEXCOMM14
impl_pin_trait!(FLEXCOMM14, sck, PIO1_11, F1);
impl_pin_trait!(FLEXCOMM14, miso, PIO1_12, F1);
impl_pin_trait!(FLEXCOMM14, mosi, PIO1_13, F1);

/// Spi Tx DMA trait.
#[allow(private_bounds)]
pub trait TxDma<T: Instance>: crate::dma::Channel {}

/// Spi Rx DMA trait.
#[allow(private_bounds)]
pub trait RxDma<T: Instance>: crate::dma::Channel {}

macro_rules! impl_dma {
    ($fcn:ident, $mode:ident, $dma:ident) => {
        paste! {
            impl [<$mode Dma>]<crate::peripherals::$fcn> for crate::peripherals::$dma {}
        }
    };
}

impl_dma!(FLEXCOMM0, Rx, DMA0_CH0);
impl_dma!(FLEXCOMM0, Tx, DMA0_CH1);

impl_dma!(FLEXCOMM1, Rx, DMA0_CH2);
impl_dma!(FLEXCOMM1, Tx, DMA0_CH3);

impl_dma!(FLEXCOMM2, Rx, DMA0_CH4);
impl_dma!(FLEXCOMM2, Tx, DMA0_CH5);

impl_dma!(FLEXCOMM3, Rx, DMA0_CH6);
impl_dma!(FLEXCOMM3, Tx, DMA0_CH7);

impl_dma!(FLEXCOMM4, Rx, DMA0_CH8);
impl_dma!(FLEXCOMM4, Tx, DMA0_CH9);

impl_dma!(FLEXCOMM5, Rx, DMA0_CH10);
impl_dma!(FLEXCOMM5, Tx, DMA0_CH11);

impl_dma!(FLEXCOMM6, Rx, DMA0_CH12);
impl_dma!(FLEXCOMM6, Tx, DMA0_CH13);

impl_dma!(FLEXCOMM7, Rx, DMA0_CH14);
impl_dma!(FLEXCOMM7, Tx, DMA0_CH15);

impl_dma!(FLEXCOMM14, Rx, DMA0_CH16);
impl_dma!(FLEXCOMM14, Tx, DMA0_CH17);

// ==============================

impl<'d, M: IoMode> embedded_hal_02::blocking::spi::Transfer<u8> for Spi<'d, M> {
    type Error = Error;
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.blocking_transfer_in_place(words)?;
        Ok(words)
    }
}

impl<'d, M: IoMode> embedded_hal_02::blocking::spi::Write<u8> for Spi<'d, M> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }
}

impl embedded_hal_1::spi::Error for Error {
    fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
        match *self {}
    }
}

impl<'d, M: IoMode> embedded_hal_1::spi::ErrorType for Spi<'d, M> {
    type Error = Error;
}

impl<'d, M: IoMode> embedded_hal_1::spi::SpiBus<u8> for Spi<'d, M> {
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush()
    }

    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(words)
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_transfer(read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_transfer_in_place(words)
    }
}

impl<'d> embedded_hal_async::spi::SpiBus<u8> for Spi<'d, Async> {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.async_flush().await;

        Ok(())
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.async_write(words).await
    }

    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.async_read(words).await
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.async_transfer(read, write).await
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.async_transfer_in_place(words).await
    }
}

impl<'d, M: IoMode> SetConfig for Spi<'d, M> {
    type Config = Config;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), ()> {
        self.set_config(config);

        Ok(())
    }
}
