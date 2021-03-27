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

use embassy_extras::impl_unborrow;

use crate::peripherals;

// ======================
//       traits

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
pub trait ConfigurableChannel: Channel + sealed::ConfigurableChannel {}

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
impl_unborrow!(AnyChannel);
impl sealed::Channel for AnyChannel {}
impl Channel for AnyChannel {
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
#[cfg(not(feature = "51"))]
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

// ======================
//       groups

pub struct AnyGroup {
    number: u8,
}
impl_unborrow!(AnyGroup);
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
