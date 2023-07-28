#![macro_use]

//! Programmable Peripheral Interconnect (PPI/DPPI) driver.
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

use core::marker::PhantomData;
use core::ptr::NonNull;

use embassy_hal_internal::{impl_peripheral, into_ref, PeripheralRef};

use crate::{peripherals, Peripheral};

#[cfg_attr(feature = "_dppi", path = "dppi.rs")]
#[cfg_attr(feature = "_ppi", path = "ppi.rs")]
mod _version;
pub(crate) use _version::*;

/// PPI channel driver.
pub struct Ppi<'d, C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize> {
    ch: PeripheralRef<'d, C>,
    #[cfg(feature = "_dppi")]
    events: [Event<'d>; EVENT_COUNT],
    #[cfg(feature = "_dppi")]
    tasks: [Task<'d>; TASK_COUNT],
}

/// PPI channel group driver.
pub struct PpiGroup<'d, G: Group> {
    g: PeripheralRef<'d, G>,
}

impl<'d, G: Group> PpiGroup<'d, G> {
    /// Create a new PPI group driver.
    ///
    /// The group is initialized as containing no channels.
    pub fn new(g: impl Peripheral<P = G> + 'd) -> Self {
        into_ref!(g);

        let r = regs();
        let n = g.number();
        r.chg[n].write(|w| unsafe { w.bits(0) });

        Self { g }
    }

    /// Add a PPI channel to this group.
    ///
    /// If the channel is already in the group, this is a no-op.
    pub fn add_channel<C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize>(
        &mut self,
        ch: &Ppi<'_, C, EVENT_COUNT, TASK_COUNT>,
    ) {
        let r = regs();
        let ng = self.g.number();
        let nc = ch.ch.number();
        r.chg[ng].modify(|r, w| unsafe { w.bits(r.bits() | 1 << nc) });
    }

    /// Remove a PPI channel from this group.
    ///
    /// If the channel is already not in the group, this is a no-op.
    pub fn remove_channel<C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize>(
        &mut self,
        ch: &Ppi<'_, C, EVENT_COUNT, TASK_COUNT>,
    ) {
        let r = regs();
        let ng = self.g.number();
        let nc = ch.ch.number();
        r.chg[ng].modify(|r, w| unsafe { w.bits(r.bits() & !(1 << nc)) });
    }

    /// Enable all the channels in this group.
    pub fn enable_all(&mut self) {
        let n = self.g.number();
        regs().tasks_chg[n].en.write(|w| unsafe { w.bits(1) });
    }

    /// Disable all the channels in this group.
    pub fn disable_all(&mut self) {
        let n = self.g.number();
        regs().tasks_chg[n].dis.write(|w| unsafe { w.bits(1) });
    }

    /// Get a reference to the "enable all" task.
    ///
    /// When triggered, it will enable all the channels in this group.
    pub fn task_enable_all(&self) -> Task<'d> {
        let n = self.g.number();
        Task::from_reg(&regs().tasks_chg[n].en)
    }

    /// Get a reference to the "disable all" task.
    ///
    /// When triggered, it will disable all the channels in this group.
    pub fn task_disable_all(&self) -> Task<'d> {
        let n = self.g.number();
        Task::from_reg(&regs().tasks_chg[n].dis)
    }
}

impl<'d, G: Group> Drop for PpiGroup<'d, G> {
    fn drop(&mut self) {
        let r = regs();
        let n = self.g.number();
        r.chg[n].write(|w| unsafe { w.bits(0) });
    }
}

#[cfg(feature = "_dppi")]
const REGISTER_DPPI_CONFIG_OFFSET: usize = 0x80 / core::mem::size_of::<u32>();

/// Represents a task that a peripheral can do.
///
/// When a task is subscribed to a PPI channel, it will run when the channel is triggered by
/// a published event.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Task<'d>(NonNull<u32>, PhantomData<&'d ()>);

impl<'d> Task<'d> {
    /// Create a new `Task` from a task register pointer
    ///
    /// # Safety
    ///
    /// `ptr` must be a pointer to a valid `TASKS_*` register from an nRF peripheral.
    pub unsafe fn new_unchecked(ptr: NonNull<u32>) -> Self {
        Self(ptr, PhantomData)
    }

    /// Triggers this task.
    pub fn trigger(&mut self) {
        unsafe { self.0.as_ptr().write_volatile(1) };
    }

    pub(crate) fn from_reg<T>(reg: &T) -> Self {
        Self(
            unsafe { NonNull::new_unchecked(reg as *const _ as *mut _) },
            PhantomData,
        )
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
unsafe impl Send for Task<'_> {}

/// Represents an event that a peripheral can publish.
///
/// An event can be set to publish on a PPI channel when the event happens.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Event<'d>(NonNull<u32>, PhantomData<&'d ()>);

impl<'d> Event<'d> {
    /// Create a new `Event` from an event register pointer
    ///
    /// # Safety
    ///
    /// `ptr` must be a pointer to a valid `EVENTS_*` register from an nRF peripheral.
    pub unsafe fn new_unchecked(ptr: NonNull<u32>) -> Self {
        Self(ptr, PhantomData)
    }

    pub(crate) fn from_reg<T>(reg: &'d T) -> Self {
        Self(
            unsafe { NonNull::new_unchecked(reg as *const _ as *mut _) },
            PhantomData,
        )
    }

    /// Describes whether this Event is currently in a triggered state.
    pub fn is_triggered(&self) -> bool {
        unsafe { self.0.as_ptr().read_volatile() == 1 }
    }

    /// Clear the current register's triggered state, reverting it to 0.
    pub fn clear(&mut self) {
        unsafe { self.0.as_ptr().write_volatile(0) };
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
unsafe impl Send for Event<'_> {}

// ======================
//       traits

pub(crate) mod sealed {
    pub trait Channel {}
    pub trait Group {}
}

/// Interface for PPI channels.
pub trait Channel: sealed::Channel + Peripheral<P = Self> + Sized + 'static {
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
pub trait Group: sealed::Group + Peripheral<P = Self> + Into<AnyGroup> + Sized + 'static {
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

        impl From<peripherals::$type> for crate::ppi::AnyGroup {
            fn from(val: peripherals::$type) -> Self {
                crate::ppi::Group::degrade(val)
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
