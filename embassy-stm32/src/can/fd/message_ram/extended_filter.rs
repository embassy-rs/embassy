// Note: This file is copied and modified from fdcan crate by Richard Meadows

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use super::common::{ESFEC_R, ESFT_R};
use super::enums::{FilterElementConfig, FilterType};
use super::generic;

#[doc = "Reader of register ExtendedFilter"]
pub(crate) type R = generic::R<super::ExtendedFilterType, super::ExtendedFilter>;
#[doc = "Writer for register ExtendedFilter"]
pub(crate) type W = generic::W<super::ExtendedFilterType, super::ExtendedFilter>;
#[doc = "Register ExtendedFilter `reset()`'s"]
impl generic::ResetValue for super::ExtendedFilter {
    type Type = super::ExtendedFilterType;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        // Sets filter element to Disabled
        [0x0, 0x0]
    }
}

#[doc = "Reader of field `EFID2`"]
pub(crate) type EFID2_R = generic::R<u32, u32>;
#[doc = "Write proxy for field `EFID2`"]
pub(crate) struct EFID2_W<'a> {
    w: &'a mut W,
}
impl<'a> EFID2_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u32) -> &'a mut W {
        self.w.bits[1] = (self.w.bits[1] & !(0x1FFFFFFF)) | ((value as u32) & 0x1FFFFFFF);
        self.w
    }
}

#[doc = "Reader of field `EFID1`"]
pub(crate) type EFID1_R = generic::R<u32, u32>;
#[doc = "Write proxy for field `EFID1`"]
pub(crate) struct EFID1_W<'a> {
    w: &'a mut W,
}
impl<'a> EFID1_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u32) -> &'a mut W {
        self.w.bits[0] = (self.w.bits[0] & !(0x1FFFFFFF)) | ((value as u32) & 0x1FFFFFFF);
        self.w
    }
}

#[doc = "Write proxy for field `EFEC`"]
pub(crate) struct EFEC_W<'a> {
    w: &'a mut W,
}
impl<'a> EFEC_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits[0] = (self.w.bits[0] & !(0x07 << 29)) | (((value as u32) & 0x07) << 29);
        self.w
    }
    #[doc = r"Sets the field according to FilterElementConfig"]
    #[inline(always)]
    pub fn set_filter_element_config(self, fec: FilterElementConfig) -> &'a mut W {
        //SAFETY: FilterElementConfig only be valid options
        unsafe { self.bits(fec as u8) }
    }
}

#[doc = "Write proxy for field `EFT`"]
pub(crate) struct EFT_W<'a> {
    w: &'a mut W,
}
impl<'a> EFT_W<'a> {
    #[doc = r"Sets the field according the FilterType"]
    #[inline(always)]
    pub fn set_filter_type(self, filter: FilterType) -> &'a mut W {
        //SAFETY: FilterType only be valid options
        unsafe { self.bits(filter as u8) }
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits[1] = (self.w.bits[1] & !(0x03 << 30)) | (((value as u32) & 0x03) << 30);
        self.w
    }
}

impl R {
    #[doc = "Byte 0 - Bits 0:28 - EFID1"]
    #[inline(always)]
    pub fn sfid1(&self) -> EFID1_R {
        EFID1_R::new(((self.bits[0]) & 0x1FFFFFFF) as u32)
    }
    #[doc = "Byte 0 - Bits 29:31 - EFEC"]
    #[inline(always)]
    pub fn efec(&self) -> ESFEC_R {
        ESFEC_R::new(((self.bits[0] >> 29) & 0x07) as u8)
    }
    #[doc = "Byte 1 - Bits 0:28 - EFID2"]
    #[inline(always)]
    pub fn sfid2(&self) -> EFID2_R {
        EFID2_R::new(((self.bits[1]) & 0x1FFFFFFF) as u32)
    }
    #[doc = "Byte 1 - Bits 30:31 - EFT"]
    #[inline(always)]
    pub fn eft(&self) -> ESFT_R {
        ESFT_R::new(((self.bits[1] >> 30) & 0x03) as u8)
    }
}
impl W {
    #[doc = "Byte 0 - Bits 0:28 - EFID1"]
    #[inline(always)]
    pub fn efid1(&mut self) -> EFID1_W {
        EFID1_W { w: self }
    }
    #[doc = "Byte 0 - Bits 29:31 - EFEC"]
    #[inline(always)]
    pub fn efec(&mut self) -> EFEC_W {
        EFEC_W { w: self }
    }
    #[doc = "Byte 1 - Bits 0:28 - EFID2"]
    #[inline(always)]
    pub fn efid2(&mut self) -> EFID2_W {
        EFID2_W { w: self }
    }
    #[doc = "Byte 1 - Bits 30:31 - EFT"]
    #[inline(always)]
    pub fn eft(&mut self) -> EFT_W {
        EFT_W { w: self }
    }
}
