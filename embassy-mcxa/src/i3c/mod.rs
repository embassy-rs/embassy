//! I3C Support

use embassy_hal_internal::PeripheralType;
use maitake_sync::WaitCell;

use crate::clocks::Gate;
use crate::clocks::periph_helpers::I3cConfig;
use crate::dma::{DmaChannel, DmaRequest};
use crate::gpio::GpioPin;
use crate::{interrupt, pac};

pub mod controller;

pub(crate) mod sealed {
    /// Seal a trait
    pub trait Sealed {}
}

pub(crate) trait SealedInstance: Gate<MrccPeriphConfig = I3cConfig> {
    fn info() -> &'static Info;

    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
    const TX_DMA_REQUEST: DmaRequest;
    const RX_DMA_REQUEST: DmaRequest;
}

/// I3C Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this I3C instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pub(crate) struct Info {
    pub(crate) regs: pac::i3c::I3c,
    pub(crate) wait_cell: WaitCell,
}

unsafe impl Sync for Info {}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::i3c::I3c {
        self.regs
    }

    #[inline(always)]
    fn wait_cell(&self) -> &WaitCell {
        &self.wait_cell
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_i3c_instance {
    ($n:literal) => {
        paste::paste! {
            impl crate::i3c::SealedInstance for crate::peripherals::[<I3C $n>] {
                fn info() -> &'static crate::i3c::Info {
                    static INFO: crate::i3c::Info = crate::i3c::Info {
                        regs: crate::pac::[<I3C $n>],
                        wait_cell: maitake_sync::WaitCell::new(),
                    };
                    &INFO
                }

                const TX_DMA_REQUEST: DmaRequest = DmaRequest::[<I3C $n Tx>];
                const RX_DMA_REQUEST: DmaRequest = DmaRequest::[<I3C $n Rx>];
                const PERF_INT_INCR: fn() = crate::perf_counters::[<incr_interrupt_i3c $n>];
                const PERF_INT_WAKE_INCR: fn() = crate::perf_counters::[<incr_interrupt_i3c $n _wake>];
            }

            impl crate::i3c::Instance for crate::peripherals::[<I3C $n>] {
                type Interrupt = crate::interrupt::typelevel::[<I3C $n>];
            }
        }
    };
}

/// SCL pin trait.
pub trait SclPin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// SDA pin trait.
pub trait SdaPin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// SDA1 pin (for I3C multi-lane) trait.
pub trait Sda1Pin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// SDA2 pin (for I3C multi-lane) trait.
pub trait Sda2Pin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// SDA3 pin (for I3C multi-lane) trait.
pub trait Sda3Pin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// PUR pin trait. (Pull up resistance)
pub trait PurPin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
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
    tx_request: DmaRequest,

    rx_dma: DmaChannel<'d>,
    rx_request: DmaRequest,
}
impl sealed::Sealed for Dma<'_> {}
impl Mode for Dma<'_> {}
impl AsyncMode for Dma<'_> {}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_i3c_pin {
    ($pin:ident, $peri:ident, $fn:ident, $trait:ident) => {
        paste::paste! {
            impl crate::i3c::sealed::Sealed for crate::peripherals::$pin {}

            impl crate::i3c::$trait<crate::peripherals::$peri> for crate::peripherals::$pin {
                fn mux(&self) {
                    use crate::gpio::SealedPin;
                    self.set_pull(crate::gpio::Pull::Disabled);
                    self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                    self.set_drive_strength(crate::gpio::DriveStrength::Double.into());
                    self.set_function(crate::pac::port::Mux::$fn);
                    self.set_enable_input_buffer(true);
                }
            }
        }
    };
}
