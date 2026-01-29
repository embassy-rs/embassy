//! Structures and accesors for MCAN configuration and message RAM areas.
//! This module provides structure definitions for each type of element in MCAN message RAM
//! and utility functions to access them.
//!
//! Similar to a PAC - this module exposes a somewhat unsound API - there are methods which take immutable references
//! yet do mutate data - this unsafe interior mutability is identical to what chiptools PACs do, but means use of this module
//! requires significant extra care to make sure the methods can't be accessed from multiple places at once.

use bitfield::bitfield;

/// Maximum data allowed in RX or TX frames.
/// This is currently fixed at 8, as the driver does not support CAN-FD right now anyways.
pub const MAX_DATA_LEN: usize = 8;

/// How many elements will be in the RX FIFO?
/// (Note: This simple driver supports only a single RX FIFO at this point - other FIFOs will have zero size.)
const NUM_RX_ELEMENTS: usize = 10;
const NUM_TX_ELEMENTS: usize = 5;
const NUM_TX_EVENTS: usize = 0; // currently unused as embedded-can traits do not require confirmation yet.

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub(super) struct MsgHeader(u32); // note: Used for both RX and TX messages.
    u32;

    /// Error State Indicator
    pub esi, set_esi: 31;
    /// Extended Identifier
    pub xtd, set_xtd: 30;
    /// Remote Transmission Request
    pub rtr, set_rtr: 29;
    /// Identifier
    pub id, set_id: 28, 0;
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub(super) struct RxHeader(u32);

    /// Accepted Non-matching Frame
    pub anmf, set_anmf: 31;

    /// Filter index
    pub u8, fidx, set_fidx: 30, 24;

    /// FD Format
    pub fdf, set_fdf: 21;

    /// Bit Rate Switch
    pub brs, set_brs: 20;

    /// Data length code.
    pub u8, dlc, set_dlc: 19, 16;

    /// RX Timestamp
    pub u16, rxts, set_rxts: 15, 0;
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct TxHeader(u32);

    /// Message Marker
    pub u8, mm, set_mm: 31, 24;

    /// Event FIFO Control
    pub efc, set_efc: 23;

    /// FD Format
    pub fdf, set_fdf: 21;

    /// Bit Rate Switch
    pub brs, set_brs: 20;

    /// Data length code.
    pub u8, dlc, set_dlc: 19, 16;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(dead_code)]
pub(super) enum EventType {
    Unknown = 0x00,
    TxEvent = 0x01,
    TransmissionNotCancelled = 0x02,
}

impl From<u8> for EventType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::TxEvent,
            0x02 => Self::TransmissionNotCancelled,
            _ => Self::Unknown,
        }
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub(super) struct EventHeader(u32);

    /// Message Marker
    pub u8, mm, _: 31, 24;

    /// Event Type
    pub u8, into EventType, et, _: 23, 22;

    /// FD Format
    pub fdf, set_fdf: 21;

    /// Bit Rate Switch
    pub brs, _: 20;

    /// Data length code.
    pub u8, dlc, _: 19, 16;
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub(super) enum FilterType {
    RangeFilter = 0x00,
    DualIdFilter = 0x01,
    ClassicFilter = 0x02,
    Disable = 0x03,
    Unknown = 0x05,
}

impl From<u8> for FilterType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::RangeFilter,
            0x01 => Self::DualIdFilter,
            0x02 => Self::ClassicFilter,
            0x03 => Self::Disable,
            _ => Self::Unknown,
        }
    }
}

