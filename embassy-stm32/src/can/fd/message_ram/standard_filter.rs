// Note: This file is copied and modified from fdcan crate by Richard Meadows

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use super::common::{ESFEC_R, ESFT_R};
use super::enums::{FilterElementConfig, FilterType};
use super::generic;

#[doc = "Reader of register StandardFilter"]
pub(crate) type R = generic::R<super::StandardFilterType, super::StandardFilter>;
#[doc = "Writer for register StandardFilter"]
pub(crate) type W = generic::W<super::StandardFilterType, super::StandardFilter>;
#[doc = "Register StandardFilter `reset()`'s with value 0xC0000"]
impl generic::ResetValue for super::StandardFilter {
    type Type = super::StandardFilterType;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        // Sets filter element to Disabled
        0xC000
    }
}

#[doc = "Reader of field `SFID2`"]
pub(crate) type SFID2_R = generic::R<u16, u16>;
#[doc = "Write proxy for field `SFID2`"]
pub(crate) struct SFID2_W<'a> {
    w: &'a mut W,
}
impl<'a> SFID2_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x07ff)) | ((value as u32) & 0x07ff);
        self.w
    }
}

#[doc = "Reader of field `SFID1`"]
pub(crate) type SFID1_R = generic::R<u16, u16>;
#[doc = "Write proxy for field `SFID1`"]
pub(crate) struct SFID1_W<'a> {
    w: &'a mut W,
}
impl<'a> SFID1_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x07ff << 16)) | (((value as u32) & 0x07ff) << 16);
        self.w
    }
}

#[doc = "Write proxy for field `SFEC`"]
pub(crate) struct SFEC_W<'a> {
    w: &'a mut W,
}
impl<'a> SFEC_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x07 << 27)) | (((value as u32) & 0x07) << 27);
        self.w
    }
    #[doc = r"Sets the field according to FilterElementConfig"]
    #[inline(always)]
    pub fn set_filter_element_config(self, fec: FilterElementConfig) -> &'a mut W {
        //SAFETY: FilterElementConfig only be valid options
        unsafe { self.bits(fec as u8) }
    }
}

#[doc = "Write proxy for field `SFT`"]
pub(crate) struct SFT_W<'a> {
    w: &'a mut W,
}
impl<'a> SFT_W<'a> {
    #[doc = r"Sets the field according the FilterType"]
    #[inline(always)]
    pub fn set_filter_type(self, filter: FilterType) -> &'a mut W {
        //SAFETY: FilterType only be valid options
        unsafe { self.bits(filter as u8) }
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 30)) | (((value as u32) & 0x03) << 30);
        self.w
    }
}

impl R {
    #[doc = "Bits 0:10 - SFID2"]
    #[inline(always)]
    pub fn sfid2(&self) -> SFID2_R {
        SFID2_R::new((self.bits & 0x07ff) as u16)
    }
    #[doc = "Bits 16:26 - SFID1"]
    #[inline(always)]
    pub fn sfid1(&self) -> SFID1_R {
        SFID1_R::new(((self.bits >> 16) & 0x07ff) as u16)
    }
    #[doc = "Bits 27:29 - SFEC"]
    #[inline(always)]
    pub fn sfec(&self) -> ESFEC_R {
        ESFEC_R::new(((self.bits >> 27) & 0x07) as u8)
    }
    #[doc = "Bits 30:31 - SFT"]
    #[inline(always)]
    pub fn sft(&self) -> ESFT_R {
        ESFT_R::new(((self.bits >> 30) & 0x03) as u8)
    }
}
impl W {
    #[doc = "Bits 0:10 - SFID2"]
    #[inline(always)]
    pub fn sfid2(&mut self) -> SFID2_W {
        SFID2_W { w: self }
    }
    #[doc = "Bits 16:26 - SFID1"]
    #[inline(always)]
    pub fn sfid1(&mut self) -> SFID1_W {
        SFID1_W { w: self }
    }
    #[doc = "Bits 27:29 - SFEC"]
    #[inline(always)]
    pub fn sfec(&mut self) -> SFEC_W {
        SFEC_W { w: self }
    }
    #[doc = "Bits 30:31 - SFT"]
    #[inline(always)]
    pub fn sft(&mut self) -> SFT_W {
        SFT_W { w: self }
    }
}
