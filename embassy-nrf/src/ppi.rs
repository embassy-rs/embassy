#![macro_use]

//! HAL interface for the PPI peripheral.
//!
//! The Programmable Peripheral Interconnect interface allows for an autonomous interoperability
//! between peripherals through their events and tasks. There are fixed PPI channels and fully
//! configurable ones, fixed channels can only connect specific events to specific tasks. For fully
//! configurable channels, it is possible to choose, via software, the event and the task that it
//! will triggered by the event.
//!
//! On nRF52 devices, there is also a fork task endpoint, where the user can configure one more task
//! to be triggered by the same event, even fixed PPI channels have a configurable fork task.

use crate::{pac, peripherals};
use core::marker::PhantomData;
use core::ptr::NonNull;
use embassy::util::Unborrow;
use embassy_hal_common::{unborrow, unsafe_impl_unborrow};

// ======================
//       driver

pub struct Ppi<'d, C: Channel> {
    ch: C,
    phantom: PhantomData<&'d mut C>,
}

impl<'d, C: Channel> Ppi<'d, C> {
    pub fn new(ch: impl Unborrow<Target = C> + 'd) -> Self {
        unborrow!(ch);

        #[allow(unused_mut)]
        let mut this = Self {
            ch,
            phantom: PhantomData,
        };
        #[cfg(not(any(feature = "nrf51", feature = "nrf9160")))]
        this.clear_fork_task();
        this
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

    #[cfg(not(any(feature = "nrf51", feature = "nrf9160")))]
    /// Sets the fork task that must be triggered when the configured event occurs. The user must
    /// provide a reference to the task.
    pub fn set_fork_task(&mut self, task: Task) {
        let r = unsafe { &*pac::PPI::ptr() };
        r.fork[self.ch.number()]
            .tep
            .write(|w| unsafe { w.bits(task.0.as_ptr() as u32) })
    }

    #[cfg(not(any(feature = "nrf51", feature = "nrf9160")))]
    /// Clear the fork task endpoint. Previously set task will no longer be triggered.
    pub fn clear_fork_task(&mut self) {
        let r = unsafe { &*pac::PPI::ptr() };
        r.fork[self.ch.number()].tep.write(|w| unsafe { w.bits(0) })
    }

    #[cfg(feature = "nrf9160")]
    /// Sets the fork task that must be triggered when the configured event occurs. The user must
    /// provide a reference to the task.
    pub fn set_fork_task(&mut self, _task: Task) {
        todo!("Tasks not yet implemented for nrf9160");
    }

    #[cfg(feature = "nrf9160")]
    /// Clear the fork task endpoint. Previously set task will no longer be triggered.
    pub fn clear_fork_task(&mut self) {
        todo!("Tasks not yet implemented for nrf9160");
    }
}

impl<'d, C: Channel> Drop for Ppi<'d, C> {
    fn drop(&mut self) {
        self.disable()
    }
}

#[cfg(not(feature = "nrf9160"))]
impl<'d, C: ConfigurableChannel> Ppi<'d, C> {
    /// Sets the task to be triggered when the configured event occurs.
    pub fn set_task(&mut self, task: Task) {
        let r = unsafe { &*pac::PPI::ptr() };
        r.ch[self.ch.number()]
            .tep
            .write(|w| unsafe { w.bits(task.0.as_ptr() as u32) })
    }

    /// Sets the event that will trigger the chosen task(s).
    pub fn set_event(&mut self, event: Event) {
        let r = unsafe { &*pac::PPI::ptr() };
        r.ch[self.ch.number()]
            .eep
            .write(|w| unsafe { w.bits(event.0.as_ptr() as u32) })
    }
}

#[cfg(feature = "nrf9160")]
impl<'d, C: ConfigurableChannel> Ppi<'d, C> {
    /// Sets the task to be triggered when the configured event occurs.
    pub fn set_task(&mut self, _task: Task) {
        todo!("Tasks not yet implemented for nrf9160")
    }

    /// Sets the event that will trigger the chosen task(s).
    pub fn set_event(&mut self, _event: Event) {
        todo!("Events not yet implemented for nrf9160")
    }
}

// ======================
//       traits

pub struct Task(pub NonNull<()>);
impl Task {
    pub(crate) fn from_reg<T>(reg: &T) -> Self {
        Self(unsafe { NonNull::new_unchecked(reg as *const _ as *mut ()) })
    }
}

pub struct Event(pub NonNull<()>);
impl Event {
    pub(crate) fn from_reg<T>(reg: &T) -> Self {
        Self(unsafe { NonNull::new_unchecked(reg as *const _ as *mut ()) })
    }
}

pub(crate) mod sealed {
    pub trait ConfigurableChannel {}
    pub trait Channel {}
    pub trait Group {}
}

pub trait Channel: sealed::Channel + Sized {
    fn number(&self) -> usize;
    fn degrade(self) -> AnyChannel {
        AnyChannel {
            number: self.number() as u8,
        }
    }
}
pub trait ConfigurableChannel: Channel + sealed::ConfigurableChannel {
    fn degrade_configurable(self) -> AnyConfigurableChannel {
        AnyConfigurableChannel {
            number: self.number() as u8,
        }
    }
}

pub trait Group: sealed::Group + Sized {
    fn number(&self) -> usize;
    fn degrade(self) -> AnyGroup {
        AnyGroup {
            number: self.number() as u8,
        }
    }
}

// ======================
//       channels

pub struct AnyChannel {
    number: u8,
}
unsafe_impl_unborrow!(AnyChannel);
impl sealed::Channel for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> usize {
        self.number as usize
    }
}

pub struct AnyConfigurableChannel {
    number: u8,
}
unsafe_impl_unborrow!(AnyConfigurableChannel);
impl sealed::Channel for AnyConfigurableChannel {}
impl sealed::ConfigurableChannel for AnyConfigurableChannel {}
impl ConfigurableChannel for AnyConfigurableChannel {}
impl Channel for AnyConfigurableChannel {
    fn number(&self) -> usize {
        self.number as usize
    }
}

macro_rules! impl_ppi_channel {
    ($type:ident, $number:expr, configurable) => {
        impl_ppi_channel!($type, $number);
        impl crate::ppi::sealed::ConfigurableChannel for peripherals::$type {}
        impl crate::ppi::ConfigurableChannel for peripherals::$type {}
    };
    ($type:ident, $number:expr) => {
        impl crate::ppi::sealed::Channel for peripherals::$type {}
        impl crate::ppi::Channel for peripherals::$type {
            fn number(&self) -> usize {
                $number
            }
        }
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
