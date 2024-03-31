//! Definition of Filter structs for FDCAN Module
// Note: This file is copied and modified from fdcan crate by Richard Meadows

use embedded_can::{ExtendedId, StandardId};

use crate::can::fd::message_ram;
pub use crate::can::fd::message_ram::{EXTENDED_FILTER_MAX, STANDARD_FILTER_MAX};

/// A Standard Filter
pub type StandardFilter = Filter<StandardId, u16>;
/// An Extended Filter
pub type ExtendedFilter = Filter<ExtendedId, u32>;

impl Default for StandardFilter {
    fn default() -> Self {
        StandardFilter::disable()
    }
}
impl Default for ExtendedFilter {
    fn default() -> Self {
        ExtendedFilter::disable()
    }
}

impl StandardFilter {
    /// Accept all messages in FIFO 0
    pub fn accept_all_into_fifo0() -> StandardFilter {
        StandardFilter {
            filter: FilterType::BitMask { filter: 0x0, mask: 0x0 },
            action: Action::StoreInFifo0,
        }
    }

    /// Accept all messages in FIFO 1
    pub fn accept_all_into_fifo1() -> StandardFilter {
        StandardFilter {
            filter: FilterType::BitMask { filter: 0x0, mask: 0x0 },
            action: Action::StoreInFifo1,
        }
    }

    /// Reject all messages
    pub fn reject_all() -> StandardFilter {
        StandardFilter {
            filter: FilterType::BitMask { filter: 0x0, mask: 0x0 },
            action: Action::Reject,
        }
    }

    /// Disable the filter
    pub fn disable() -> StandardFilter {
        StandardFilter {
            filter: FilterType::Disabled,
            action: Action::Disable,
        }
    }
}

impl ExtendedFilter {
    /// Accept all messages in FIFO 0
    pub fn accept_all_into_fifo0() -> ExtendedFilter {
        ExtendedFilter {
            filter: FilterType::BitMask { filter: 0x0, mask: 0x0 },
            action: Action::StoreInFifo0,
        }
    }

    /// Accept all messages in FIFO 1
    pub fn accept_all_into_fifo1() -> ExtendedFilter {
        ExtendedFilter {
            filter: FilterType::BitMask { filter: 0x0, mask: 0x0 },
            action: Action::StoreInFifo1,
        }
    }

    /// Reject all messages
    pub fn reject_all() -> ExtendedFilter {
        ExtendedFilter {
            filter: FilterType::BitMask { filter: 0x0, mask: 0x0 },
            action: Action::Reject,
        }
    }

    /// Disable the filter
    pub fn disable() -> ExtendedFilter {
        ExtendedFilter {
            filter: FilterType::Disabled,
            action: Action::Disable,
        }
    }
}

/// Filter Type
#[derive(Clone, Copy, Debug)]
pub enum FilterType<ID, UNIT>
where
    ID: Copy + Clone + core::fmt::Debug,
    UNIT: Copy + Clone + core::fmt::Debug,
{
    /// Match with a range between two messages
    Range {
        /// First Id of the range
        from: ID,
        /// Last Id of the range
        to: ID,
    },
    /// Match with a bitmask
    BitMask {
        /// Filter of the bitmask
        filter: UNIT,
        /// Mask of the bitmask
        mask: UNIT,
    },
    /// Match with a single ID
    DedicatedSingle(ID),
    /// Match with one of two ID's
    DedicatedDual(ID, ID),
    /// Filter is disabled
    Disabled,
}
impl<ID, UNIT> From<FilterType<ID, UNIT>> for message_ram::enums::FilterType
where
    ID: Copy + Clone + core::fmt::Debug,
    UNIT: Copy + Clone + core::fmt::Debug,
{
    fn from(f: FilterType<ID, UNIT>) -> Self {
        match f {
            FilterType::Range { to: _, from: _ } => Self::RangeFilter,
            FilterType::BitMask { filter: _, mask: _ } => Self::ClassicFilter,
            FilterType::DedicatedSingle(_) => Self::DualIdFilter,
            FilterType::DedicatedDual(_, _) => Self::DualIdFilter,
            FilterType::Disabled => Self::FilterDisabled,
        }
    }
}

