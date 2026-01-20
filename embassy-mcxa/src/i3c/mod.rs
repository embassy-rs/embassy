//! I3C Support

use embassy_hal_internal::PeripheralType;
use maitake_sync::WaitCell;
use paste::paste;

use crate::clocks::ClockError;
use crate::gpio::{GpioPin, SealedPin};
use crate::{interrupt, pac};

pub mod controller;

/// Error information type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Clock configuration error.
    ClockSetup(ClockError),
    /// Underrun error
    Underrun,
    /// Not Acknowledge error
    Nack,
    /// Write abort error
    WriteAbort,
    /// Terminate error
    Terminate,
    /// High data rate parity flag
    HighDataRateParity,
    /// High data rate CRC error
    HighDataRateCrc,
    /// Overread error
    Overread,
    /// Overwrite error
    Overwrite,
    /// Message error
    Message,
    /// Invalid request error
    InvalidRequest,
    /// Timeout error
    Timeout,
    /// Address out of range.
    AddressOutOfRange(u8),
    /// Invalid write buffer length.
    InvalidWriteBufferLength,
    /// Invalid read buffer length.
    InvalidReadBufferLength,
    /// User provided an invalid configuration
    InvalidConfiguration,
    /// Other internal errors or unexpected state.
    Other,
}

/// I3C interrupt handler.
pub struct InterruptHandler;

impl interrupt::typelevel::Handler<interrupt::typelevel::I3C0> for InterruptHandler {
    unsafe fn on_interrupt() {
        let status = info().regs().mintmasked().read();

        if status.nowmaster().bit_is_set()
            || status.complete().bit_is_set()
            || status.mctrldone().bit_is_set()
            || status.slvstart().bit_is_set()
            || status.errwarn().bit_is_set()
            || status.rxpend().bit_is_set()
            || status.txnotfull().bit_is_set()
        {
            info().regs().mintclr().write(|w| {
                w.nowmaster()
                    .clear_bit_by_one()
                    .complete()
                    .clear_bit_by_one()
                    .mctrldone()
                    .clear_bit_by_one()
                    .slvstart()
                    .clear_bit_by_one()
                    .errwarn()
                    .clear_bit_by_one()
                    .rxpend()
                    .clear_bit_by_one()
                    .txnotfull()
                    .clear_bit_by_one()
            });

            info().wait_cell().wake();
        }
    }
}

mod sealed {
    /// Seal a trait
    pub trait Sealed {}
}

struct Info {
    regs: *const pac::i3c0::RegisterBlock,
    wait_cell: WaitCell,
}

unsafe impl Sync for Info {}

impl Info {
    #[inline(always)]
    fn regs(&self) -> &'static pac::i3c0::RegisterBlock {
        unsafe { &*self.regs }
    }

    #[inline(always)]
    fn wait_cell(&self) -> &WaitCell {
        &self.wait_cell
    }
}

fn info() -> &'static Info {
    static INFO: Info = Info {
        regs: pac::I3c0::ptr(),
        wait_cell: WaitCell::new(),
    };
    &INFO
}

/// SCL pin trait.
pub trait SclPin<I3C0>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// SDA pin trait.
pub trait SdaPin<I3C0>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: sealed::Sealed {}

/// Blocking mode.
pub struct Blocking;
impl sealed::Sealed for Blocking {}
impl Mode for Blocking {}

/// Async mode.
pub struct Async;
impl sealed::Sealed for Async {}
impl Mode for Async {}

macro_rules! impl_pin {
    ($pin:ident, $fn:ident, $trait:ident) => {
        paste! {
            impl sealed::Sealed for crate::peripherals::$pin {}

            impl $trait<crate::peripherals::I3C0> for crate::peripherals::$pin {
                fn mux(&self) {
                    self.set_pull(crate::gpio::Pull::Disabled);
                    self.set_slew_rate(crate::gpio::SlewRate::Fast.into());
                    self.set_drive_strength(crate::gpio::DriveStrength::Double.into());
                    self.set_function(crate::pac::port0::pcr0::Mux::$fn);
                    self.set_enable_input_buffer();
                }
            }
        }
    };
}

// impl_pin!(P0_2, Mux10, PurPin); REVISIT: what is this for?
impl_pin!(P0_17, Mux10, SclPin);
impl_pin!(P0_18, Mux10, SdaPin);
impl_pin!(P1_8, Mux10, SdaPin);
impl_pin!(P1_9, Mux10, SclPin);
// impl_pin!(P1_11, Mux10, PurPin); REVISIT: what is this for?
#[cfg(feature = "sosc-as-gpio")]
impl_pin!(P1_30, Mux10, SdaPin);
#[cfg(feature = "sosc-as-gpio")]
impl_pin!(P1_31, Mux10, SclPin);
