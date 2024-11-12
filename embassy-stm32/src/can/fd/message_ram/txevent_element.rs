// Note: This file is copied and modified from fdcan crate by Richard Meadows

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use super::common::{BRS_R, DLC_R, ESI_R, RTR_R, XTD_R};
use super::generic;

#[doc = "Reader of register TxEventElement"]
pub(crate) type R = generic::R<super::TxEventElementType, super::TxEventElement>;
// #[doc = "Writer for register TxEventElement"]
// pub(crate) type W = generic::W<super::TxEventElementType, super::TxEventElement>;
#[doc = "Register TxEventElement `reset()`'s"]
impl generic::ResetValue for super::TxEventElement {
    type Type = super::TxEventElementType;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        [0, 0]
    }
}

#[doc = "Reader of field `ID`"]
pub(crate) type ID_R = generic::R<u32, u32>;

#[doc = "Reader of field `TXTS`"]
pub(crate) type TXTS_R = generic::R<u16, u16>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DataLengthFormat {
    StandardLength = 0,
    FDCANLength = 1,
}
impl From<DataLengthFormat> for bool {
    #[inline(always)]
    fn from(dlf: DataLengthFormat) -> Self {
        dlf as u8 != 0
    }
}

#[doc = "Reader of field `EDL`"]
pub(crate) type EDL_R = generic::R<bool, DataLengthFormat>;
impl EDL_R {
    pub fn data_length_format(&self) -> DataLengthFormat {
        match self.bits() {
            false => DataLengthFormat::StandardLength,
            true => DataLengthFormat::FDCANLength,
        }
    }
    pub fn is_standard_length(&self) -> bool {
        *self == DataLengthFormat::StandardLength
    }
    pub fn is_fdcan_length(&self) -> bool {
        *self == DataLengthFormat::FDCANLength
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum EventType {
    //_Reserved = 0b00,
    TxEvent = 0b01,
    TxDespiteAbort = 0b10,
    //_Reserved = 0b10,
}

#[doc = "Reader of field `EFC`"]
pub(crate) type EFC_R = generic::R<u8, EventType>;
impl EFC_R {
    pub fn event_type(&self) -> EventType {
        match self.bits() {
            0b01 => EventType::TxEvent,
            0b10 => EventType::TxDespiteAbort,
            _ => unimplemented!(),
        }
    }
    pub fn is_tx_event(&self) -> bool {
        self.event_type() == EventType::TxEvent
    }
    pub fn is_despite_abort(&self) -> bool {
        self.event_type() == EventType::TxDespiteAbort
    }
}

#[doc = "Reader of field `MM`"]
pub(crate) type MM_R = generic::R<u8, u8>;

impl R {
    #[doc = "Byte 0 - Bits 0:28 - ID"]
    #[inline(always)]
    pub fn id(&self) -> ID_R {
        ID_R::new(((self.bits[0]) & 0x1FFFFFFF) as u32)
    }
    #[doc = "Byte 0 - Bit 29 - RTR"]
    #[inline(always)]
    pub fn rtr(&self) -> RTR_R {
        RTR_R::new(((self.bits[0] >> 29) & 0x01) != 0)
    }
    #[doc = "Byte 0 - Bit 30 - XTD"]
    #[inline(always)]
    pub fn xtd(&self) -> XTD_R {
        XTD_R::new(((self.bits[0] >> 30) & 0x01) != 0)
    }
    #[doc = "Byte 0 - Bit 30 - ESI"]
    #[inline(always)]
    pub fn esi(&self) -> ESI_R {
        ESI_R::new(((self.bits[0] >> 31) & 0x01) != 0)
    }
    #[doc = "Byte 1 - Bits 0:15 - TXTS"]
    #[inline(always)]
    pub fn txts(&self) -> TXTS_R {
        TXTS_R::new(((self.bits[1]) & 0xFFFF) as u16)
    }
    #[doc = "Byte 1 - Bits 16:19 - DLC"]
    #[inline(always)]
    pub fn dlc(&self) -> DLC_R {
        DLC_R::new(((self.bits[1] >> 16) & 0x0F) as u8)
    }
    #[doc = "Byte 1 - Bits 20 - BRS"]
    #[inline(always)]
    pub fn brs(&self) -> BRS_R {
        BRS_R::new(((self.bits[1] >> 20) & 0x01) != 0)
    }
    #[doc = "Byte 1 - Bits 21 - EDL"]
    #[inline(always)]
    pub fn edl(&self) -> EDL_R {
        EDL_R::new(((self.bits[1] >> 21) & 0x01) != 0)
    }
    #[doc = "Byte 1 - Bits 22:23 - EFC"]
    #[inline(always)]
    pub fn efc(&self) -> EFC_R {
        EFC_R::new(((self.bits[1] >> 22) & 0x03) as u8)
    }
    #[doc = "Byte 1 - Bits 24:31 - MM"]
    #[inline(always)]
    pub fn mm(&self) -> MM_R {
        MM_R::new(((self.bits[1] >> 24) & 0xFF) as u8)
    }
}