/// Filter Action
#[derive(Clone, Copy, Debug)]
pub enum Action {
    /// No Action
    Disable = 0b000,
    /// Store an matching message in FIFO 0
    StoreInFifo0 = 0b001,
    /// Store an matching message in FIFO 1
    StoreInFifo1 = 0b010,
    /// Reject an matching message
    Reject = 0b011,
    /// Flag a matching message (But not store?!?)
    FlagHighPrio = 0b100,
    /// Flag a matching message as a High Priority message and store it in FIFO 0
    FlagHighPrioAndStoreInFifo0 = 0b101,
    /// Flag a matching message as a High Priority message and store it in FIFO 1
    FlagHighPrioAndStoreInFifo1 = 0b110,
}
impl From<Action> for message_ram::enums::FilterElementConfig {
    fn from(a: Action) -> Self {
        match a {
            Action::Disable => Self::DisableFilterElement,
            Action::StoreInFifo0 => Self::StoreInFifo0,
            Action::StoreInFifo1 => Self::StoreInFifo1,
            Action::Reject => Self::Reject,
            Action::FlagHighPrio => Self::SetPriority,
            Action::FlagHighPrioAndStoreInFifo0 => Self::SetPriorityAndStoreInFifo0,
            Action::FlagHighPrioAndStoreInFifo1 => Self::SetPriorityAndStoreInFifo1,
        }
    }
}

/// Filter
#[derive(Clone, Copy, Debug)]
pub struct Filter<ID, UNIT>
where
    ID: Copy + Clone + core::fmt::Debug,
    UNIT: Copy + Clone + core::fmt::Debug,
{
    /// How to match an incoming message
    pub filter: FilterType<ID, UNIT>,
    /// What to do with a matching message
    pub action: Action,
}

/// Standard Filter Slot
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StandardFilterSlot {
    /// 0
    _0 = 0,
    /// 1
    _1 = 1,
    /// 2
    _2 = 2,
    /// 3
    _3 = 3,
    /// 4
    _4 = 4,
    /// 5
    _5 = 5,
    /// 6
    _6 = 6,
    /// 7
    _7 = 7,
    /// 8
    _8 = 8,
    /// 9
    _9 = 9,
    /// 10
    _10 = 10,
    /// 11
    _11 = 11,
    /// 12
    _12 = 12,
    /// 13
    _13 = 13,
    /// 14
    _14 = 14,
    /// 15
    _15 = 15,
    /// 16
    _16 = 16,
    /// 17
    _17 = 17,
    /// 18
    _18 = 18,
    /// 19
    _19 = 19,
    /// 20
    _20 = 20,
    /// 21
    _21 = 21,
    /// 22
    _22 = 22,
    /// 23
    _23 = 23,
    /// 24
    _24 = 24,
    /// 25
    _25 = 25,
    /// 26
    _26 = 26,
    /// 27
    _27 = 27,
}
impl From<u8> for StandardFilterSlot {
    fn from(u: u8) -> Self {
        match u {
            0 => StandardFilterSlot::_0,
            1 => StandardFilterSlot::_1,
            2 => StandardFilterSlot::_2,
            3 => StandardFilterSlot::_3,
            4 => StandardFilterSlot::_4,
            5 => StandardFilterSlot::_5,
            6 => StandardFilterSlot::_6,
            7 => StandardFilterSlot::_7,
            8 => StandardFilterSlot::_8,
            9 => StandardFilterSlot::_9,
            10 => StandardFilterSlot::_10,
            11 => StandardFilterSlot::_11,
            12 => StandardFilterSlot::_12,
            13 => StandardFilterSlot::_13,
            14 => StandardFilterSlot::_14,
            15 => StandardFilterSlot::_15,
            16 => StandardFilterSlot::_16,
            17 => StandardFilterSlot::_17,
            18 => StandardFilterSlot::_18,
            19 => StandardFilterSlot::_19,
            20 => StandardFilterSlot::_20,
            21 => StandardFilterSlot::_21,
            22 => StandardFilterSlot::_22,
            23 => StandardFilterSlot::_23,
            24 => StandardFilterSlot::_24,
            25 => StandardFilterSlot::_25,
            26 => StandardFilterSlot::_26,
            27 => StandardFilterSlot::_27,
            _ => panic!("Standard Filter Slot Too High!"),
        }
    }
}

