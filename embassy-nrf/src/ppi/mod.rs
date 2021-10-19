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

use crate::{pac, peripherals};
use core::marker::PhantomData;
use core::ptr::NonNull;
use embassy::util::Unborrow;
use embassy_hal_common::{unborrow, unsafe_impl_unborrow};

#[cfg(feature = "_dppi")]
mod dppi;
#[cfg(feature = "_ppi")]
mod ppi;

pub struct Ppi<'d, C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize> {
    ch: C,
    #[cfg(feature = "_dppi")]
    events: [Event; EVENT_COUNT],
    #[cfg(feature = "_dppi")]
    tasks: [Task; TASK_COUNT],
    phantom: PhantomData<&'d mut C>,
}

impl<'d, C: Channel + 'd, const EVENT_COUNT: usize, const TASK_COUNT: usize>
    Ppi<'d, C, EVENT_COUNT, TASK_COUNT>
{
    pub fn degrade(self) -> Ppi<'d, AnyChannel, EVENT_COUNT, TASK_COUNT> {
        Ppi {
            ch: AnyChannel {
                number: self.ch.number() as u8,
                #[cfg(feature = "_ppi")]
                has_configurable_task: self.ch.is_task_configurable(),
            },
            #[cfg(feature = "_dppi")]
            events: self.events,
            #[cfg(feature = "_dppi")]
            tasks: self.tasks,
            phantom: PhantomData,
        }
    }

    /// Enables the channel.
    pub fn enable(&mut self) {
        let r = unsafe { &*pac::PPI::ptr() };
        r.chenset
            .write(|w| unsafe { w.bits(1 << self.ch.number()) });
    }

    /// Disables the channel.
    pub fn disable(&mut self) {
        let r = unsafe { &*pac::PPI::ptr() };
        r.chenclr
            .write(|w| unsafe { w.bits(1 << self.ch.number()) });
    }
}

impl<'d, C: Channel, const EVENT_COUNT: usize, const TASK_COUNT: usize> Drop
    for Ppi<'d, C, EVENT_COUNT, TASK_COUNT>
{
    fn drop(&mut self) {
        self.disable();
        self.disable_all();
    }
}

impl<'d, C: ZeroToOneChannel> Ppi<'d, C, 0, 1> {
    pub fn new_static_to_one(ch: impl Unborrow<Target = C> + 'd, task: Task) -> Self {
        unborrow!(ch);

        let events = [];
        let tasks = [task];

        Self::enable_all(&tasks, &events, &ch);

        Self {
            ch,
            #[cfg(feature = "_dppi")]
            events,
            #[cfg(feature = "_dppi")]
            tasks,
            phantom: PhantomData,
        }
    }
}

impl<'d, C: OneToOneChannel> Ppi<'d, C, 1, 1> {
    pub fn new_one_to_one(ch: impl Unborrow<Target = C> + 'd, event: Event, task: Task) -> Self {
        unborrow!(ch);

        let events = [event];
        let tasks = [task];

        Self::enable_all(&tasks, &events, &ch);

        Self {
            ch,
            #[cfg(feature = "_dppi")]
            events,
            #[cfg(feature = "_dppi")]
            tasks,
            phantom: PhantomData,
        }
    }
}

impl<'d, C: OneToTwoChannel> Ppi<'d, C, 1, 2> {
    pub fn new_one_to_two(
        ch: impl Unborrow<Target = C> + 'd,
        event: Event,
        task1: Task,
        task2: Task,
    ) -> Self {
        unborrow!(ch);

        let events = [event];
        let tasks = [task1, task2];

        Self::enable_all(&tasks, &events, &ch);

        Self {
            ch,
            #[cfg(feature = "_dppi")]
            events,
            #[cfg(feature = "_dppi")]
            tasks,
            phantom: PhantomData,
        }
    }
}

impl<'d, C: ManyToManyChannel, const EVENT_COUNT: usize, const TASK_COUNT: usize>
    Ppi<'d, C, EVENT_COUNT, TASK_COUNT>
{
    pub fn new_many_to_many(
        ch: impl Unborrow<Target = C> + 'd,
        events: [Event; EVENT_COUNT],
        tasks: [Task; TASK_COUNT],
    ) -> Self {
        unborrow!(ch);

        Self::enable_all(&tasks, &events, &ch);

        Self {
            ch,
            #[cfg(feature = "_dppi")]
            events,
            #[cfg(feature = "_dppi")]
            tasks,
            phantom: PhantomData,
        }
    }
}

