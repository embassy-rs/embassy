//! Management Data Input/Output Slave (MDIOS)
//!
//! MDIOS implements an IEEE 802.3 MDIO Slave interface, allowing the microcontroller
//! to act as a managed Ethernet PHY or multi-register slave device on a 2-wire MDIO serial bus.

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AfType, OutputType, Pull, Speed};
use crate::rcc::RccPeripheral;
use crate::{Peri, interrupt, peripherals};

/// MDIOS configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Config {
    /// Port address on the MDIO bus (0..31).
    PortAddress(u8),
}

impl Default for Config {
    fn default() -> Self {
        Self::PortAddress(0)
    }
}

/// MDIOS interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();
        // Disable interrupts to prevent refiring, wake async task
        r.cr().modify(|w| {
            w.set_eie(false);
        });
        T::waker().wake();
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> crate::pac::mdios::Mdios;
    fn waker() -> &'static AtomicWaker;
}

/// MDIOS instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral {
    /// Interrupt for this MDIOS instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(MdcPin, Instance);
pin_trait!(MdioPin, Instance);

/// MDIOS driver.
pub struct Mdios<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> Mdios<'d, T> {
    /// Create a new MDIOS driver.
    pub fn new(
        peri: Peri<'d, T>,
        mdc: Peri<'d, impl MdcPin<T>>,
        mdio: Peri<'d, impl MdioPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        crate::rcc::enable_and_reset::<T>();

        mdc.set_as_af(mdc.af_num(), AfType::input(Pull::None));
        mdio.set_as_af(mdio.af_num(), AfType::output(OutputType::PushPull, Speed::VeryHigh));

        let r = T::regs();

        let port_addr = match config {
            Config::PortAddress(addr) => addr & 0x1F,
        };

        r.cr().write(|w| {
            w.set_port_address(port_addr);
            w.set_en(true); // Enable MDIOS peripheral
        });

        Self { _peri: peri }
    }

    /// Read data written by external MDIO master into input register `reg_idx` (0..31).
    pub fn read_input_register(&self, reg_idx: usize) -> u16 {
        assert!(reg_idx < 32, "MDIOS register index out of range 0..31");
        T::regs().dinr(reg_idx).read().din()
    }

    /// Write output register `reg_idx` (0..31) to be served to external MDIO master upon read.
    pub fn write_output_register(&mut self, reg_idx: usize, val: u16) {
        assert!(reg_idx < 32, "MDIOS register index out of range 0..31");
        T::regs().doutr(reg_idx).write(|w| w.set_dout(val));
    }

    /// Read status register.
    pub fn status(&self) -> u32 {
        T::regs().sr().read().0
    }

    /// Clear register flags.
    pub fn clear_flags(&mut self, mask: u32) {
        T::regs().clrfr().write(|w| w.0 = mask);
    }
}

foreach_interrupt!(
    ($inst:ident, mdios, MDIOS, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::mdios::Mdios {
                crate::pac::$inst
            }
            fn waker() -> &'static AtomicWaker {
                static WAKER: AtomicWaker = AtomicWaker::new();
                &WAKER
            }
        }
    };
);
