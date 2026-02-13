//! I2C Support

use embassy_hal_internal::PeripheralType;
use maitake_sync::WaitCell;
use paste::paste;

use crate::clocks::Gate;
use crate::clocks::periph_helpers::Lpi2cConfig;
use crate::dma::{Channel, DmaChannel};
use crate::gpio::{GpioPin, SealedPin};
use crate::{interrupt, pac};

pub mod controller;
pub mod target;

mod sealed {
    /// Seal a trait
    pub trait Sealed {}
}

trait SealedInstance {
    fn info() -> &'static Info;
}

/// I2C Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send + Gate<MrccPeriphConfig = Lpi2cConfig> {
    /// Interrupt for this I2C instance.
    type Interrupt: interrupt::typelevel::Interrupt;
    /// Clock instance
    const CLOCK_INSTANCE: crate::clocks::periph_helpers::Lpi2cInstance;
    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
}

struct Info {
    regs: pac::lpi2c::Lpi2c,
    wait_cell: WaitCell,
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

macro_rules! impl_instance {
    ($($n:literal),*) => {
        $(
            paste!{
                impl SealedInstance for crate::peripherals::[<LPI2C $n>] {
                    fn info() -> &'static Info {
                        static INFO: Info = Info {
                            regs: pac::[<LPI2C $n>],
                            wait_cell: WaitCell::new(),
                        };
                        &INFO
                    }
                }

                impl Instance for crate::peripherals::[<LPI2C $n>] {
                    type Interrupt = crate::interrupt::typelevel::[<LPI2C $n>];
                    const CLOCK_INSTANCE: crate::clocks::periph_helpers::Lpi2cInstance
                        = crate::clocks::periph_helpers::Lpi2cInstance::[<Lpi2c $n>];
                    const PERF_INT_INCR: fn() = crate::perf_counters::[<incr_interrupt_i2c $n>];
                    const PERF_INT_WAKE_INCR: fn() = crate::perf_counters::[<incr_interrupt_i2c $n _wake>];
                }
            }
        )*
    };
}

impl_instance!(0, 1, 2, 3);

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

impl_dma!(LPI2C0, 11, 12);
impl_dma!(LPI2C1, 13, 14);
impl_dma!(LPI2C2, 3, 4);
impl_dma!(LPI2C3, 5, 6);

/// SCL pin trait.
pub trait SclPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// SDA pin trait.
pub trait SdaPin<Instance>: GpioPin + sealed::Sealed + PeripheralType {
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

impl_pin!(P0_16, LPI2C0, MUX2, SdaPin);
impl_pin!(P0_17, LPI2C0, MUX2, SclPin);
impl_pin!(P0_18, LPI2C0, MUX2, SclPin);
impl_pin!(P0_19, LPI2C0, MUX2, SdaPin);
impl_pin!(P1_0, LPI2C1, MUX3, SdaPin);
impl_pin!(P1_1, LPI2C1, MUX3, SclPin);
impl_pin!(P1_2, LPI2C1, MUX3, SdaPin);
impl_pin!(P1_3, LPI2C1, MUX3, SclPin);
impl_pin!(P1_8, LPI2C2, MUX3, SdaPin);
impl_pin!(P1_9, LPI2C2, MUX3, SclPin);
impl_pin!(P1_10, LPI2C2, MUX3, SdaPin);
impl_pin!(P1_11, LPI2C2, MUX3, SclPin);
impl_pin!(P1_12, LPI2C1, MUX2, SdaPin);
impl_pin!(P1_13, LPI2C1, MUX2, SclPin);
impl_pin!(P1_14, LPI2C1, MUX2, SclPin);
impl_pin!(P1_15, LPI2C1, MUX2, SdaPin);

#[cfg(feature = "sosc-as-gpio")]
impl_pin!(P1_30, LPI2C0, MUX3, SdaPin);
#[cfg(feature = "sosc-as-gpio")]
impl_pin!(P1_31, LPI2C0, MUX3, SclPin);

impl_pin!(P3_27, LPI2C3, MUX2, SclPin);
impl_pin!(P3_28, LPI2C3, MUX2, SdaPin);
// impl_pin!(P3_29, LPI2C3, MUX2, HreqPin); What is this HREQ pin?
impl_pin!(P3_30, LPI2C3, MUX2, SclPin);
impl_pin!(P3_31, LPI2C3, MUX2, SdaPin);
impl_pin!(P4_2, LPI2C2, MUX2, SdaPin);
impl_pin!(P4_3, LPI2C0, MUX2, SclPin);
impl_pin!(P4_4, LPI2C2, MUX2, SdaPin);
impl_pin!(P4_5, LPI2C0, MUX2, SclPin);
// impl_pin!(P4_6, LPI2C0, MUX2, HreqPin); What is this HREQ pin?
