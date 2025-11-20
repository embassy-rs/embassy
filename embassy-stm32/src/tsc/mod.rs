//! TSC Peripheral Interface
//!
//! This module provides an interface for the Touch Sensing Controller (TSC) peripheral.
//! It supports both blocking and async modes of operation, as well as different TSC versions (v1, v2, v3).
//!
//! # Key Concepts
//!
//! - **Pin Groups**: TSC pins are organized into groups, each containing up to four IOs.
//! - **Pin Roles**: Each pin in a group can have a role: Channel, Sample, or Shield.
//! - **Acquisition Banks**: Used for efficient, repeated TSC acquisitions on specific sets of pins.
//!
//! # Example (stm32)
//!
//! ```rust
//! let device_config = embassy_stm32::Config::default();
//! let context = embassy_stm32::init(device_config);
//!
//! let config = tsc::Config {
//!     ct_pulse_high_length: ChargeTransferPulseCycle::_4,
//!     ct_pulse_low_length: ChargeTransferPulseCycle::_4,
//!     spread_spectrum: false,
//!     spread_spectrum_deviation: SSDeviation::new(2).unwrap(),
//!     spread_spectrum_prescaler: false,
//!     pulse_generator_prescaler: PGPrescalerDivider::_16,
//!     max_count_value: MaxCount::_255,
//!     io_default_mode: false,
//!     synchro_pin_polarity: false,
//!     acquisition_mode: false,
//!     max_count_interrupt: false,
//! };
//!
//! let mut g2: PinGroupWithRoles<embassy_stm32::peripherals::TSC, G2> = PinGroupWithRoles::new();
//! g2.set_io1::<tsc_pin_roles::Sample>(context.PB4);
//! let sensor_pin = g2.set_io2::<tsc_pin_roles::Channel>(context.PB5);
//!
//! let pin_groups = PinGroups {
//!     g2: Some(g2.pin_group),
//!     ..Default::default()
//! };
//!
//! let mut touch_controller = tsc::Tsc::new_blocking(
//!     context.TSC,
//!     pin_groups,
//!     config,
//! ).unwrap();
//!
//! let discharge_delay = 5; // ms
//!
//! loop {
//!     touch_controller.set_active_channels_mask(sensor_pin.pin.into());
//!     touch_controller.start();
//!     touch_controller.poll_for_acquisition();
//!     touch_controller.discharge_io(true);
//!     Timer::after_millis(discharge_delay).await;
//!
//!     match touch_controller.group_get_status(sensor_pin.pin.group()) {
//!         GroupStatus::Complete => {
//!             let group_val = touch_controller.group_get_value(sensor_pin.pin.group());
//!             // Process the touch value
//!             // ...
//!         }
//!         GroupStatus::Ongoing => {
//!             // Handle ongoing acquisition
//!             // ...
//!         }
//!     }
//! }
//! ```
//!
//! # Async Usage
//!
//! For async operation, use `Tsc::new_async` and `pend_for_acquisition` instead of polling.

#![macro_use]

/// Configuration structures and enums for the TSC peripheral.
pub mod config;

/// Definitions and implementations for TSC pin groups.
pub mod pin_groups;

/// Definitions and implementations for individual TSC I/O pins.
pub mod io_pin;

/// Structures and implementations for TSC acquisition banks.
pub mod acquisition_banks;

/// Core implementation of the TSC (Touch Sensing Controller) driver.
pub mod tsc;

/// Type definitions used throughout the TSC module.
pub mod types;

/// Error types and definitions for the TSC module.
pub mod errors;

use core::marker::PhantomData;

pub use acquisition_banks::*;
pub use config::*;
use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
pub use errors::*;
pub use io_pin::*;
pub use pin_groups::*;
pub use tsc::*;
pub use types::*;

use crate::rcc::RccPeripheral;
use crate::{interrupt, peripherals};

#[cfg(tsc_v1)]
const TSC_NUM_GROUPS: usize = 6;
#[cfg(tsc_v2)]
const TSC_NUM_GROUPS: usize = 7;
#[cfg(tsc_v3)]
const TSC_NUM_GROUPS: usize = 8;

/// Error type defined for TSC
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Test error for TSC
    Test,
}

/// TSC interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::regs().ier().write(|w| w.set_eoaie(false));
        T::waker().wake();
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> crate::pac::tsc::Tsc;
    fn waker() -> &'static AtomicWaker;
}

/// TSC instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral {
    /// Interrupt for this TSC instance
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, tsc, TSC, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::tsc::Tsc {
                crate::pac::$inst
            }
            fn waker() -> &'static AtomicWaker {
                static WAKER: AtomicWaker = AtomicWaker::new();
                &WAKER
            }
        }
    };
);
