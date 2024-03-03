#[doc = "Reader of register TDTR"]
pub type R = crate::R<u32, super::TDTR>;
#[doc = "Writer for register TDTR"]
pub type W = crate::W<u32, super::TDTR>;
#[doc = "Register TDTR `reset()`'s with value 0"]
impl crate::ResetValue for super::TDTR {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `TIME`"]
pub type TIME_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `TIME`"]
pub struct TIME_W<'a> {
    w: &'a mut W,
}
impl<'a> TIME_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0xffff << 16)) | (((value as u32) & 0xffff) << 16);
        self.w
    }
}
#[doc = "Reader of field `TGT`"]
pub type TGT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TGT`"]
pub struct TGT_W<'a> {
    w: &'a mut W,
}
impl<'a> TGT_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 8)) | (((value as u32) & 0x01) << 8);
        self.w
    }
}
#[doc = "Reader of field `DLC`"]
pub type DLC_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `DLC`"]
pub struct DLC_W<'a> {
    w: &'a mut W,
}
impl<'a> DLC_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x0f) | ((value as u32) & 0x0f);
        self.w
    }
}
impl R {
    #[doc = "Bits 16:31 - TIME"]
    #[inline(always)]
    pub fn time(&self) -> TIME_R {
        TIME_R::new(((self.bits >> 16) & 0xffff) as u16)
    }
    #[doc = "Bit 8 - TGT"]
    #[inline(always)]
    pub fn tgt(&self) -> TGT_R {
        TGT_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bits 0:3 - DLC"]
    #[inline(always)]
    pub fn dlc(&self) -> DLC_R {
        DLC_R::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 16:31 - TIME"]
    #[inline(always)]
    pub fn time(&mut self) -> TIME_W {
        TIME_W { w: self }
    }
    #[doc = "Bit 8 - TGT"]
    #[inline(always)]
    pub fn tgt(&mut self) -> TGT_W {
        TGT_W { w: self }
    }
    #[doc = "Bits 0:3 - DLC"]
    #[inline(always)]
    pub fn dlc(&mut self) -> DLC_W {
        DLC_W { w: self }
    }
}
