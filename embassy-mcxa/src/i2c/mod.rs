//! I2C Support

use embassy_hal_internal::PeripheralType;
use maitake_sync::WaitCell;

use crate::clocks::Gate;
use crate::clocks::periph_helpers::Lpi2cConfig;
use crate::dma::{DmaChannel, DmaRequest};
use crate::gpio::GpioPin;
use crate::{interrupt, pac};

pub mod controller;
pub mod target;

pub(crate) mod sealed {
    /// Seal a trait
    pub trait Sealed {}
    pub trait SealedPin<Instance> {}
}

pub(crate) trait SealedInstance: Gate<MrccPeriphConfig = Lpi2cConfig> {
    fn info() -> &'static Info;

    /// Clock instance
    const CLOCK_INSTANCE: crate::clocks::periph_helpers::Lpi2cInstance;
    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
    const TX_DMA_REQUEST: DmaRequest;
    const RX_DMA_REQUEST: DmaRequest;
}

/// I2C Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this I2C instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pub(crate) struct Info {
    pub(crate) regs: pac::lpi2c::Lpi2c,
    pub(crate) wait_cell: WaitCell,
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::lpi2c::Lpi2c {
        self.regs
    }

    #[inline(always)]
    fn wait_cell(&self) -> &WaitCell {
        &self.wait_cell
    }
}

unsafe impl Sync for Info {}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_lpi2c_instance {
    ($n:literal) => {
        paste::paste! {
            impl crate::i2c::SealedInstance for crate::peripherals::[<LPI2C $n>] {
                fn info() -> &'static crate::i2c::Info {
                    static INFO: crate::i2c::Info = crate::i2c::Info {
                        regs: crate::pac::[<LPI2C $n>],
                        wait_cell: maitake_sync::WaitCell::new(),
                    };
                    &INFO
                }

                const CLOCK_INSTANCE: crate::clocks::periph_helpers::Lpi2cInstance
                    = crate::clocks::periph_helpers::Lpi2cInstance::[<Lpi2c $n>];
                const PERF_INT_INCR: fn() = crate::perf_counters::[<incr_interrupt_i2c $n>];
                const PERF_INT_WAKE_INCR: fn() = crate::perf_counters::[<incr_interrupt_i2c $n _wake>];
                const TX_DMA_REQUEST: DmaRequest = DmaRequest::[<Lpi2C $n Tx>];
                const RX_DMA_REQUEST: DmaRequest = DmaRequest::[<Lpi2C $n Rx>];
            }

            impl crate::i2c::Instance for crate::peripherals::[<LPI2C $n>] {
                type Interrupt = crate::interrupt::typelevel::[<LPI2C $n>];
            }
        }
    };
}

/// SCL pin trait.
pub trait SclPin<Instance>: GpioPin + sealed::SealedPin<Instance> + PeripheralType {
    fn mux(&self);
}

/// SDA pin trait.
pub trait SdaPin<Instance>: GpioPin + sealed::SealedPin<Instance> + PeripheralType {
    fn mux(&self);
}

/// SCLS pin trait. (SCL secondary)
pub trait SclsPin<Instance>: GpioPin + sealed::SealedPin<Instance> + PeripheralType {
    fn mux(&self);
}

/// SDAS pin trait. (SDA secondary)
pub trait SdasPin<Instance>: GpioPin + sealed::SealedPin<Instance> + PeripheralType {
    fn mux(&self);
}

/// HREQ pin trait. (Host request)
pub trait HreqPin<Instance>: GpioPin + sealed::SealedPin<Instance> + PeripheralType {
    fn mux(&self);
}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: sealed::Sealed {}

/// Async driver mode.
#[allow(private_bounds)]
pub trait AsyncMode: sealed::Sealed + Mode {}

/// Blocking mode.
pub struct Blocking;
impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

/// Async mode.
pub struct Async;
impl sealed::Sealed for Async {}
impl Mode for Async {}
impl AsyncMode for Async {}

/// DMA mode.
pub struct Dma<'d> {
    tx_dma: DmaChannel<'d>,
    rx_dma: DmaChannel<'d>,
    rx_request: DmaRequest,
    tx_request: DmaRequest,
}
impl sealed::Sealed for Dma<'_> {}
impl Mode for Dma<'_> {}
impl AsyncMode for Dma<'_> {}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_lpi2c_pin {
    ($pin:ident, $peri:ident, $fn:ident, $trait:ident) => {
        impl crate::i2c::sealed::SealedPin<crate::peripherals::$peri> for crate::peripherals::$pin {}

        impl crate::i2c::$trait<crate::peripherals::$peri> for crate::peripherals::$pin {
            fn mux(&self) {
                use crate::gpio::SealedPin;
                self.set_pull(crate::gpio::Pull::Disabled);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Double.into());
                self.set_function(crate::pac::port::Mux::$fn);
                self.set_enable_input_buffer(true);
            }
        }
    };
}
