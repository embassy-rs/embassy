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

use embassy_hal_internal::{Peri, PeripheralType};

use crate::pac::common::{RW, Reg, W};
use crate::pac::{self};

#[cfg_attr(feature = "_dppi", path = "dppi.rs")]
#[cfg_attr(feature = "_ppi", path = "ppi.rs")]
mod _version;

#[allow(unused_imports)]
pub(crate) use _version::*;

/// PPI channel driver.
pub struct Ppi<'d, const EVENT_COUNT: usize, const TASK_COUNT: usize> {
    ch: ChannelNumber,
    #[cfg(feature = "_dppi")]
    events: [Event<'d>; EVENT_COUNT],
    #[cfg(feature = "_dppi")]
    tasks: [Task<'d>; TASK_COUNT],
    _phantom: PhantomData<&'d mut ()>,
}

/// PPI channel group driver.
pub struct PpiGroup<'d> {
    g: GroupNumber,
    _phantom: PhantomData<&'d mut ()>,
}

impl<'d> PpiGroup<'d> {
    /// Create a new PPI group driver.
    ///
    /// The group is initialized as containing no channels.
    pub fn new(g: Peri<'d, impl Group>) -> Self {
        Self::new_inner(GroupNumber::new(g))
    }

    pub(crate) fn new_inner(g: GroupNumber) -> Self {
        g.regs().chg(g.number()).write(|_| ());

        Self {
            g,
            _phantom: PhantomData,
        }
    }

    /// Enable all the channels in this group.
    pub fn enable_all(&mut self) {
        let n = self.g.number();
        self.g.regs().tasks_chg(n).en().write_value(1);
    }

    /// Disable all the channels in this group.
    pub fn disable_all(&mut self) {
        let n = self.g.number();
        self.g.regs().tasks_chg(n).dis().write_value(1);
    }

    /// Get a reference to the "enable all" task.
    ///
    /// When triggered, it will enable all the channels in this group.
    pub fn task_enable_all(&self) -> Task<'d> {
        let n = self.g.number();
        Task::from_reg(self.g.regs().tasks_chg(n).en())
    }

    /// Get a reference to the "disable all" task.
    ///
    /// When triggered, it will disable all the channels in this group.
    pub fn task_disable_all(&self) -> Task<'d> {
        let n = self.g.number();
        Task::from_reg(self.g.regs().tasks_chg(n).dis())
    }
}
impl PpiGroup<'static> {
    /// Persist this group's configuration for the rest of the program's lifetime. This method
    /// should be preferred over [`core::mem::forget()`] because the `'static` bound prevents
    /// accidental reuse of the underlying peripheral.
    pub fn persist(self) {
        core::mem::forget(self);
    }
}

impl<'d> Drop for PpiGroup<'d> {
    fn drop(&mut self) {
        let r = self.g.regs();
        let n = self.g.number();
        r.chg(n).write(|_| ());
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

    pub(crate) fn from_reg(reg: Reg<u32, W>) -> Self {
        Self(unsafe { NonNull::new_unchecked(reg.as_ptr()) }, PhantomData)
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

    pub(crate) fn from_reg(reg: Reg<u32, RW>) -> Self {
        Self(unsafe { NonNull::new_unchecked(reg.as_ptr()) }, PhantomData)
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

pub(crate) trait SealedChannel {
    #[cfg(feature = "_dppi")]
    fn regs(&self) -> pac::dppic::Dppic;
}
pub(crate) trait SealedGroup {
    #[cfg(feature = "_dppi")]
    fn regs(&self) -> pac::dppic::Dppic;
    #[cfg(not(feature = "_dppi"))]
    fn regs(&self) -> pac::ppi::Ppi;
}

/// Interface for PPI channels.
#[allow(private_bounds)]
pub trait Channel: SealedChannel + PeripheralType + Sized + 'static {
    /// Returns the number of the channel
    fn number(&self) -> usize;
}

/// Interface for PPI channels that can be configured.
pub trait ConfigurableChannel: Channel {}

/// Interface for PPI channels that cannot be configured.
pub trait StaticChannel: Channel {}

/// Interface for a group of PPI channels.
#[allow(private_bounds)]
pub trait Group: SealedGroup + PeripheralType + Sized + 'static {
    /// Returns the number of the group.
    fn number(&self) -> usize;
}

// ======================
//       channels

/// A (D)PPI channel: its number and, for DPPI, its controller's registers. Plain data, it doesn't
/// own the channel: [`Ppi`] does.
#[derive(Clone, Copy)]
pub(crate) struct ChannelNumber {
    number: u8,
    #[cfg(feature = "_dppi")]
    regs: pac::dppic::Dppic,
}

impl ChannelNumber {
    pub(crate) fn new(ch: Peri<'_, impl Channel>) -> Self {
        Self {
            number: ch.number() as u8,
            #[cfg(feature = "_dppi")]
            regs: ch.regs(),
        }
    }

    pub(crate) fn number(self) -> usize {
        self.number as usize
    }

    #[cfg(feature = "_dppi")]
    fn regs(self) -> pac::dppic::Dppic {
        self.regs
    }
}

#[cfg(not(feature = "_nrf51"))]
macro_rules! impl_ppi_channel {
    ($type:ident, $inst:ident, $number:expr) => {
        impl crate::ppi::SealedChannel for peripherals::$type {
            #[cfg(feature = "_dppi")]
            fn regs(&self) -> pac::dppic::Dppic {
                pac::$inst
            }
        }
        impl crate::ppi::Channel for peripherals::$type {
            fn number(&self) -> usize {
                $number
            }
        }
    };
    ($type:ident, $inst:ident, $number:expr => static) => {
        impl_ppi_channel!($type, $inst, $number);
        impl crate::ppi::StaticChannel for peripherals::$type {}
    };
    ($type:ident, $inst:ident, $number:expr => configurable) => {
        impl_ppi_channel!($type, $inst, $number);
        impl crate::ppi::ConfigurableChannel for peripherals::$type {}
    };
}

// ======================
//       groups

/// A (D)PPI channel group: its number and its controller's registers. Plain data, it doesn't own
/// the group: [`PpiGroup`] does.
#[derive(Clone, Copy)]
pub(crate) struct GroupNumber {
    number: u8,
    #[cfg(feature = "_dppi")]
    regs: pac::dppic::Dppic,
    #[cfg(not(feature = "_dppi"))]
    regs: pac::ppi::Ppi,
}

impl GroupNumber {
    pub(crate) fn new(g: Peri<'_, impl Group>) -> Self {
        Self {
            number: g.number() as u8,
            regs: g.regs(),
        }
    }

    fn number(self) -> usize {
        self.number as usize
    }

    #[cfg(feature = "_dppi")]
    fn regs(self) -> pac::dppic::Dppic {
        self.regs
    }

    #[cfg(not(feature = "_dppi"))]
    fn regs(self) -> pac::ppi::Ppi {
        self.regs
    }
}

macro_rules! impl_ppi_group {
    ($type:ident, $inst:ident, $number:expr) => {
        impl crate::ppi::SealedGroup for crate::peripherals::$type {
            #[cfg(feature = "_dppi")]
            fn regs(&self) -> pac::dppic::Dppic {
                pac::$inst
            }
            #[cfg(not(feature = "_dppi"))]
            fn regs(&self) -> pac::ppi::Ppi {
                pac::$inst
            }
        }
        impl crate::ppi::Group for crate::peripherals::$type {
            fn number(&self) -> usize {
                $number
            }
        }
    };
}
