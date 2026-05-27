//! TACH driver.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AnyPin, Pin as GpioPin};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::pac::tach0::Tach0;
use crate::{interrupt, pac, peripherals};

// The minimum and maximum RPMs the TACH peripheral can reliably measure according to datasheet
const MIN_RPM: u32 = 100;
const MAX_RPM: u32 = 30_000;

// The fixed clock to TACH
const TACH_CLK: u32 = 100_000;

// Seconds per minute
const SEC_PER_MIN: u32 = 60;

/// Tach interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // Disable the interrupt here to prevent storm, so we can read the count ready bit and clear it later
        T::regs().ctrl().modify(|w| w.set_cnt_rdy_int_en(false));
        T::waker().wake();
    }
}

/// TACH error.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The calculated RPM is below the minimum RPM the tach peripheral can reliably measure (100 RPM),
    /// thus it is likely inaccurate.
    ///
    /// It is very likely the fan is stopped, however this driver does not assume that.
    RpmLow(u32),

    /// The calculated RPM is above the maximum RPM the tach peripheral can reliably measure (30,000 RPM),
    /// thus it is likely inaccurate.
    RpmHigh(u32),

    /// Calculating RPM would result in a division by zero.
    ///
    /// This would indicate the selected number of [`Edges`] occurred in less than one tach clock period (10 µs),
    /// thus the tach counter is zero.
    DivideByZero,
}

/// The number of TACH input edges for which the number of 100 kHz pulses will be counted.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Edges {
    /// Two edges.
    _2,
    /// Three edges.
    _3,
    /// Five edges.
    _5,
    /// Nine edges.
    _9,
}

impl From<Edges> for u8 {
    fn from(edges: Edges) -> Self {
        match edges {
            Edges::_2 => 0b00,
            Edges::_3 => 0b01,
            Edges::_5 => 0b10,
            Edges::_9 => 0b11,
        }
    }
}

/// TACH peripheral config.
pub struct Config {
    /// Remove high frequency glitches from TACH input
    /// (e.g. pulses less than two 100 KHz periods wide will be filtered).
    ///
    /// It is recommended to always enable this.
    pub filter_en: bool,

    /// The number of TACH input edges for which the number of 100 kHz pulses will be counted.
    ///
    /// This should match the number of edges per revolution for specific fan model.
    ///
    /// For typical fans which produce a 50% duty cycle square wave,
    /// five edges (two periods) represents one revolution.
    ///
    /// See figure 28-2 in datasheet.
    pub edges: Edges,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            filter_en: true,
            edges: Edges::_5,
        }
    }
}

/// TACH driver.
pub struct Tach<'d, T: Instance> {
    _peri: Peri<'d, T>,
    _pin: Peri<'d, AnyPin>,
}

impl<'d, T: Instance> Tach<'d, T> {
    /// Create a new TACH driver instance.
    pub fn new(
        _peri: Peri<'d, T>,
        _pin: Peri<'d, impl TachPin<T>>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        config: Config,
    ) -> Self {
        // Sets given pin to TACH mode (alternate function 1)
        // REVISIT: make an API for this
        critical_section::with(|_| {
            _pin.regs().ctrl1.modify(|w| {
                w.set_mux_ctrl(pac::Function::F1);
            })
        });

        // Set tach to mode 1 with edges and filtering set by config, then enable.
        //
        // Mode 0 does not seem very useful for a high level driver like this,
        // so users are free to use the PAC if they need it.
        T::regs().ctrl().write(|w| {
            w.set_en(true);
            w.set_filt_en(config.filter_en);
            w.set_rd_mod_sel(true);
            w.set_edges(config.edges.into());
        });

        T::Interrupt::unpend();
        // SAFETY: We have sole control of TACH interrupts so this is safe to do.
        unsafe { T::Interrupt::enable() };

        Self {
            _peri,
            _pin: _pin.into(),
        }
    }

    fn calculate_rpm(&self) -> Result<u16, Error> {
        let count = T::regs().ctrl().read().cntr();

        if count == 0 {
            // Somehow we saw all edges before a single 100 kHz pulse?
            Err(Error::DivideByZero)
        } else {
            // This calculation assumes the user correctly set the number of edges per one revolution
            let rpm = (SEC_PER_MIN * TACH_CLK) / count as u32;

            // The datasheet specifies the peripheral can reliably measure 100-30,000 RPM
            // If we calculate a value outside that range, still return it, but wrap in an error to let user know
            match rpm {
                ..MIN_RPM => Err(Error::RpmLow(rpm)),
                MIN_RPM..=MAX_RPM => Ok(rpm as u16),
                _ => Err(Error::RpmHigh(rpm)),
            }
        }
    }

    /// Return the measured fan speed in revolutions per minute (RPM).
    ///
    /// # Errors
    ///
    /// If calculated RPM is below 100 RPM, [`Error::RpmLow`] is returned. In this case,
    /// the fan is likely stopped.
    ///
    /// If calculated RPM is above 30,000 RPM, [`Error::RpmHigh`] is returned. In this case,
    /// the fan is rotating too quickly and the calculated RPM is likely inaccurate.
    ///
    /// If selected number of [`Edges`] occurred in less than one TACH clock period (10 µs),
    /// [`Error::DivideByZero`] is returned as calculating RPM is impossible.
    pub async fn rpm(&mut self) -> Result<u16, Error> {
        poll_fn(|cx| {
            T::waker().register(cx.waker());

            if T::regs().sts().read().cnt_rdy_sts() {
                // Count has been latched, so clear ready bit and calculate RPM from count
                T::regs().sts().modify(|w| w.set_cnt_rdy_sts(true));
                Poll::Ready(self.calculate_rpm())
            } else {
                // Count still not latched, so enable interrupt and wait
                T::regs().ctrl().modify(|w| w.set_cnt_rdy_int_en(true));
                Poll::Pending
            }
        })
        .await
    }
}

impl<'d, T: Instance> Drop for Tach<'d, T> {
    fn drop(&mut self) {
        T::regs().ctrl().write(|w| w.set_en(false));
        T::Interrupt::disable();
    }
}

trait SealedInstance {
    fn regs() -> Tach0;
    fn waker() -> &'static AtomicWaker;
}

/// TACH Instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_instance {
    ($peri:ident) => {
        impl SealedInstance for peripherals::$peri {
            #[inline(always)]
            fn regs() -> Tach0 {
                pac::$peri
            }

            #[inline(always)]
            fn waker() -> &'static AtomicWaker {
                static WAKER: AtomicWaker = AtomicWaker::new();
                &WAKER
            }
        }

        impl Instance for peripherals::$peri {
            type Interrupt = crate::interrupt::typelevel::$peri;
        }
    };
}

impl_instance!(TACH0);
impl_instance!(TACH1);
impl_instance!(TACH2);
impl_instance!(TACH3);

/// A GPIO pin that can be configured as a TACH pin.
pub trait TachPin<T: Instance>: GpioPin + PeripheralType {}

macro_rules! impl_pin {
    ($peri:ident, $($pin:ident),*) => {
        $(
            impl TachPin<peripherals::$peri> for peripherals::$pin {}
        )*
    }
}

impl_pin!(TACH0, GPIO50);
impl_pin!(TACH1, GPIO51);
impl_pin!(TACH2, GPIO52);
impl_pin!(TACH3, GPIO33);
