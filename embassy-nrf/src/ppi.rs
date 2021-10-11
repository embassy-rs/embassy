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

use core::marker::PhantomData;
use core::ptr::NonNull;
use embassy::util::Unborrow;
use embassy_hal_common::{unborrow, unsafe_impl_unborrow};

use crate::{pac, peripherals};

#[cfg(not(feature = "nrf9160"))]
pub(crate) use pac::PPI;
#[cfg(feature = "nrf9160")]
pub(crate) use pac::DPPIC_NS as PPI;

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
        #[cfg(not(any(feature = "51", feature = "nrf9160")))]
        this.clear_fork_task();
        this
    }

    /// Enables the channel.
    pub fn enable(&mut self) {
        let r = unsafe { &*PPI::ptr() };
        r.chenset
            .write(|w| unsafe { w.bits(1 << self.ch.number()) });
    }

    /// Disables the channel.
    pub fn disable(&mut self) {
        let r = unsafe { &*PPI::ptr() };
        r.chenclr
            .write(|w| unsafe { w.bits(1 << self.ch.number()) });
    }

    #[cfg(not(any(feature = "51", feature = "nrf9160")))]
    /// Sets the fork task that must be triggered when the configured event occurs. The user must
    /// provide a reference to the task.
    pub fn set_fork_task(&mut self, task: Task) {
        let r = unsafe { &*PPI::ptr() };
        r.fork[self.ch.number()]
            .tep
            .write(|w| unsafe { w.bits(task.0.as_ptr() as u32) })
    }

    #[cfg(not(any(feature = "51", feature = "nrf9160")))]
    /// Clear the fork task endpoint. Previously set task will no longer be triggered.
    pub fn clear_fork_task(&mut self) {
        let r = unsafe { &*PPI::ptr() };
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
        let r = unsafe { &*PPI::ptr() };
        r.ch[self.ch.number()]
            .tep
            .write(|w| unsafe { w.bits(task.0.as_ptr() as u32) })
    }

    /// Sets the event that will trigger the chosen task(s).
    pub fn set_event(&mut self, event: Event) {
        let r = unsafe { &*PPI::ptr() };
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

mod sealed {
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

macro_rules! impl_channel {
    ($type:ident, $number:expr, configurable) => {
        impl_channel!($type, $number);
        impl sealed::ConfigurableChannel for peripherals::$type {}
        impl ConfigurableChannel for peripherals::$type {}
    };
    ($type:ident, $number:expr) => {
        impl sealed::Channel for peripherals::$type {}
        impl Channel for peripherals::$type {
            fn number(&self) -> usize {
                $number
            }
        }
    };
}

pub use channel_impl::*;
#[cfg(not(feature = "nrf9160"))]
mod channel_impl {
    use super::*;

    impl_channel!(PPI_CH0, 0, configurable);
    impl_channel!(PPI_CH1, 1, configurable);
    impl_channel!(PPI_CH2, 2, configurable);
    impl_channel!(PPI_CH3, 3, configurable);
    impl_channel!(PPI_CH4, 4, configurable);
    impl_channel!(PPI_CH5, 5, configurable);
    impl_channel!(PPI_CH6, 6, configurable);
    impl_channel!(PPI_CH7, 7, configurable);
    impl_channel!(PPI_CH8, 8, configurable);
    impl_channel!(PPI_CH9, 9, configurable);
    impl_channel!(PPI_CH10, 10, configurable);
    impl_channel!(PPI_CH11, 11, configurable);
    impl_channel!(PPI_CH12, 12, configurable);
    impl_channel!(PPI_CH13, 13, configurable);
    impl_channel!(PPI_CH14, 14, configurable);
    impl_channel!(PPI_CH15, 15, configurable);
    #[cfg(not(feature = "51",))]
    impl_channel!(PPI_CH16, 16, configurable);
    #[cfg(not(feature = "51"))]
    impl_channel!(PPI_CH17, 17, configurable);
    #[cfg(not(feature = "51"))]
    impl_channel!(PPI_CH18, 18, configurable);
    #[cfg(not(feature = "51"))]
    impl_channel!(PPI_CH19, 19, configurable);
    impl_channel!(PPI_CH20, 20);
    impl_channel!(PPI_CH21, 21);
    impl_channel!(PPI_CH22, 22);
    impl_channel!(PPI_CH23, 23);
    impl_channel!(PPI_CH24, 24);
    impl_channel!(PPI_CH25, 25);
    impl_channel!(PPI_CH26, 26);
    impl_channel!(PPI_CH27, 27);
    impl_channel!(PPI_CH28, 28);
    impl_channel!(PPI_CH29, 29);
    impl_channel!(PPI_CH30, 30);
    impl_channel!(PPI_CH31, 31);
}
#[cfg(feature = "nrf9160")] // TODO: Implement configurability for nrf9160 and then remove these channel_impl modules
mod channel_impl {
    use super::*;

    impl_channel!(PPI_CH0, 0, configurable);
    impl_channel!(PPI_CH1, 1, configurable);
    impl_channel!(PPI_CH2, 2, configurable);
    impl_channel!(PPI_CH3, 3, configurable);
    impl_channel!(PPI_CH4, 4, configurable);
    impl_channel!(PPI_CH5, 5, configurable);
    impl_channel!(PPI_CH6, 6, configurable);
    impl_channel!(PPI_CH7, 7, configurable);
    impl_channel!(PPI_CH8, 8, configurable);
    impl_channel!(PPI_CH9, 9, configurable);
    impl_channel!(PPI_CH10, 10, configurable);
    impl_channel!(PPI_CH11, 11, configurable);
    impl_channel!(PPI_CH12, 12, configurable);
    impl_channel!(PPI_CH13, 13, configurable);
    impl_channel!(PPI_CH14, 14, configurable);
    impl_channel!(PPI_CH15, 15, configurable);
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
#[cfg(not(feature = "51"))]
impl_group!(PPI_GROUP4, 4);
#[cfg(not(feature = "51"))]
impl_group!(PPI_GROUP5, 5);