/// Extended Filter Slot
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExtendedFilterSlot {
    /// 0
    _0 = 0,
    /// 1
    _1 = 1,
    /// 2
    _2 = 2,
    /// 3
    _3 = 3,
    /// 4
    _4 = 4,
    /// 5
    _5 = 5,
    /// 6
    _6 = 6,
    /// 7
    _7 = 7,
}
impl From<u8> for ExtendedFilterSlot {
    fn from(u: u8) -> Self {
        match u {
            0 => ExtendedFilterSlot::_0,
            1 => ExtendedFilterSlot::_1,
            2 => ExtendedFilterSlot::_2,
            3 => ExtendedFilterSlot::_3,
            4 => ExtendedFilterSlot::_4,
            5 => ExtendedFilterSlot::_5,
            6 => ExtendedFilterSlot::_6,
            7 => ExtendedFilterSlot::_7,
            _ => panic!("Extended Filter Slot Too High!"), // Should be unreachable
        }
    }
}

/// Enum over both Standard and Extended Filter ID's
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FilterId {
    /// Standard Filter Slots
    Standard(StandardFilterSlot),
    /// Extended Filter Slots
    Extended(ExtendedFilterSlot),
}

pub(crate) trait ActivateFilter<ID, UNIT>
where
    ID: Copy + Clone + core::fmt::Debug,
    UNIT: Copy + Clone + core::fmt::Debug,
{
    fn activate(&mut self, f: Filter<ID, UNIT>);
    // fn read(&self) -> Filter<ID, UNIT>;
}

impl ActivateFilter<StandardId, u16> for message_ram::StandardFilter {
    fn activate(&mut self, f: Filter<StandardId, u16>) {
        let sft = f.filter.into();

        let (sfid1, sfid2) = match f.filter {
            FilterType::Range { to, from } => (to.as_raw(), from.as_raw()),
            FilterType::DedicatedSingle(id) => (id.as_raw(), id.as_raw()),
            FilterType::DedicatedDual(id1, id2) => (id1.as_raw(), id2.as_raw()),
            FilterType::BitMask { filter, mask } => (filter, mask),
            FilterType::Disabled => (0x0, 0x0),
        };
        let sfec = f.action.into();
        self.write(|w| {
            unsafe { w.sfid1().bits(sfid1).sfid2().bits(sfid2) }
                .sft()
                .set_filter_type(sft)
                .sfec()
                .set_filter_element_config(sfec)
        });
    }
    // fn read(&self) -> Filter<StandardId, u16> {
    //     todo!()
    // }
}
impl ActivateFilter<ExtendedId, u32> for message_ram::ExtendedFilter {
    fn activate(&mut self, f: Filter<ExtendedId, u32>) {
        let eft = f.filter.into();

        let (efid1, efid2) = match f.filter {
            FilterType::Range { to, from } => (to.as_raw(), from.as_raw()),
            FilterType::DedicatedSingle(id) => (id.as_raw(), id.as_raw()),
            FilterType::DedicatedDual(id1, id2) => (id1.as_raw(), id2.as_raw()),
            FilterType::BitMask { filter, mask } => (filter, mask),
            FilterType::Disabled => (0x0, 0x0),
        };
        let efec = f.action.into();
        self.write(|w| {
            unsafe { w.efid1().bits(efid1).efid2().bits(efid2) }
                .eft()
                .set_filter_type(eft)
                .efec()
                .set_filter_element_config(efec)
        });
    }
    // fn read(&self) -> Filter<ExtendedId, u32> {
    //     todo!()
    // }
}