impl From<FilterType> for u8 {
    fn from(value: FilterType) -> Self {
        match value {
            FilterType::RangeFilter => 0x00,
            FilterType::DualIdFilter => 0x01,
            FilterType::ClassicFilter => 0x02,
            FilterType::Disable => 0x03,
            FilterType::Unknown => 0x03, // disable filter if type is unknown.
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub(super) enum FilterConfiguration {
    Disable = 0x00,
    StoreRxFifo0 = 0x01,
    StoreRxFifo1 = 0x02,
    Reject = 0x03,
    SetPriority = 0x04,
    SetPriorityAndStoreInFifo0 = 0x05,
    SetPriorityAndStoreInFifo1 = 0x06,
    StoreInRxBuffer = 0x07,
    Unknown = 0x08,
}

impl From<u8> for FilterConfiguration {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Disable,
            0x01 => Self::StoreRxFifo0,
            0x02 => Self::StoreRxFifo1,
            0x03 => Self::Reject,
            0x04 => Self::SetPriority,
            0x05 => Self::SetPriorityAndStoreInFifo0,
            0x06 => Self::SetPriorityAndStoreInFifo1,
            0x07 => Self::StoreInRxBuffer,
            _ => Self::Unknown,
        }
    }
}

impl From<FilterConfiguration> for u8 {
    fn from(value: FilterConfiguration) -> Self {
        match value {
            FilterConfiguration::Disable => 0x00,
            FilterConfiguration::StoreRxFifo0 => 0x01,
            FilterConfiguration::StoreRxFifo1 => 0x02,
            FilterConfiguration::Reject => 0x03,
            FilterConfiguration::SetPriority => 0x04,
            FilterConfiguration::SetPriorityAndStoreInFifo0 => 0x05,
            FilterConfiguration::SetPriorityAndStoreInFifo1 => 0x06,
            FilterConfiguration::StoreInRxBuffer => 0x07,
            FilterConfiguration::Unknown => 0x00, // will be truncated into disable for storage.
        }
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub(super) struct StandardFilter(u32);

    /// Filter type
    pub u8, from into FilterType, sft, set_sft: 31, 30;
    /// Filter configuration
    pub u8, from into FilterConfiguration, sfec, set_sfec: 29, 27;
    /// Filter ID 1
    pub u16, sfid1, set_sfid1: 26, 16;
    /// Filter ID 2
    pub u16, sfid2, set_sfid2: 10, 0;
}

impl Default for StandardFilter {
    fn default() -> Self {
        let mut s = Self(0);
        s.set_sft(FilterType::Disable);
        s.set_sfec(FilterConfiguration::Disable);

        s
    }
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub(super) struct ExtendedFilter1(u32);
    /// Filter configuration
    pub u8, from into FilterConfiguration, efec, set_efec: 31, 29;
    /// Filter ID 1.
    pub u32, efid1, set_efid1: 28, 0;
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub(super) struct ExtendedFilter2(u32);

    /// Filter type
    pub u8, from into FilterType, eft, set_eft: 31, 30;

