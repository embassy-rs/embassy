// Note: This file is copied and modified from fdcan crate by Richard Meadows

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use super::common::{BRS_R, DLC_R, ESI_R, FDF_R, ID_R, RTR_R, XTD_R};
use super::enums::{
    BitRateSwitching, DataLength, ErrorStateIndicator, Event, EventControl, FrameFormat, IdType,
    RemoteTransmissionRequest,
};
use super::generic;

#[doc = "Reader of register TxBufferElement"]
pub(crate) type R = generic::R<super::TxBufferElementHeaderType, super::TxBufferElementHeader>;
#[doc = "Writer for register TxBufferElement"]
pub(crate) type W = generic::W<super::TxBufferElementHeaderType, super::TxBufferElementHeader>;
impl generic::ResetValue for super::TxBufferElementHeader {
    type Type = super::TxBufferElementHeaderType;

    #[allow(dead_code)]
    #[inline(always)]
    fn reset_value() -> Self::Type {
        [0; 2]
    }
}

#[doc = "Write proxy for field `ESI`"]
pub(crate) struct ESI_W<'a> {
    w: &'a mut W,
}
impl<'a> ESI_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_error_indicator(self, esi: ErrorStateIndicator) -> &'a mut W {
        self.bit(esi as u8 != 0)
    }

    #[doc = r"Sets the field bit"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits[0] = (self.w.bits[0] & !(0x01 << 31)) | (((value as u32) & 0x01) << 31);
        self.w
    }
}

#[doc = "Write proxy for field `XTD`"]
pub(crate) struct XTD_W<'a> {
    w: &'a mut W,
}
impl<'a> XTD_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_id_type(self, idt: IdType) -> &'a mut W {
        self.bit(idt as u8 != 0)
    }

    #[doc = r"Sets the field bit"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits[0] = (self.w.bits[0] & !(0x01 << 30)) | (((value as u32) & 0x01) << 30);
        self.w
    }
}

#[doc = "Write proxy for field `RTR`"]
pub(crate) struct RTR_W<'a> {
    w: &'a mut W,
}
impl<'a> RTR_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_rtr(self, rtr: RemoteTransmissionRequest) -> &'a mut W {
        self.bit(rtr as u8 != 0)
    }

    #[doc = r"Sets the field bit"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits[0] = (self.w.bits[0] & !(0x01 << 29)) | (((value as u32) & 0x01) << 29);
        self.w
    }
}

#[doc = "Write proxy for field `ID`"]
pub(crate) struct ID_W<'a> {
    w: &'a mut W,
}
impl<'a> ID_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub unsafe fn bits(self, value: u32) -> &'a mut W {
        self.w.bits[0] = (self.w.bits[0] & !(0x1FFFFFFF)) | ((value as u32) & 0x1FFFFFFF);
        self.w
    }
}

#[doc = "Write proxy for field `DLC`"]
pub(crate) struct DLC_W<'a> {
    w: &'a mut W,
}
impl<'a> DLC_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits[1] = (self.w.bits[1] & !(0x0F << 16)) | (((value as u32) & 0x0F) << 16);
        self.w
    }
}

#[doc = "Write proxy for field `BRS`"]
pub(crate) struct BRS_W<'a> {
    w: &'a mut W,
}
impl<'a> BRS_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_brs(self, brs: BitRateSwitching) -> &'a mut W {
        self.bit(brs as u8 != 0)
    }

    #[doc = r"Sets the field bit"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits[1] = (self.w.bits[1] & !(0x01 << 20)) | (((value as u32) & 0x01) << 20);
        self.w
    }
}

#[doc = "Write proxy for field `FDF`"]
pub(crate) struct FDF_W<'a> {
    w: &'a mut W,
}
impl<'a> FDF_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_format(self, fdf: FrameFormat) -> &'a mut W {
        self.bit(fdf as u8 != 0)
    }

    #[doc = r"Sets the field bit"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits[1] = (self.w.bits[1] & !(0x01 << 21)) | (((value as u32) & 0x01) << 21);
        self.w
    }
}

#[doc = "Reader of field `EFC`"]
pub(crate) type EFC_R = generic::R<bool, EventControl>;
impl EFC_R {
    pub fn to_event_control(&self) -> EventControl {
        match self.bit() {
            false => EventControl::DoNotStore,
            true => EventControl::Store,
        }
    }
}
#[doc = "Write proxy for field `EFC`"]
pub(crate) struct EFC_W<'a> {
    w: &'a mut W,
}
impl<'a> EFC_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_event_control(self, efc: EventControl) -> &'a mut W {
        self.bit(match efc {
            EventControl::DoNotStore => false,
            EventControl::Store => true,
        })
    }

    #[doc = r"Sets the field bit"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    #[allow(dead_code)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits[1] = (self.w.bits[1] & !(0x01 << 23)) | (((value as u32) & 0x01) << 23);
        self.w
    }
}