const REGISTER_DPPI_CONFIG_OFFSET: usize = 0x80 / core::mem::size_of::<u32>();

/// Represents a task that a peripheral can do.
/// When a task is subscribed to a PPI channel it will run when the channel is triggered by
/// a published event.
///
/// The pointer is to a task register
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Task(pub NonNull<u32>);
impl Task {
    pub(crate) fn from_reg<T>(reg: &T) -> Self {
        Self(unsafe { NonNull::new_unchecked(reg as *const _ as *mut _) })
    }

    pub fn subscribe_reg(&self) -> *mut u32 {
        unsafe { self.0.as_ptr().add(REGISTER_DPPI_CONFIG_OFFSET) }
    }
}

/// # Safety
///
/// NonNull is not send, but this event is only allowed to point at registers and those exist in any context on the same core.
unsafe impl Send for Task {}

/// Represents an event that a peripheral can publish.
/// An event can be set to publish on a PPI channel when the event happens.
///
/// The pointer is to an event register
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Event(pub NonNull<u32>);
impl Event {
    pub(crate) fn from_reg<T>(reg: &T) -> Self {
        Self(unsafe { NonNull::new_unchecked(reg as *const _ as *mut _) })
    }

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

pub trait Channel: sealed::Channel + Unborrow<Target = Self> + Sized {
    /// Returns the number of the channel
    fn number(&self) -> usize;
    #[cfg(feature = "_ppi")]
    fn is_task_configurable(&self) -> bool;
}

pub trait ZeroToOneChannel: Channel {}
pub trait OneToOneChannel: ZeroToOneChannel {}
pub trait OneToTwoChannel: OneToOneChannel {}
pub trait ManyToManyChannel: OneToTwoChannel {}

pub trait Group: sealed::Group + Sized {
    fn number(&self) -> usize;
}

// ======================
//       channels

pub struct AnyChannel {
    number: u8,
    #[cfg(feature = "_ppi")]
    has_configurable_task: bool,
}
unsafe_impl_unborrow!(AnyChannel);
impl sealed::Channel for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> usize {
        self.number as usize
    }

    #[cfg(feature = "_ppi")]
    fn is_task_configurable(&self) -> bool {
        self.has_configurable_task
    }
}

macro_rules! impl_ppi_channel {
    ($type:ident, $number:expr, $has_configurable_task:expr) => {
        impl crate::ppi::sealed::Channel for peripherals::$type {}
        impl crate::ppi::Channel for peripherals::$type {
            fn number(&self) -> usize {
                $number
            }

            #[cfg(feature = "_ppi")]
            fn is_task_configurable(&self) -> bool {
                $has_configurable_task
            }
        }
    };
    ($type:ident, $number:expr, $has_configurable_task:expr, 0, 0) => {
        impl_ppi_channel!($type, $number, $has_configurable_task);
    };
    ($type:ident, $number:expr, $has_configurable_task:expr, 0, 1) => {
        impl_ppi_channel!($type, $number, $has_configurable_task, 0, 0);
        impl crate::ppi::ZeroToOneChannel for peripherals::$type {}
    };
    ($type:ident, $number:expr, $has_configurable_task:expr, 1, 1) => {
        impl_ppi_channel!($type, $number, $has_configurable_task, 0, 1);
        impl crate::ppi::OneToOneChannel for peripherals::$type {}
    };
    ($type:ident, $number:expr, $has_configurable_task:expr, 1, 2) => {
        impl_ppi_channel!($type, $number, $has_configurable_task, 1, 1);
        impl crate::ppi::OneToTwoChannel for peripherals::$type {}
    };
    ($type:ident, $number:expr, $has_configurable_task:expr, many, many) => {
        impl_ppi_channel!($type, $number, $has_configurable_task, 1, 2);
        impl crate::ppi::ManyToManyChannel for peripherals::$type {}
    };
}

// ======================
//       groups

pub struct AnyGroup {
    number: u8,
}
unsafe_impl_unborrow!(AnyGroup);
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
