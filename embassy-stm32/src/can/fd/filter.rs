//! Definition of Filter structs for FDCAN Module
// Note: This file is copied and modified from fdcan crate by Richard Meadows

use embedded_can::{ExtendedId, StandardId};

use super::low_level::message_ram;

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
        StandardFilter::Queue {
            filter: FilterType::BitMask { filter: 0x0, mask: 0x0 },
            action: Action::StoreInFifo0,
        }
    }

    /// Accept all messages in FIFO 1
    pub fn accept_all_into_fifo1() -> StandardFilter {
        StandardFilter::Queue {
            filter: FilterType::BitMask { filter: 0x0, mask: 0x0 },
            action: Action::StoreInFifo1,
        }
    }

    /// Reject all messages
    pub fn reject_all() -> StandardFilter {
        StandardFilter::Queue {
            filter: FilterType::BitMask { filter: 0x0, mask: 0x0 },
            action: Action::Reject,
        }
    }

    /// Disable the filter
    pub fn disable() -> StandardFilter {
        StandardFilter::Queue {
            filter: FilterType::Disabled,
            action: Action::Disable,
        }
    }
}

impl ExtendedFilter {
    /// Accept all messages in FIFO 0
    pub fn accept_all_into_fifo0() -> ExtendedFilter {
        ExtendedFilter::Queue {
            filter: FilterType::BitMask { filter: 0x0, mask: 0x0 },
            action: Action::StoreInFifo0,
        }
    }

    /// Accept all messages in FIFO 1
    pub fn accept_all_into_fifo1() -> ExtendedFilter {
        ExtendedFilter::Queue {
            filter: FilterType::BitMask { filter: 0x0, mask: 0x0 },
            action: Action::StoreInFifo1,
        }
    }

    /// Reject all messages
    pub fn reject_all() -> ExtendedFilter {
        ExtendedFilter::Queue {
            filter: FilterType::BitMask { filter: 0x0, mask: 0x0 },
            action: Action::Reject,
        }
    }

    /// Disable the filter
    pub fn disable() -> ExtendedFilter {
        ExtendedFilter::Queue {
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
impl<ID, UNIT> From<FilterType<ID, UNIT>> for message_ram::FilterType
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
impl From<Action> for message_ram::FilterElementConfig {
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

/// The destination for a buffer type filter.
#[derive(Clone, Copy, Debug)]
pub enum BufferDestination {
    /// Message into the given dedicated Rx buffer index
    DedicatedBuffer(u8),
    // /// Debug message A
    // DebugA,
    // /// Debug message B
    // DebugB,
    // /// Debug message C
    // DebugC,
}

/// Filter
#[derive(Clone, Copy, Debug)]
pub enum Filter<ID, UNIT>
where
    ID: Copy + Clone + core::fmt::Debug,
    UNIT: Copy + Clone + core::fmt::Debug,
{
    /// Filter which directs the message to a FIFO based on
    /// filter logic.
    Queue {
        /// How to match an incoming message
        filter: FilterType<ID, UNIT>,
        /// What to do with a matching message
        action: Action,
    },
    /// Filter which directs a specific message ID to a dedicated
    /// Rx buffer.
    DedicatedBuffer {
        /// The CAN ID of the message to match
        id: ID,
        /// The destination buffer index
        destination: BufferDestination,
        // Dont support TTCAN for now, we don't support event pin stuff
    },
}

/// Enum over both Standard and Extended Filter ID's
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FilterId {
    /// Standard Filter Slots
    Standard(u8),
    /// Extended Filter Slots
    Extended(u8),
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
        match f {
            Filter::Queue { filter, action } => {
                let sft = filter.into();
                let (sfid1, sfid2) = match filter {
                    FilterType::Range { to, from } => (to.as_raw(), from.as_raw()),
                    FilterType::DedicatedSingle(id) => (id.as_raw(), id.as_raw()),
                    FilterType::DedicatedDual(id1, id2) => (id1.as_raw(), id2.as_raw()),
                    FilterType::BitMask { filter, mask } => (filter, mask),
                    FilterType::Disabled => (0x0, 0x0),
                };
                let sfec = action.into();
                self.write(|w| {
                    unsafe { w.sfid1().bits(sfid1).sfid2().bits(sfid2) }
                        .sft()
                        .set_filter_type(sft)
                        .sfec()
                        .set_filter_element_config(sfec)
                });
            }
            Filter::DedicatedBuffer { id, destination } => {
                self.write(|w| {
                    w.sfec()
                        .set_filter_element_config(message_ram::FilterElementConfig::StoreInRxBuffer);
                    // Doesn't matter
                    w.sft().set_filter_type(message_ram::FilterType::RangeFilter);

                    unsafe { w.sfid1().bits(id.as_raw()) };

                    let (buf_idx, store_flag) = match destination {
                        BufferDestination::DedicatedBuffer(buf_idx) => (buf_idx, 0),
                        // BufferDestination::DebugA => (0, 1),
                        // BufferDestination::DebugB => (0, 2),
                        // BufferDestination::DebugC => (0, 3),
                    };
                    let sfid2_bits = (buf_idx as u16 & 0b111111) | (store_flag << 9);
                    unsafe { w.sfid2().bits(sfid2_bits) };
                    w
                });
            }
        }
    }
}
impl ActivateFilter<ExtendedId, u32> for message_ram::ExtendedFilter {
    fn activate(&mut self, f: Filter<ExtendedId, u32>) {
        match f {
            Filter::Queue { filter, action } => {
                let eft = filter.into();

                let (efid1, efid2) = match filter {
                    FilterType::Range { to, from } => (to.as_raw(), from.as_raw()),
                    FilterType::DedicatedSingle(id) => (id.as_raw(), id.as_raw()),
                    FilterType::DedicatedDual(id1, id2) => (id1.as_raw(), id2.as_raw()),
                    FilterType::BitMask { filter, mask } => (filter, mask),
                    FilterType::Disabled => (0x0, 0x0),
                };
                let efec = action.into();
                self.write(|w| {
                    unsafe { w.efid1().bits(efid1).efid2().bits(efid2) }
                        .eft()
                        .set_filter_type(eft)
                        .efec()
                        .set_filter_element_config(efec)
                });
            }
            Filter::DedicatedBuffer { id, destination } => {
                self.write(|w| {
                    w.efec()
                        .set_filter_element_config(message_ram::FilterElementConfig::StoreInRxBuffer);
                    // Doesn't matter
                    w.eft().set_filter_type(message_ram::FilterType::RangeFilter);

                    unsafe { w.efid1().bits(id.as_raw()) };

                    let (buf_idx, store_flag) = match destination {
                        BufferDestination::DedicatedBuffer(buf_idx) => (buf_idx, 0),
                        // BufferDestination::DebugA => (0, 1),
                        // BufferDestination::DebugB => (0, 2),
                        // BufferDestination::DebugC => (0, 3),
                    };
                    let efid2_bits = (buf_idx as u32 & 0b111111) | (store_flag << 9);
                    unsafe { w.efid2().bits(efid2_bits) };
                    w
                });
            }
        }
    }
}
