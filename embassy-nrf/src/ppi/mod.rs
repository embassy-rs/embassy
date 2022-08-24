#![macro_use]

//! HAL interface for the PPI and DPPI peripheral.
//!
//! The (Distributed) Programmable Peripheral Interconnect interface allows for an autonomous interoperability
//! between peripherals through their events and tasks. There are fixed PPI channels and fully
//! configurable ones. Fixed channels can only connect specific events to specific tasks. For fully
//! configurable channels, it is possible to choose, via software, the event and the task that it
//! will triggered by the event.
//!
//! On nRF52 devices, there is also a fork task endpoint, where the user can configure one more task
//! to be triggered by the same event, even fixed PPI channels have a configurable fork task.
//!
//! The DPPI for nRF53 and nRF91 devices works in a different way. Every channel can support infinitely
//! many tasks and events, but any single task or event can only be coupled with one channel.
//!

use core::ptr::NonNull;

use embassy_hal_common::{impl_peripheral, PeripheralRef};

use crate::{peripherals, Peripheral};

#[cfg(feature = "_dppi")]
mod dppi;
#[cfg(feature = "_ppi")]
mod ppi;

/// An instance of the Programmable peripheral interconnect on nRF devices.
pub struct Ppi<'d, C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize> {
    ch: PeripheralRef<'d, C>,
    #[cfg(feature = "_dppi")]
    events: [Event; EVENT_COUNT],
    #[cfg(feature = "_dppi")]
    tasks: [Task; TASK_COUNT],
}

#[cfg(feature = "_dppi")]
const REGISTER_DPPI_CONFIG_OFFSET: usize = 0x80 / core::mem::size_of::<u32>();

/// Represents a task that a peripheral can do.
///
/// When a task is subscribed to a PPI channel, it will run when the channel is triggered by
/// a published event.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Task(NonNull<u32>);

impl Task {
    /// Create a new `Task` from a task register pointer
    ///
    /// # Safety
    ///
    /// `ptr` must be a pointer to a valid `TASKS_*` register from an nRF peripheral.
    pub unsafe fn new_unchecked(ptr: NonNull<u32>) -> Self {
        Self(ptr)
    }

    pub(crate) fn from_reg<T>(reg: &T) -> Self {
        Self(unsafe { NonNull::new_unchecked(reg as *const _ as *mut _) })
    }

    /// Address of subscription register for this task.
    #[cfg(feature = "_dppi")]
    pub fn subscribe_reg(&self) -> *mut u32 {
        unsafe { self.0.as_ptr().add(REGISTER_DPPI_CONFIG_OFFSET) }
    }
}

/// # Safety
///
/// NonNull is not send, but this event is only allowed to point at registers and those exist in any context on the same core.
unsafe impl Send for Task {}

/// Represents an event that a peripheral can publish.
///
/// An event can be set to publish on a PPI channel when the event happens.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Event(NonNull<u32>);

impl Event {
    /// Create a new `Event` from an event register pointer
    ///
    /// # Safety
    ///
    /// `ptr` must be a pointer to a valid `EVENTS_*` register from an nRF peripheral.
    pub unsafe fn new_unchecked(ptr: NonNull<u32>) -> Self {
        Self(ptr)
    }

    pub(crate) fn from_reg<T>(reg: &T) -> Self {
        Self(unsafe { NonNull::new_unchecked(reg as *const _ as *mut _) })
    }

    /// Address of publish register for this event.
    #[cfg(feature = "_dppi")]
    pub fn publish_reg(&self) -> *mut u32 {
        unsafe { self.0.as_ptr().add(REGISTER_DPPI_CONFIG_OFFSET) }
    }
}

/// # Safety
///
/// NonNull is not send, but this event is only allowed to point at registers and those exist in any context on the same core.
unsafe impl Send for Event {}

// ======================
//       traits

pub(crate) mod sealed {
    pub trait Channel {}
    pub trait Group {}
}

/// Interface for PPI channels.
pub trait Channel: sealed::Channel + Peripheral<P = Self> + Sized {
    /// Returns the number of the channel
    fn number(&self) -> usize;
}

