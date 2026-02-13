//! LPSPI driver
use embassy_hal_internal::PeripheralType;
use maitake_sync::WaitCell;
use paste::paste;

use crate::clocks::Gate;
use crate::clocks::periph_helpers::LpspiConfig;
use crate::dma::{Channel, DmaChannel};
use crate::gpio::{GpioPin, SealedPin};
use crate::{interrupt, pac};

pub mod controller;
mod sealed {
    /// Seal a trait
    pub trait Sealed {}
}

trait SealedInstance {
    fn info() -> &'static Info;
}

/// SPI Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send + Gate<MrccPeriphConfig = LpspiConfig> {
    /// Interrupt for this SPI instance.
    type Interrupt: interrupt::typelevel::Interrupt;
    /// Clock instance
    const CLOCK_INSTANCE: crate::clocks::periph_helpers::LpspiInstance;
    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
}

struct Info {
    regs: pac::lpspi::Lpspi,
    wait_cell: WaitCell,
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

macro_rules! impl_instance {
    ($($n:expr),*) => {
        $(
            paste!{
                impl SealedInstance for crate::peripherals::[<LPSPI $n>] {
                    fn info() -> &'static Info {
                        static INFO: Info = Info {
                            regs: pac::[<LPSPI $n>],
                            wait_cell: WaitCell::new(),
                        };
                        &INFO
                    }
                }

                impl Instance for crate::peripherals::[<LPSPI $n>] {
                    type Interrupt = crate::interrupt::typelevel::[<LPSPI $n>];
                    const CLOCK_INSTANCE: crate::clocks::periph_helpers::LpspiInstance
                        = crate::clocks::periph_helpers::LpspiInstance::[<Lpspi $n>];
                    const PERF_INT_INCR: fn() = crate::perf_counters::[<incr_interrupt_spi $n>];
                    const PERF_INT_WAKE_INCR: fn() = crate::perf_counters::[<incr_interrupt_spi $n _wake>];
                }
            }
        )*
    };
}

impl_instance!(0, 1);

/// RxDma marker trait.
pub trait RxDma<Instance>: PeripheralType + Send + Channel {
    fn request_number(&self) -> u8;
}

/// TxDma marker trait.
pub trait TxDma<Instance>: PeripheralType + Send + Channel {
    fn request_number(&self) -> u8;
}

macro_rules! impl_dma {
    ($peri:ident, $rx:literal, $tx:literal) => {
        impl_dma!($peri, 0, $rx, $tx);
        impl_dma!($peri, 1, $rx, $tx);
        impl_dma!($peri, 2, $rx, $tx);
        impl_dma!($peri, 3, $rx, $tx);
        impl_dma!($peri, 4, $rx, $tx);
        impl_dma!($peri, 5, $rx, $tx);
        impl_dma!($peri, 6, $rx, $tx);
        impl_dma!($peri, 7, $rx, $tx);
    };

    ($peri:ident, $ch:literal, $rx:literal, $tx: literal) => {
        paste! {
            impl RxDma<crate::peripherals::$peri> for crate::peripherals::[<DMA_CH $ch>] {
                #[inline(always)]
                fn request_number(&self) -> u8 {
                    $rx
                }
            }

            impl TxDma<crate::peripherals::$peri> for crate::peripherals::[<DMA_CH $ch>] {
                #[inline(always)]
                fn request_number(&self) -> u8 {
                    $tx
                }
            }
        }
    };
}

impl_dma!(LPSPI0, 15, 16);
impl_dma!(LPSPI1, 17, 18);

/// MOSI pin trait.
pub trait MosiPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// MISO pin trait.
pub trait MisoPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// SCK pin trait.
pub trait SckPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
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
    rx_request_number: u8,
    tx_request_number: u8,
}
impl sealed::Sealed for Dma<'_> {}
impl Mode for Dma<'_> {}
impl AsyncMode for Dma<'_> {}

macro_rules! impl_pin {
    ($pin:ident, $peri:ident, $fn:ident, $trait:ident) => {
        impl sealed::Sealed for crate::peripherals::$pin {}

        impl $trait<crate::peripherals::$peri> for crate::peripherals::$pin {
            fn mux(&self) {
                self.set_pull(crate::gpio::Pull::Disabled);
                self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                self.set_drive_strength(crate::gpio::DriveStrength::Double.into());
                self.set_function(crate::pac::port::vals::Mux::$fn);
                self.set_enable_input_buffer(true);
            }
        }
    };
}

#[cfg(feature = "swd-as-gpio")]
impl_pin!(P0_1, LPSPI0, MUX3, MisoPin);
#[cfg(feature = "swd-swo-as-gpio")]
impl_pin!(P0_2, LPSPI0, MUX3, SckPin);
#[cfg(feature = "jtag-extras-as-gpio")]
impl_pin!(P0_3, LPSPI0, MUX3, MosiPin);

impl_pin!(P1_0, LPSPI0, MUX2, MosiPin);
impl_pin!(P1_1, LPSPI0, MUX2, SckPin);
impl_pin!(P1_2, LPSPI0, MUX2, MisoPin);

impl_pin!(P2_12, LPSPI1, MUX2, SckPin);
impl_pin!(P2_13, LPSPI1, MUX2, MosiPin);
impl_pin!(P2_15, LPSPI1, MUX2, MisoPin);
impl_pin!(P2_16, LPSPI1, MUX2, MisoPin);

impl_pin!(P3_8, LPSPI1, MUX2, MosiPin);
impl_pin!(P3_9, LPSPI1, MUX2, MisoPin);
impl_pin!(P3_10, LPSPI1, MUX2, SckPin);