    // Filter ID 2
    pub u32, sfid2, set_sfid2: 28, 0;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub(super) struct ExtendedFilter {
    // a bit awkward to split this into two structs, but this is hidden from downstream consumers anyways.
    pub ef1: ExtendedFilter1,
    pub ef2: ExtendedFilter2,
}

impl Default for ExtendedFilter {
    fn default() -> Self {
        let mut s = ExtendedFilter {
            ef1: ExtendedFilter1(0),
            ef2: ExtendedFilter2(0),
        };
        s.ef2.set_eft(FilterType::Disable);
        s.ef1.set_efec(FilterConfiguration::Disable);

        s
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub(super) struct RxBufferElement {
    // a bit awkward to split this into two structs, but this is hidden from downstream consumers anyways.
    pub hdr: MsgHeader,
    pub rxhdr: RxHeader,
    pub data: [u8; MAX_DATA_LEN],
}

impl Default for RxBufferElement {
    fn default() -> Self {
        Self {
            hdr: MsgHeader(0),
            rxhdr: RxHeader(0),
            data: [0u8; MAX_DATA_LEN],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub(super) struct TxBufferElement {
    // a bit awkward to split this into two structs, but this is hidden from downstream consumers anyways.
    pub hdr: MsgHeader,
    pub txhdr: TxHeader,
    pub data: [u8; MAX_DATA_LEN],
}

impl Default for TxBufferElement {
    fn default() -> Self {
        Self {
            hdr: MsgHeader(0),
            txhdr: TxHeader(0),
            data: [0u8; MAX_DATA_LEN],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub(super) struct TxEventElement {
    // a bit awkward to split this into two structs, but this is hidden from downstream consumers anyways.
    pub hdr: MsgHeader,
    pub event: EventHeader,
}

impl Default for TxEventElement {
    fn default() -> Self {
        Self {
            hdr: MsgHeader(0),
            event: EventHeader(0),
        }
    }
}

/// Structure to represent the data within Message RAM of the CANFD / MCAN peripheral.
/// Note that on TI parts, this data lives at the base address of the CANFD/MCAN peripheral (this is not yet documented by TI, though.)
#[repr(C)]
pub(super) struct McanMessageRAM {
    filters: [StandardFilter; 0],
    extended_filters: [ExtendedFilter; 0],
    rxfifo0: [RxBufferElement; NUM_RX_ELEMENTS],
    rxfifo1: [RxBufferElement; 0], // Note: while we're not using most of these features yet, I've included their offsets and structures anyways to save the next person some pain.
    rxbuffers: [RxBufferElement; 0],
    txevents: [TxEventElement; NUM_TX_EVENTS],
    txfifo0: [TxBufferElement; NUM_TX_ELEMENTS],
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(super) struct MessageRamOffsetsSizes {
    pub filters: usize,
    pub extended_filters: usize,
    pub rxfifo0: usize,
    pub rxfifo1: usize,
    pub rxbuffers: usize,
    pub txevents: usize,
    pub txfifo: usize,
}

impl McanMessageRAM {
    /// Determine the sizes of the various arrays at compile time,
    /// will be used to configure the peripheral.
    pub const SIZES: MessageRamOffsetsSizes = {
        // trying to do this without needing to instantiate anything.
        let num_filters =
            core::mem::offset_of!(McanMessageRAM, extended_filters) / core::mem::size_of::<StandardFilter>();
        let num_extended = (core::mem::offset_of!(McanMessageRAM, rxfifo0)
            - core::mem::offset_of!(McanMessageRAM, extended_filters))
            / core::mem::size_of::<ExtendedFilter>();
        let num_rxfifo0 = (core::mem::offset_of!(McanMessageRAM, rxfifo1)
            - core::mem::offset_of!(McanMessageRAM, rxfifo0))
            / core::mem::size_of::<RxBufferElement>();
        let num_rxfifo1 = (core::mem::offset_of!(McanMessageRAM, rxbuffers)
            - core::mem::offset_of!(McanMessageRAM, rxfifo1))
            / core::mem::size_of::<RxBufferElement>();
        let num_rxbuffers = (core::mem::offset_of!(McanMessageRAM, txevents)
            - core::mem::offset_of!(McanMessageRAM, rxbuffers))
            / core::mem::size_of::<RxBufferElement>();
        let num_txevents = (core::mem::offset_of!(McanMessageRAM, txfifo0)
            - core::mem::offset_of!(McanMessageRAM, txevents))
            / core::mem::size_of::<TxEventElement>();
        let num_txfifo = (core::mem::size_of::<McanMessageRAM>() - core::mem::offset_of!(McanMessageRAM, txfifo0))
            / core::mem::size_of::<TxBufferElement>();

        if core::mem::size_of::<McanMessageRAM>() > 1024 {
            core::panic!("message RAM too large!");
        }

        MessageRamOffsetsSizes {
            filters: num_filters,
            extended_filters: num_extended,
            rxfifo0: num_rxfifo0,
            rxfifo1: num_rxfifo1,
            rxbuffers: num_rxbuffers,
            txevents: num_txevents,
            txfifo: num_txfifo,
        }
    };

    /// Determine the offsets of the various arrays at compile time,
    /// will be used to configure the peripheral.
    pub const OFFSETS: MessageRamOffsetsSizes = {
        MessageRamOffsetsSizes {
            filters: 0,
            extended_filters: core::mem::offset_of!(McanMessageRAM, extended_filters) / 4,
            rxfifo0: core::mem::offset_of!(McanMessageRAM, rxfifo0) / 4,
            rxfifo1: core::mem::offset_of!(McanMessageRAM, rxfifo1) / 4,
            rxbuffers: core::mem::offset_of!(McanMessageRAM, rxbuffers) / 4,
            txevents: core::mem::offset_of!(McanMessageRAM, txevents) / 4,
            txfifo: core::mem::offset_of!(McanMessageRAM, txfifo0) / 4,
        }
    };
}

macro_rules! impl_ram_access {
    // 1. Match both Getter and Setter
    ($field:ident, $getter:ident, $setter:ident, $el_type:ty) => {
        impl_ram_access!(@gen_getter $field, $getter, $el_type);
        impl_ram_access!(@gen_setter $field, $setter, $el_type);
    };

    // 2. Match Getter, skip Setter
    ($field:ident, $getter:ident, _, $el_type:ty) => {
        impl_ram_access!(@gen_getter $field, $getter, $el_type);
    };

    // 3. Skip Getter, match Setter
    ($field:ident, _, $setter:ident, $el_type:ty) => {
        impl_ram_access!(@gen_setter $field, $setter, $el_type);
    };


    (@gen_getter $field:ident, $getter:ident, $el_type:ty) => {
        pub(super) fn $getter(&self, idx: usize) -> Option<$el_type> {
            unsafe {
                // type check.
                let _: *const [$el_type; _] = core::ptr::addr_of!((*self.ptr).$field);

                if idx >= (*self.ptr).$field.len() {
                    return None;
                }

                let array_ptr = core::ptr::addr_of!((*self.ptr).$field) as *const $el_type;
                Some(array_ptr.add(idx).read_volatile())
            }
        }
    };

    (@gen_setter $field:ident, $setter:ident, $el_type:ty) => {
        // Note that this setter takes a non-mutable reference yet does mutate data.
        // The PACs do the same thing - a writable Reg has a function:
        // `pub fn write_value(&self, val: T)`
        // which effectively ignores ownership rules.
        //
        // This is following the same pattern, because this module is effectively doing the same job as the PAC,
        // (offering access to MMIO registers), but can't be directly included in the PAC as the offsets, sizes,
        // etc are configurable which doesn't seem to be something that chiptool/svd2rust can really support code-gen wise.
        //
        // We'll be super careful inside of these unsafe blocks to do the right thing, but we will ignore ownership rules
        // just like the PAC does - it's up to the rest of the HAL to use this with great caution and expose a truly sound
        // API to consumers.
        pub(super) fn $setter(&self, idx: usize, val: $el_type) -> Option<()> {
            unsafe {
                // type check.
                let _: *const [$el_type; _] = core::ptr::addr_of!((*self.ptr).$field);

                if idx >= (*self.ptr).$field.len() {
                    return None;
                }

                let array_ptr = core::ptr::addr_of_mut!((*self.ptr).$field) as *mut $el_type;
                array_ptr.add(idx).write_volatile(val);
                Some(())
            }
        }
    };
}

/// register/pac-like access to MessageRAM at a specific address.
pub(crate) struct MessageRAMAccess {
    ptr: *mut McanMessageRAM,
}
impl MessageRAMAccess {
    #[inline(always)]
    pub const unsafe fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr: ptr as _ }
    }

    impl_ram_access!(rxfifo0, get_rx_fifo_element, _, RxBufferElement);
    impl_ram_access!(txfifo0, _, set_tx_element, TxBufferElement);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_getters_setters_work() {
        let mut msgram = McanMessageRAM {
            filters: [StandardFilter::default(); 0],
            extended_filters: [ExtendedFilter::default(); 0],
            rxfifo0: [RxBufferElement::default(); NUM_RX_ELEMENTS],
            rxfifo1: [RxBufferElement::default(); 0],
            rxbuffers: [RxBufferElement::default(); 0],
            txevents: [TxEventElement::default(); NUM_TX_EVENTS],
            txfifo0: [TxBufferElement::default(); NUM_TX_ELEMENTS],
        };

        msgram.rxfifo0[0].data[0] = 0xDE;
        msgram.rxfifo0[0].data[1] = 0xAD;
        msgram.rxfifo0[0].data[2] = 0xCA;
        msgram.rxfifo0[0].data[3] = 0xFE;
        msgram.rxfifo0[0].rxhdr.set_dlc(4);

        let element_bkup = msgram.rxfifo0[0].clone();

        let ramacess: MessageRAMAccess = MessageRAMAccess { ptr: &mut msgram };

        assert!(ramacess.get_rx_fifo_element(NUM_RX_ELEMENTS).is_none());
        let element = ramacess.get_rx_fifo_element(0).unwrap();
        assert_eq!(element_bkup, element);

        assert!(
            ramacess
                .set_tx_element(NUM_TX_ELEMENTS + 1, TxBufferElement::default())
                .is_none()
        );
        let mut txelement = TxBufferElement::default();
        txelement.data[0] = 0x55;
        txelement.data[1] = 0xAA;
        txelement.data[2] = 0xBB;
        txelement.data[3] = 0xCC;

        let txelement_bak = txelement.clone();
        assert!(ramacess.set_tx_element(0, txelement).is_some());

        assert_eq!(msgram.txfifo0[0], txelement_bak);
    }
}
