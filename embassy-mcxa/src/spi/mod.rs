//! LPSPI driver
use embassy_hal_internal::PeripheralType;
use maitake_sync::WaitCell;

use crate::clocks::Gate;
use crate::clocks::periph_helpers::LpspiConfig;
use crate::dma::{DmaChannel, DmaRequest};
use crate::gpio::GpioPin;
use crate::{interrupt, pac};

pub mod controller;
pub(crate) mod sealed {
    /// Seal a trait
    pub trait Sealed {}

    pub trait SealedSpiPin<Instance> {}
}

pub(crate) trait SealedInstance: Gate<MrccPeriphConfig = LpspiConfig> {
    fn info() -> &'static Info;

    /// Clock instance
    const CLOCK_INSTANCE: crate::clocks::periph_helpers::LpspiInstance;
    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
    const TX_DMA_REQUEST: DmaRequest;
    const RX_DMA_REQUEST: DmaRequest;
}

/// SPI Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this SPI instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pub(crate) struct Info {
    pub(crate) regs: pac::lpspi::Lpspi,
    pub(crate) wait_cell: WaitCell,
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::lpspi::Lpspi {
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
macro_rules! impl_lpspi_instance {
    ($n:expr) => {
        paste::paste! {
            impl crate::spi::SealedInstance for crate::peripherals::[<LPSPI $n>] {
                fn info() -> &'static crate::spi::Info {
                    static INFO: crate::spi::Info = crate::spi::Info {
                        regs: crate::pac::[<LPSPI $n>],
                        wait_cell: maitake_sync::WaitCell::new(),
                    };
                    &INFO
                }

                const CLOCK_INSTANCE: crate::clocks::periph_helpers::LpspiInstance
                    = crate::clocks::periph_helpers::LpspiInstance::[<Lpspi $n>];
                const PERF_INT_INCR: fn() = crate::perf_counters::[<incr_interrupt_spi $n>];
                const PERF_INT_WAKE_INCR: fn() = crate::perf_counters::[<incr_interrupt_spi $n _wake>];
                const TX_DMA_REQUEST: DmaRequest = DmaRequest::[<Lpspi $n Tx>];
                const RX_DMA_REQUEST: DmaRequest = DmaRequest::[<Lpspi $n Rx>];
            }

            impl crate::spi::Instance for crate::peripherals::[<LPSPI $n>] {
                type Interrupt = crate::interrupt::typelevel::[<LPSPI $n>];
            }
        }
    };
}

/// MOSI or data pin 0 during parallel data transfers pin trait.
#[allow(private_bounds)]
pub trait SdiPin<Instance>: GpioPin + sealed::SealedSpiPin<Instance> + PeripheralType {
    fn mux(&self);
}

/// MISO or data pin 1 during parallel data transfers pin trait.
#[allow(private_bounds)]
pub trait SdoPin<Instance>: GpioPin + sealed::SealedSpiPin<Instance> + PeripheralType {
    fn mux(&self);
}

/// SCK pin trait.
#[allow(private_bounds)]
pub trait SckPin<Instance>: GpioPin + sealed::SealedSpiPin<Instance> + PeripheralType {
    fn mux(&self);
}

/// Peripheral chip select pin trait.
#[allow(private_bounds)]
pub trait Pcs0Pin<Instance>: GpioPin + sealed::SealedSpiPin<Instance> + PeripheralType {
    fn mux(&self);
}

/// Peripheral chip select or host request pin trait.
#[allow(private_bounds)]
pub trait Pcs1Pin<Instance>: GpioPin + sealed::SealedSpiPin<Instance> + PeripheralType {
    fn mux(&self);
}

/// Peripheral chip select or data pin 2 during parallel data transfers pin trait.
#[allow(private_bounds)]
pub trait Pcs2Pin<Instance>: GpioPin + sealed::SealedSpiPin<Instance> + PeripheralType {
    fn mux(&self);
}

/// Peripheral chip select or data pin 3 during parallel data transfers pin trait.
#[allow(private_bounds)]
pub trait Pcs3Pin<Instance>: GpioPin + sealed::SealedSpiPin<Instance> + PeripheralType {
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
macro_rules! impl_spi_pin {
    ($pin:ident, $peri:ident, $fn:ident, $trait:ident) => {
        impl crate::spi::sealed::SealedSpiPin<crate::peripherals::$peri> for crate::peripherals::$pin {}

        impl crate::spi::$trait<crate::peripherals::$peri> for crate::peripherals::$pin {
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
