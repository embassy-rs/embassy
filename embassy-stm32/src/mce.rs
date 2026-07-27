//! Memory Cipher Engine (MCE)
//!
//! MCE transparently encrypts/decrypts memory regions on the AXI bus. This is a
//! minimal driver covering region configuration and illegal-access interrupts;
//! cipher-context/key programming is left to the application via [`Mce::regs`].

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};

use crate::interrupt::typelevel::Interrupt;
use crate::pac::mce::Mce as Regs;
use crate::pac::mce::regs::Iacr;
use crate::rcc::{self, RccPeripheral};
use crate::{interrupt, peripherals};

/// One of four address regions on an MCE instance (0..3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Region(pub u8);

impl Region {
    fn index(self) -> usize {
        assert!(self.0 < 4);
        self.0 as usize
    }
}

/// Region address and encryption settings.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RegionConfig {
    /// Inclusive start address (physical). Must be 4KiB-aligned.
    pub start: u32,
    /// Exclusive end address (physical). Must be 4KiB-aligned.
    pub end: u32,
    /// Cipher context ID (0..3).
    pub context_id: u8,
    /// When true, traffic in this region is encrypted.
    pub encrypted: bool,
}

trait SealedInstance: RccPeripheral {
    fn regs() -> Regs;
}

/// MCE instance trait.
#[allow(private_bounds)]
pub trait Instance: PeripheralType + SealedInstance + 'static {
    /// Global interrupt for this instance.
    type GlobalInterrupt: interrupt::typelevel::Interrupt;
}

/// Memory Cipher Engine driver.
pub struct Mce<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> Mce<'d, T> {
    /// Create a new MCE driver and enable its bus clock.
    pub fn new(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::GlobalInterrupt, GlobalInterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        T::GlobalInterrupt::unpend();
        unsafe { T::GlobalInterrupt::enable() };

        Self { _peri: peri }
    }

    /// Direct access to the underlying PAC registers.
    pub fn regs() -> Regs {
        T::regs()
    }

    /// Configure (but do not enable) a memory region.
    ///
    /// `config.start` and `config.end` must be aligned to the 4KiB block size.
    pub fn configure_region(region: Region, config: RegionConfig) {
        assert!(config.context_id < 4);
        assert!(config.start < config.end);
        assert!(config.start % 4096 == 0, "MCE region start must be 4KiB-aligned");
        assert!(config.end % 4096 == 0, "MCE region end must be 4KiB-aligned");

        let idx = region.index();
        let start = config.start >> 12;
        let end = (config.end - 1) >> 12;

        T::regs().saddr(idx).write(|w| w.set_baddstart(start));
        T::regs().eaddr(idx).write(|w| w.set_baddend(end));
        T::regs().regcr(idx).modify(|w| {
            w.set_ctxid(config.context_id);
            w.set_enc(if config.encrypted { 1 } else { 0 });
            w.set_bren(false);
        });
    }

    /// Enable or disable a configured region.
    pub fn set_region_enabled(region: Region, enabled: bool) {
        T::regs().regcr(region.index()).modify(|w| w.set_bren(enabled));
    }

    /// Clear illegal-access status flags.
    pub fn clear_illegal_access_flags() {
        let iasr = T::regs().iasr().read().0;
        T::regs().iacr().write_value(Iacr(iasr));
    }
}

/// Global interrupt handler for illegal access events.
pub struct GlobalInterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::GlobalInterrupt> for GlobalInterruptHandler<T> {
    unsafe fn on_interrupt() {
        Mce::<T>::clear_illegal_access_flags();
    }
}

foreach_peripheral!(
    (mce, $inst:ident) => {
        impl SealedInstance for peripherals::$inst {
            fn regs() -> Regs {
                crate::pac::$inst
            }
        }

        impl Instance for peripherals::$inst {
            type GlobalInterrupt = crate::_generated::peripheral_interrupts::$inst::GLOBAL;
        }
    };
);
