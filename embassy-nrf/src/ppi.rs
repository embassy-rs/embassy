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

/// Error type of the PPI driver
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Error {
    /// There is no capacity to enable this task or event (nRF51 & nRF52 only)
    NoCapacityLeft,
    /// This task or event is not in use by the current channel
    NotInUseByChannel,
    /// This task or event is already enabled on another channel (nRF53 & nRF91 only)
    AlreadyInUse,
}

pub struct Ppi<'d, C: Channel> {
    ch: C,
    phantom: PhantomData<&'d mut C>,
}

impl<'d, C: Channel> Ppi<'d, C> {
    pub fn new(ch: impl Unborrow<Target = C> + 'd) -> Self {
        unborrow!(ch);

        Self {
            ch,
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

#[cfg(feature = "_ppi")]
impl<'d, C: Channel> Ppi<'d, C> {
    /// Makes it so that the given task is subscribed to this channel
    pub fn subscribe(&mut self, task: Task) -> Result<(), Error> {
        if self.is_main_task_free() {
            self.set_main_task(Some(task));
            Ok(())
        } else if self.is_fork_task_free() {
            self.set_fork_task(Some(task));
            Ok(())
        } else {
            Err(Error::NoCapacityLeft)
        }
    }

    /// Makes it so that the given task is not subscribed to this channel
    pub fn unsubscribe(&mut self, task: Task) -> Result<(), Error> {
        if self.get_main_task() == Some(task) {
            // If there is a fork task, we move that to the main task for consistency
            // If there is no fork task, then the main task is set to 0
            let fork_task = self.get_fork_task();
            self.set_main_task(fork_task);

            if self.has_fork_task() {
                // The fork task was copied to the main task, so reset the fork task
                self.set_fork_task(None);
            }
            Ok(())
        } else if self.get_fork_task() == Some(task) {
            // Reset the fork task
            self.set_fork_task(None);
            Ok(())
        } else {
            Err(Error::NotInUseByChannel)
        }
    }

    /// Makes it so that the given event is published on this channel
    pub fn publish(&mut self, event: Event) -> Result<(), Error> {
        if self.is_event_free() {
            self.set_event(Some(event));
            Ok(())
        } else {
            Err(Error::NoCapacityLeft)
        }
    }

    /// Makes it so that the given event is not published on this channel
    pub fn unpublish(&mut self, event: Event) -> Result<(), Error> {
        if self.get_event() == Some(event) {
            self.set_event(None);
            Ok(())
        } else {
            Err(Error::NotInUseByChannel)
        }
    }

    fn set_main_task(&mut self, task: Option<Task>) {
        let r = unsafe { &*pac::PPI::ptr() };
        if let Some(task) = task {
            r.ch[self.ch.number()]
                .tep
                .write(|w| unsafe { w.bits(task.0.as_ptr() as u32) })
        } else {
            r.ch[self.ch.number()].tep.write(|w| unsafe { w.bits(0) })
        }
    }

    fn get_main_task(&mut self) -> Option<Task> {
        let r = unsafe { &*pac::PPI::ptr() };

        if !self.has_main_task() {
            return None;
        }

        let bits = r.ch[self.ch.number()].tep.read().tep().bits();

        if bits == 0 {
            None
        } else {
            unsafe { Some(Task(NonNull::new_unchecked(bits as *mut _))) }
        }
    }

    fn set_fork_task(&mut self, task: Option<Task>) {
        let r = unsafe { &*pac::PPI::ptr() };
        if let Some(task) = task {
            r.fork[self.ch.number()]
                .tep
                .write(|w| unsafe { w.bits(task.0.as_ptr() as u32) })
        } else {
            r.fork[self.ch.number()].tep.write(|w| unsafe { w.bits(0) })
        }
    }

    fn get_fork_task(&mut self) -> Option<Task> {
        let r = unsafe { &*pac::PPI::ptr() };

        if !self.has_fork_task() {
            return None;
        }

        let bits = r.fork[self.ch.number()].tep.read().tep().bits();

        if bits == 0 {
            None
        } else {
            unsafe { Some(Task(NonNull::new_unchecked(bits as *mut _))) }
        }
    }

    fn has_main_task(&self) -> bool {
        match (self.ch.task_capacity(), self.ch.event_capacity()) {
            (0, 0) => false,     // Static task
            (1, 0) => false,     // Static task with fork
            (1 | 2, 1) => true,  // Configurable task with possibly a fork
            _ => unreachable!(), // Every PPI config is covered
        }
    }

    fn has_fork_task(&self) -> bool {
        match (self.ch.task_capacity(), self.ch.event_capacity()) {
            (0, 0) => false,     // Static task
            (1, 0) => true,      // Static task with fork
            (1, 1) => false,     // Configurable task without fork
            (2, 1) => true,      // Configurable task with fork
            _ => unreachable!(), // Every PPI config is covered
        }
    }

    fn is_main_task_free(&mut self) -> bool {
        self.get_main_task().is_none()
    }

    fn is_fork_task_free(&mut self) -> bool {
        self.get_fork_task().is_none()
    }

    fn set_event(&mut self, event: Option<Event>) {
        let r = unsafe { &*pac::PPI::ptr() };
        if let Some(event) = event {
            r.ch[self.ch.number()]
                .eep
                .write(|w| unsafe { w.bits(event.0.as_ptr() as u32) })
        } else {
            r.ch[self.ch.number()].eep.write(|w| unsafe { w.bits(0) })
        }
    }

    fn get_event(&mut self) -> Option<Event> {
        let r = unsafe { &*pac::PPI::ptr() };

        if !self.has_event() {
            return None;
        }

        let bits = r.ch[self.ch.number()].eep.read().eep().bits();

        if bits == 0 {
            None
        } else {
            unsafe { Some(Event(NonNull::new_unchecked(bits as *mut _))) }
        }
    }

    fn has_event(&self) -> bool {
        match (self.ch.task_capacity(), self.ch.event_capacity()) {
            (_, 0) => false,     // Static event
            (_, 1) => true,      // Configurable event
            _ => unreachable!(), // Every PPI config is covered
        }
    }

    fn is_event_free(&mut self) -> bool {
        self.get_event().is_none()
    }
}

#[cfg(feature = "_dppi")]
const DPPI_ENABLE_BIT: u32 = 0x8000_0000;
#[cfg(feature = "_dppi")]
const DPPI_CHANNEL_MASK: u32 = 0x0000_00FF;

#[cfg(feature = "_dppi")]
impl<'d, C: Channel> Ppi<'d, C> {
    /// Makes it so that the given task is subscribed to this channel
    pub fn subscribe(&mut self, task: Task) -> Result<(), Error> {
        unsafe {
            if Self::is_register_enabled(task.0) {
                Err(Error::AlreadyInUse)
            } else {
                Self::set_register_active(task.0, self.ch.number() as u8);
                Ok(())
            }
        }
    }

    /// Makes it so that the given task is not subscribed to this channel
    pub fn unsubscribe(&mut self, task: Task) -> Result<(), Error> {
        unsafe {
            if Self::get_register_channel(task.0) != self.ch.number() as u8 {
                Err(Error::NotInUseByChannel)
            } else {
                Self::set_register_inactive(task.0);
                Ok(())
            }
        }
    }

    /// Makes it so that the given event is published on this channel
    pub fn publish(&mut self, event: Event) -> Result<(), Error> {
        unsafe {
            if Self::is_register_enabled(event.0) {
                Err(Error::AlreadyInUse)
            } else {
                Self::set_register_active(event.0, self.ch.number() as u8);
                Ok(())
            }
        }
    }

    /// Makes it so that the given event is not published on this channel
    pub fn unpublish(&mut self, event: Event) -> Result<(), Error> {
        unsafe {
            if Self::get_register_channel(event.0) != self.ch.number() as u8 {
                Err(Error::NotInUseByChannel)
            } else {
                Self::set_register_inactive(event.0);
                Ok(())
            }
        }
    }

    /// Checks if the DPPI_ENABLE_BIT is set in the register
    ///
    /// # Safety
    ///
    /// The register pointer must point at one of the many SUBSCRIBE_* or PUBLISH_* registers of the peripherals
    unsafe fn is_register_enabled(register: NonNull<u32>) -> bool {
        let bits = register.as_ptr().read_volatile();
        bits & DPPI_ENABLE_BIT > 0
    }

    /// Sets the register to the given channel and enables it
    ///
    /// # Safety
    ///
    /// The register pointer must point at one of the many SUBSCRIBE_* or PUBLISH_* registers of the peripherals
    unsafe fn set_register_active(register: NonNull<u32>, channel: u8) {
        register
            .as_ptr()
            .write_volatile(DPPI_ENABLE_BIT | (channel as u32 & DPPI_CHANNEL_MASK));
    }

    /// Resets the channel number and disables the register
    ///
    /// # Safety
    ///
    /// The register pointer must point at one of the many SUBSCRIBE_* or PUBLISH_* registers of the peripherals
    unsafe fn set_register_inactive(register: NonNull<u32>) {
        register.as_ptr().write_volatile(0);
    }

    /// Gets the current configured channel number of the register
    ///
    /// # Safety
    ///
    /// The register pointer must point at one of the many SUBSCRIBE_* or PUBLISH_* registers of the peripherals
    unsafe fn get_register_channel(register: NonNull<u32>) -> u8 {
        let bits = register.as_ptr().read_volatile();
        (bits & DPPI_CHANNEL_MASK) as u8
    }
}

impl<'d, C: Channel> Drop for Ppi<'d, C> {
    fn drop(&mut self) {
        self.disable()
    }
}

/// Represents a task that a peripheral can do.
/// When a task is subscribed to a PPI channel it will run when the channel is triggered by
/// a published event.
///
/// The pointer in the task can point to two different kinds of register:
/// - PPI *(nRF51 & nRF52)*: A pointer to a task register of the task of the peripheral that has
/// to be registered with the PPI to subscribe to a channel
/// - DPPI *(nRF53 & nRF91)*: A pointer to the subscribe register of the task of the peripheral
/// that has to have the channel number and enable bit written tp it to subscribe to a channel
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Task(pub NonNull<u32>);
impl Task {
    pub(crate) fn from_reg<T>(reg: &T) -> Self {
        Self(unsafe { NonNull::new_unchecked(reg as *const _ as *mut _) })
    }
}

/// Represents an event that a peripheral can publish.
/// An event can be set to publish on a PPI channel when the event happens.
///
/// The pointer in the event can point to two different kinds of register:
/// - PPI *(nRF51 & nRF52)*: A pointer to an event register of the event of the peripheral that has
/// to be registered with the PPI to publish to a channel
/// - DPPI *(nRF53 & nRF91)*: A pointer to the publish register of the event of the peripheral
/// that has to have the channel number and enable bit written tp it to publish to a channel
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Event(pub NonNull<u32>);
impl Event {
    pub(crate) fn from_reg<T>(reg: &T) -> Self {
        Self(unsafe { NonNull::new_unchecked(reg as *const _ as *mut _) })
    }
}

// ======================
//       traits

pub(crate) mod sealed {
    pub trait Channel {}
    pub trait Group {}
}

pub trait Channel: sealed::Channel + Sized {
    /// Returns the number of the channel
    fn number(&self) -> usize;

    /// Returns the amount of configurable tasks this channel has.
    ///
    /// - MAX for DPPI with unlimited capacity (nRF53 & nRF91)
    /// - 0 for static channel without fork (nRF51)
    /// - 1 for static channel with fork (nRF52) or for configurable channel (nRF51)
    /// - 2 for configurable channel with fork (nRF52)
    fn task_capacity(&self) -> usize;

    /// Returns the amount of configurable events this channels has
    ///
    /// - MAX for DPPI with unlimited capacity (nRF53 & nRF91)
    /// - 0 for static channel (nRF51 & nRF52)
    /// - 1 for configurable channel (nRF51 & nRF52)
    fn event_capacity(&self) -> usize;

    fn degrade(self) -> AnyChannel {
        pub trait ConfigurableChannel {}

        AnyChannel {
            number: self.number() as u8,
            task_capacity: self.task_capacity() as _,
            event_capacity: self.event_capacity() as _,
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
    task_capacity: u8,
    event_capacity: u8,
}
unsafe_impl_unborrow!(AnyChannel);
impl sealed::Channel for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> usize {
        self.number as usize
    }

    fn task_capacity(&self) -> usize {
        self.task_capacity as _
    }

    fn event_capacity(&self) -> usize {
        self.event_capacity as _
    }
}

macro_rules! impl_ppi_channel {
    ($type:ident, $number:expr, $task_capacity:expr, $event_capacity:expr) => {
        impl crate::ppi::sealed::Channel for peripherals::$type {}
        impl crate::ppi::Channel for peripherals::$type {
            fn number(&self) -> usize {
                $number
            }
            fn task_capacity(&self) -> usize {
                $task_capacity
            }
            fn event_capacity(&self) -> usize {
                $event_capacity
            }
        }
    };
    ($type:ident, $number:expr) => {
        impl crate::ppi::sealed::Channel for peripherals::$type {}
        impl crate::ppi::Channel for peripherals::$type {
            fn number(&self) -> usize {
                $number
            }
            fn task_capacity(&self) -> usize {
                usize::MAX
            }
            fn event_capacity(&self) -> usize {
                usize::MAX
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
