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

        if status.nowmaster()
            || status.complete()
            || status.mctrldone()
            || status.slvstart()
            || status.errwarn()
            || status.rxpend()
            || status.txnotfull()
        {
            info().regs().mintclr().write(|w| {
                w.set_nowmaster(true);
                w.set_complete(true);
                w.set_mctrldone(true);
                w.set_slvstart(true);
                w.set_errwarn(true);
                w.set_rxpend(true);
                w.set_txnotfull(true);
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
    regs: pac::i3c0::I3c0,
    wait_cell: WaitCell,
}

unsafe impl Sync for Info {}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::i3c0::I3c0 {
        self.regs
    }

    #[inline(always)]
    fn wait_cell(&self) -> &WaitCell {
        &self.wait_cell
    }
}

fn info() -> &'static Info {
    static INFO: Info = Info {
        regs: pac::I3C0,
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
                    self.set_function(crate::pac::port::vals::Mux::$fn);
                    self.set_enable_input_buffer();
                }
            }
        }
    };
}

// impl_pin!(P0_2, MUX10, PurPin); REVISIT: what is this for?
impl_pin!(P0_17, MUX10, SclPin);
impl_pin!(P0_18, MUX10, SdaPin);
impl_pin!(P1_8, MUX10, SdaPin);
impl_pin!(P1_9, MUX10, SclPin);
// impl_pin!(P1_11, MUX10, PurPin); REVISIT: what is this for?
#[cfg(feature = "sosc-as-gpio")]
impl_pin!(P1_30, MUX10, SdaPin);
#[cfg(feature = "sosc-as-gpio")]
impl_pin!(P1_31, MUX10, SclPin);
