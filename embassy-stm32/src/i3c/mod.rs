//! Improved inter-integrated circuit (I3C)
//!
//! I3C controller driver for STM32N6, STM32H5, and STM32U3. Originally based on the
//! STM32N6 HAL I3C driver (`stm32n6xx_hal_i3c`). Supports blocking controller
//! operations: direct CCC, private transfers, broadcast CCC, and dynamic address
//! assignment (ENTDAA).

mod config;
pub mod controller;

pub use config::{BusTiming, Config, ControllerOptions};
pub use controller::Controller;
use embassy_hal_internal::PeripheralType;

use crate::pac::i3c::I3c as Regs;
use crate::peripherals;
use crate::rcc::{RccInfo, RccPeripheral, SealedRccPeripheral};

// ---- Control register constants (from STM32 LL/HAL) ----

pub(crate) const DIR_READ: u32 = 1 << 16;
pub(crate) const DIR_WRITE: u32 = 0;
pub(crate) const GENERATE_STOP: u32 = 1 << 31;
pub(crate) const GENERATE_RESTART: u32 = 0;

pub(crate) const MTYPE_PRIVATE: u32 = 1 << 28;
pub(crate) const MTYPE_DIRECT: u32 = 3 << 27;
pub(crate) const MTYPE_CCC: u32 = 6 << 27;

pub(crate) const DIRECT_WITHOUT_DEFBYTE_RESTART: u32 = MTYPE_DIRECT;
pub(crate) const DIRECT_WITHOUT_DEFBYTE_STOP: u32 = MTYPE_DIRECT | GENERATE_STOP;
pub(crate) const PRIVATE_WITHOUT_ARB_RESTART: u32 = MTYPE_PRIVATE | 4;
pub(crate) const PRIVATE_WITHOUT_ARB_STOP: u32 = MTYPE_PRIVATE | 4 | GENERATE_STOP;

/// I3C error.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Address not acknowledged.
    AddressNack,
    /// Data not acknowledged.
    DataNack,
    /// FIFO overrun/underrun.
    FifoOverrun,
    /// Protocol error.
    Protocol,
    /// Data hand-off error during controller-role transfer.
    DataHandOff,
    /// Transfer size mismatch.
    Size,
    /// Invalid parameter.
    InvalidParam,
    /// Dynamic address out of range.
    AddressOutOfRange,
    /// Timeout.
    Timeout,
    /// Other/unclassified error.
    Other,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AddressNack => f.write_str("address NACK"),
            Self::DataNack => f.write_str("data NACK"),
            Self::FifoOverrun => f.write_str("FIFO overrun"),
            Self::Protocol => f.write_str("protocol error"),
            Self::DataHandOff => f.write_str("data hand-off error"),
            Self::Size => f.write_str("transfer size mismatch"),
            Self::InvalidParam => f.write_str("invalid parameter"),
            Self::AddressOutOfRange => f.write_str("address out of range"),
            Self::Timeout => f.write_str("timeout"),
            Self::Other => f.write_str("other error"),
        }
    }
}

impl core::error::Error for Error {}

/// Common I3C direct CCC command codes.
pub mod ccc {
    /// Broadcast ENTDAA (enter dynamic address assignment).
    pub const ENTDAA: u8 = 0x07;
    /// Broadcast RSTDAA (reset dynamic address assignment).
    pub const RSTDAA: u8 = 0x06;
    /// Direct GETPID.
    pub const GETPID: u8 = 0x8D;
    /// Direct GETBCR.
    pub const GETBCR: u8 = 0x8E;
    /// Direct GETDCR.
    pub const GETDCR: u8 = 0x8F;
    /// Direct GETSTATUS.
    pub const GETSTATUS: u8 = 0x90;
    /// Direct GETMWL.
    pub const GETMWL: u8 = 0x91;
    /// Direct GETMRL.
    pub const GETMRL: u8 = 0x92;
    /// Direct SETMWL.
    pub const SETMWL: u8 = 0x89;
    /// Direct SETMRL.
    pub const SETMRL: u8 = 0x8A;
}

pub(crate) struct Info {
    pub(crate) regs: Regs,
    pub(crate) rcc: RccInfo,
}

trait SealedInstance: SealedRccPeripheral {
    fn info() -> &'static Info;
}

/// I3C instance trait.
#[allow(private_bounds)]
pub trait Instance: PeripheralType + SealedInstance + RccPeripheral + 'static {
    /// Event interrupt for this instance.
    type EventInterrupt: crate::interrupt::typelevel::Interrupt;
    /// Error interrupt for this instance.
    type ErrorInterrupt: crate::interrupt::typelevel::Interrupt;
}

pin_trait!(SclPin, Instance, @A);
pin_trait!(SdaPin, Instance, @A);

foreach_peripheral!(
    (i3c, $inst:ident) => {
        impl SealedInstance for peripherals::$inst {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: crate::pac::$inst,
                    rcc: crate::peripherals::$inst::RCC_INFO,
                };
                &INFO
            }
        }

        impl Instance for peripherals::$inst {
            type EventInterrupt = crate::_generated::peripheral_interrupts::$inst::EV;
            type ErrorInterrupt = crate::_generated::peripheral_interrupts::$inst::ER;
        }
    };
);