struct Marker(u8);
impl From<Event> for Marker {
    fn from(e: Event) -> Marker {
        match e {
            Event::NoEvent => Marker(0),
            Event::Event(mm) => Marker(mm),
        }
    }
}

#[doc = "Reader of field `MM`"]
pub(crate) type MM_R = generic::R<u8, u8>;
#[doc = "Write proxy for field `MM`"]
pub(crate) struct MM_W<'a> {
    w: &'a mut W,
}
impl<'a> MM_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits[1] = (self.w.bits[1] & !(0x7F << 24)) | (((value as u32) & 0x7F) << 24);
        self.w
    }

    fn set_message_marker(self, mm: Marker) -> &'a mut W {
        unsafe { self.bits(mm.0) }
    }
}

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
    #[doc = "Byte 1 - Bits 20 - FDF"]
    #[inline(always)]
    pub fn fdf(&self) -> FDF_R {
        FDF_R::new(((self.bits[1] >> 21) & 0x01) != 0)
    }
    #[doc = "Byte 1 - Bits 23 - EFC"]
    #[inline(always)]
    pub fn efc(&self) -> EFC_R {
        EFC_R::new(((self.bits[1] >> 23) & 0x01) != 0)
    }
    #[doc = "Byte 1 - Bits 24:31 - MM"]
    #[inline(always)]
    pub fn mm(&self) -> MM_R {
        MM_R::new(((self.bits[1] >> 24) & 0xFF) as u8)
    }
    pub fn to_data_length(&self) -> DataLength {
        let dlc = self.dlc().bits();
        let ff = self.fdf().frame_format();
        let len = if ff == FrameFormat::Fdcan {
            // See RM0433 Rev 7 Table 475. DLC coding
            match dlc {
                0..=8 => dlc,
                9 => 12,
                10 => 16,
                11 => 20,
                12 => 24,
                13 => 32,
                14 => 48,
                15 => 64,
                _ => panic!("DLC > 15"),
            }
        } else {
            match dlc {
                0..=8 => dlc,
                9..=15 => 8,
                _ => panic!("DLC > 15"),
            }
        };
        DataLength::new(len, ff)
    }
    pub fn to_event(&self) -> Event {
        let mm = self.mm().bits();
        let efc = self.efc().to_event_control();
        match efc {
            EventControl::DoNotStore => Event::NoEvent,
            EventControl::Store => Event::Event(mm),
        }
    }
}
impl W {
    #[doc = "Byte 0 - Bits 0:28 - ID"]
    #[inline(always)]
    pub fn id(&mut self) -> ID_W {
        ID_W { w: self }
    }
    #[doc = "Byte 0 - Bit 29 - RTR"]
    #[inline(always)]
    pub fn rtr(&mut self) -> RTR_W {
        RTR_W { w: self }
    }
    #[doc = "Byte 0 - Bit 30 - XTD"]
    #[inline(always)]
    pub fn xtd(&mut self) -> XTD_W {
        XTD_W { w: self }
    }
    #[doc = "Byte 0 - Bit 31 - ESI"]
    #[inline(always)]
    pub fn esi(&mut self) -> ESI_W {
        ESI_W { w: self }
    }
    #[doc = "Byte 1 - Bit 16:19 - DLC"]
    #[inline(always)]
    pub fn dlc(&mut self) -> DLC_W {
        DLC_W { w: self }
    }
    #[doc = "Byte 1 - Bit 20 - BRS"]
    #[inline(always)]
    pub fn brs(&mut self) -> BRS_W {
        BRS_W { w: self }
    }
    #[doc = "Byte 1 - Bit 21 - FDF"]
    #[inline(always)]
    pub fn fdf(&mut self) -> FDF_W {
        FDF_W { w: self }
    }
    #[doc = "Byte 1 - Bit 23 - EFC"]
    #[inline(always)]
    pub fn efc(&mut self) -> EFC_W {
        EFC_W { w: self }
    }
    #[doc = "Byte 1 - Bit 24:31 - MM"]
    #[inline(always)]
    pub fn mm(&mut self) -> MM_W {
        MM_W { w: self }
    }
    #[doc = "Convenience function for setting the data length and frame format"]
    #[inline(always)]
    pub fn set_len(&mut self, dl: impl Into<DataLength>) -> &mut Self {
        let dl: DataLength = dl.into();
        self.fdf().set_format(dl.into());
        unsafe { self.dlc().bits(dl.dlc()) }
    }
    pub fn set_event(&mut self, event: Event) -> &mut Self {
        self.mm().set_message_marker(event.into());
        self.efc().set_event_control(event.into())
    }
}
