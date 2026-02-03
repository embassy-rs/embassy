//! I3C Support

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use maitake_sync::WaitCell;
use paste::paste;

use crate::clocks::Gate;
use crate::clocks::periph_helpers::I3cConfig;
use crate::gpio::{GpioPin, SealedPin};
use crate::{interrupt, pac};

pub mod controller;

/// I3C interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let status = T::info().regs().mintmasked().read();
        T::PERF_INT_INCR();

        if status.nowmaster()
            || status.complete()
            || status.mctrldone()
            || status.slvstart()
            || status.errwarn()
            || status.rxpend()
            || status.txnotfull()
        {
            T::info().regs().mintclr().write(|w| {
                w.set_nowmaster(true);
                w.set_complete(true);
                w.set_mctrldone(true);
                w.set_slvstart(true);
                w.set_errwarn(true);
                w.set_rxpend(true);
                w.set_txnotfull(true);
            });

            T::PERF_INT_WAKE_INCR();
            T::info().wait_cell().wake();
        }
    }
}

mod sealed {
    /// Seal a trait
    pub trait Sealed {}
}

trait SealedInstance {
    fn info() -> &'static Info;
}

/// I3C Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send + Gate<MrccPeriphConfig = I3cConfig> {
    /// Interrupt for this I3C instance.
    type Interrupt: interrupt::typelevel::Interrupt;
    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
}

struct Info {
    regs: pac::i3c::I3c,
    wait_cell: WaitCell,
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

macro_rules! impl_instance {
    ($n:literal) => {
        paste! {
            impl SealedInstance for crate::peripherals::[<I3C $n>] {
                fn info() -> &'static Info {
                    static INFO: Info = Info {
                        regs: pac::[<I3C $n>],
                        wait_cell: WaitCell::new(),
                    };
                    &INFO
                }
            }

            impl Instance for crate::peripherals::[<I3C $n>] {
                type Interrupt = crate::interrupt::typelevel::[<I3C $n>];
                const PERF_INT_INCR: fn() = crate::perf_counters::[<incr_interrupt_i3c $n>];
                const PERF_INT_WAKE_INCR: fn() = crate::perf_counters::[<incr_interrupt_i3c $n _wake>];
            }
        }
    };
}

impl_instance!(0);

/// SCL pin trait.
pub trait SclPin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
    fn mux(&self);
}

/// SDA pin trait.
pub trait SdaPin<T: Instance>: GpioPin + sealed::Sealed + PeripheralType {
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
    ($pin:ident, $peri:ident, $fn:ident, $trait:ident) => {
        paste! {
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
        }
    };
}

// impl_pin!(P0_2, I3C0, MUX10, PurPin); REVISIT: what is this for?
impl_pin!(P0_17, I3C0, MUX10, SclPin);
impl_pin!(P0_18, I3C0, MUX10, SdaPin);
impl_pin!(P1_8, I3C0, MUX10, SdaPin);
impl_pin!(P1_9, I3C0, MUX10, SclPin);
// impl_pin!(P1_11, I3C0, MUX10, PurPin); REVISIT: what is this for?
#[cfg(feature = "sosc-as-gpio")]
impl_pin!(P1_30, I3C0, MUX10, SdaPin);
#[cfg(feature = "sosc-as-gpio")]
impl_pin!(P1_31, I3C0, MUX10, SclPin);
