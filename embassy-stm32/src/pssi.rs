//! Parallel Synchronous Slave Interface (PSSI)
//!
//! PSSI supports high-speed 8-bit or 16-bit parallel data transfers (transmit or receive)
//! with clock (PDCK), data enable (DE), and ready (RDY) signals.

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AfType, Pull};
use crate::rcc::RccPeripheral;
use crate::{Peri, interrupt, peripherals};

/// Data bus width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BusWidth {
    /// 8-bit parallel bus (D0..D7).
    Bits8,
    /// 16-bit parallel bus (D0..D15).
    Bits16,
}

/// Transfer direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// Receive mode (input).
    Receive,
    /// Transmit mode (output).
    Transmit,
}

/// Clock polarity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockPolarity {
    /// Data sampled on rising edge of PDCK.
    RisingEdge,
    /// Data sampled on falling edge of PDCK.
    FallingEdge,
}

/// PSSI configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Parallel bus width.
    pub bus_width: BusWidth,
    /// Data transfer direction.
    pub direction: Direction,
    /// Pixel clock polarity.
    pub clock_polarity: ClockPolarity,
    /// Enable DE (Data Enable) / RDY (Ready) pin signals.
    pub enable_de_rdy: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bus_width: BusWidth::Bits8,
            direction: Direction::Receive,
            clock_polarity: ClockPolarity::RisingEdge,
            enable_de_rdy: false,
        }
    }
}

/// PSSI interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();
        // Clear interrupt enable flags
        r.ier().modify(|w| {
            w.set_ovr_ie(false);
        });
        T::waker().wake();
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> crate::pac::pssi::Pssi;
    fn waker() -> &'static AtomicWaker;
}

/// PSSI instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral {
    /// Interrupt for this PSSI instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(D0Pin, Instance);
pin_trait!(D1Pin, Instance);
pin_trait!(D2Pin, Instance);
pin_trait!(D3Pin, Instance);
pin_trait!(D4Pin, Instance);
pin_trait!(D5Pin, Instance);
pin_trait!(D6Pin, Instance);
pin_trait!(D7Pin, Instance);
pin_trait!(D8Pin, Instance);
pin_trait!(D9Pin, Instance);
pin_trait!(D10Pin, Instance);
pin_trait!(D11Pin, Instance);
pin_trait!(D12Pin, Instance);
pin_trait!(D13Pin, Instance);
pin_trait!(D14Pin, Instance);
pin_trait!(D15Pin, Instance);

pin_trait!(PdckPin, Instance);
pin_trait!(DePin, Instance);
pin_trait!(RdyPin, Instance);

dma_trait!(Dma, Instance);

/// PSSI driver.
pub struct Pssi<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> Pssi<'d, T> {
    /// Create a new PSSI driver for 8-bit mode.
    pub fn new_8bit(
        peri: Peri<'d, T>,
        pdck: Peri<'d, impl PdckPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        crate::rcc::enable_and_reset::<T>();

        pdck.set_as_af(pdck.af_num(), AfType::input(Pull::None));
        d0.set_as_af(d0.af_num(), AfType::input(Pull::None));
        d1.set_as_af(d1.af_num(), AfType::input(Pull::None));
        d2.set_as_af(d2.af_num(), AfType::input(Pull::None));
        d3.set_as_af(d3.af_num(), AfType::input(Pull::None));
        d4.set_as_af(d4.af_num(), AfType::input(Pull::None));
        d5.set_as_af(d5.af_num(), AfType::input(Pull::None));
        d6.set_as_af(d6.af_num(), AfType::input(Pull::None));
        d7.set_as_af(d7.af_num(), AfType::input(Pull::None));

        let r = T::regs();
        r.cr().modify(|w| {
            w.set_edm(match config.bus_width {
                BusWidth::Bits8 => crate::pac::pssi::vals::Edm::BitWidth8,
                BusWidth::Bits16 => crate::pac::pssi::vals::Edm::BitWidth16,
            });
            w.set_ckpol(match config.clock_polarity {
                ClockPolarity::RisingEdge => crate::pac::pssi::vals::Ckpol::RisingEdge,
                ClockPolarity::FallingEdge => crate::pac::pssi::vals::Ckpol::FallingEdge,
            });
            w.set_derdycfg(if config.enable_de_rdy {
                crate::pac::pssi::vals::Derdycfg::Disabled
            } else {
                crate::pac::pssi::vals::Derdycfg::Disabled
            });
            w.set_outen(match config.direction {
                Direction::Receive => crate::pac::pssi::vals::Outen::ReceiveMode,
                Direction::Transmit => crate::pac::pssi::vals::Outen::TransmitMode,
            });
            w.set_enable(true);
        });

        Self { _peri: peri }
    }

    /// Read data from PSSI data register.
    pub fn read_data_register(&self) -> u32 {
        T::regs().dr().read().0
    }

    /// Write data to PSSI data register.
    pub fn write_data_register(&mut self, val: u32) {
        T::regs().dr().write(|w| w.0 = val);
    }
}

foreach_interrupt!(
    ($inst:ident, pssi, PSSI, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::pssi::Pssi {
                crate::pac::$inst
            }
            fn waker() -> &'static AtomicWaker {
                static WAKER: AtomicWaker = AtomicWaker::new();
                &WAKER
            }
        }
    };
);
