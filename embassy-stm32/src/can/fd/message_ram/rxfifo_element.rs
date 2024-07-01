// Note: This file is copied and modified from fdcan crate by Richard Meadows

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use super::common::{BRS_R, DLC_R, ESI_R, FDF_R, ID_R, RTR_R, XTD_R};
use super::enums::{DataLength, FilterFrameMatch, FrameFormat};
use super::generic;

#[doc = "Reader of register RxFifoElement"]
pub(crate) type R = generic::R<super::RxFifoElementHeaderType, super::RxFifoElementHeader>;
// #[doc = "Writer for register ExtendedFilter"]
// pub(crate) type W = generic::W<super::RxFifoElementHeaderType, super::RxFifoElementHeader>;
#[doc = "Register ExtendedFilter `reset()`'s"]
impl generic::ResetValue for super::RxFifoElementHeader {
    type Type = super::RxFifoElementHeaderType;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        [0x0, 0x0]
    }
}

#[doc = "Reader of field `RXTS`"]
pub(crate) type RXTS_R = generic::R<u16, u16>;

#[doc = "Reader of field `FIDX`"]
pub(crate) type FIDX_R = generic::R<u8, u8>;

pub(crate) struct _ANMF;
#[doc = "Reader of field `ANMF`"]
pub(crate) type ANMF_R = generic::R<bool, _ANMF>;
impl ANMF_R {
    pub fn is_matching_frame(&self) -> bool {
        self.bit_is_clear()
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
    #[doc = "Byte 1 - Bits 0:15 - RXTS"]
    #[inline(always)]
    pub fn txts(&self) -> RXTS_R {
        RXTS_R::new(((self.bits[1]) & 0xFFFF) as u16)
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
    #[doc = "Byte 1 - Bits 24:30 - FIDX"]
    #[inline(always)]
    pub fn fidx(&self) -> FIDX_R {
        FIDX_R::new(((self.bits[1] >> 24) & 0xFF) as u8)
    }
    #[doc = "Byte 1 - Bits 31 - ANMF"]
    #[inline(always)]
    pub fn anmf(&self) -> ANMF_R {
        ANMF_R::new(((self.bits[1] >> 31) & 0x01) != 0)
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
    pub fn to_filter_match(&self) -> FilterFrameMatch {
        if self.anmf().is_matching_frame() {
            FilterFrameMatch::DidMatch(self.fidx().bits())
        } else {
            FilterFrameMatch::DidNotMatch
        }
    }
}