/// Interface for PPI channels that can be configured.
pub trait ConfigurableChannel: Channel + Into<AnyConfigurableChannel> {
    /// Convert into a type erased configurable channel.
    fn degrade(self) -> AnyConfigurableChannel;
}

/// Interface for PPI channels that cannot be configured.
pub trait StaticChannel: Channel + Into<AnyStaticChannel> {
    /// Convert into a type erased static channel.
    fn degrade(self) -> AnyStaticChannel;
}

/// Interface for a group of PPI channels.
pub trait Group: sealed::Group + Sized {
    /// Returns the number of the group.
    fn number(&self) -> usize;
    /// Convert into a type erased group.
    fn degrade(self) -> AnyGroup {
        AnyGroup {
            number: self.number() as u8,
        }
    }
}

// ======================
//       channels

/// The any channel can represent any static channel at runtime.
/// This can be used to have fewer generic parameters in some places.
pub struct AnyStaticChannel {
    pub(crate) number: u8,
}
impl_peripheral!(AnyStaticChannel);
impl sealed::Channel for AnyStaticChannel {}
impl Channel for AnyStaticChannel {
    fn number(&self) -> usize {
        self.number as usize
    }
}
impl StaticChannel for AnyStaticChannel {
    fn degrade(self) -> AnyStaticChannel {
        self
    }
}

/// The any configurable channel can represent any configurable channel at runtime.
/// This can be used to have fewer generic parameters in some places.
pub struct AnyConfigurableChannel {
    pub(crate) number: u8,
}
impl_peripheral!(AnyConfigurableChannel);
impl sealed::Channel for AnyConfigurableChannel {}
impl Channel for AnyConfigurableChannel {
    fn number(&self) -> usize {
        self.number as usize
    }
}
impl ConfigurableChannel for AnyConfigurableChannel {
    fn degrade(self) -> AnyConfigurableChannel {
        self
    }
}

macro_rules! impl_ppi_channel {
    ($type:ident, $number:expr) => {
        impl crate::ppi::sealed::Channel for peripherals::$type {}
        impl crate::ppi::Channel for peripherals::$type {
            fn number(&self) -> usize {
                $number
            }
        }
    };
    ($type:ident, $number:expr => static) => {
        impl_ppi_channel!($type, $number);
        impl crate::ppi::StaticChannel for peripherals::$type {
            fn degrade(self) -> crate::ppi::AnyStaticChannel {
                use crate::ppi::Channel;
                crate::ppi::AnyStaticChannel {
                    number: self.number() as u8,
                }
            }
        }

        impl From<peripherals::$type> for crate::ppi::AnyStaticChannel {
            fn from(val: peripherals::$type) -> Self {
                crate::ppi::StaticChannel::degrade(val)
            }
        }
    };
    ($type:ident, $number:expr => configurable) => {
        impl_ppi_channel!($type, $number);
        impl crate::ppi::ConfigurableChannel for peripherals::$type {
            fn degrade(self) -> crate::ppi::AnyConfigurableChannel {
                use crate::ppi::Channel;
                crate::ppi::AnyConfigurableChannel {
                    number: self.number() as u8,
                }
            }
        }

        impl From<peripherals::$type> for crate::ppi::AnyConfigurableChannel {
            fn from(val: peripherals::$type) -> Self {
                crate::ppi::ConfigurableChannel::degrade(val)
            }
        }
    };
}

// ======================
//       groups

/// A type erased PPI group.
pub struct AnyGroup {
    number: u8,
}
impl_peripheral!(AnyGroup);
impl sealed::Group for AnyGroup {}
impl Group for AnyGroup {
    fn number(&self) -> usize {
        self.number as usize
    }
}

macro_rules! impl_group {
    ($type:ident, $number:expr) => {
        impl sealed::Group for peripherals::$type {}
        impl Group for peripherals::$type {
            fn number(&self) -> usize {
                $number
            }
        }
    };
}

impl_group!(PPI_GROUP0, 0);
impl_group!(PPI_GROUP1, 1);
impl_group!(PPI_GROUP2, 2);
impl_group!(PPI_GROUP3, 3);
#[cfg(not(feature = "nrf51"))]
impl_group!(PPI_GROUP4, 4);
#[cfg(not(feature = "nrf51"))]
impl_group!(PPI_GROUP5, 5);
