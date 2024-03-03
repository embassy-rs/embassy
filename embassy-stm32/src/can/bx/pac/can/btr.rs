#[doc = "Reader of register BTR"]
pub type R = crate::R<u32, super::BTR>;
#[doc = "Writer for register BTR"]
pub type W = crate::W<u32, super::BTR>;
#[doc = "Register BTR `reset()`'s with value 0"]
impl crate::ResetValue for super::BTR {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "SILM\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SILM_A {
    #[doc = "0: Normal operation"]
    NORMAL = 0,
    #[doc = "1: Silent Mode"]
    SILENT = 1,
}
impl From<SILM_A> for bool {
    #[inline(always)]
    fn from(variant: SILM_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `SILM`"]
pub type SILM_R = crate::R<bool, SILM_A>;
impl SILM_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> SILM_A {
        match self.bits {
            false => SILM_A::NORMAL,
            true => SILM_A::SILENT,
        }
    }
    #[doc = "Checks if the value of the field is `NORMAL`"]
    #[inline(always)]
    pub fn is_normal(&self) -> bool {
        *self == SILM_A::NORMAL
    }
    #[doc = "Checks if the value of the field is `SILENT`"]
    #[inline(always)]
    pub fn is_silent(&self) -> bool {
        *self == SILM_A::SILENT
    }
}
#[doc = "Write proxy for field `SILM`"]
pub struct SILM_W<'a> {
    w: &'a mut W,
}
impl<'a> SILM_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: SILM_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Normal operation"]
    #[inline(always)]
    pub fn normal(self) -> &'a mut W {
        self.variant(SILM_A::NORMAL)
    }
    #[doc = "Silent Mode"]
    #[inline(always)]
    pub fn silent(self) -> &'a mut W {
        self.variant(SILM_A::SILENT)
    }
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
        self.w.bits = (self.w.bits & !(0x01 << 31)) | (((value as u32) & 0x01) << 31);
        self.w
    }
}
#[doc = "LBKM\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LBKM_A {
    #[doc = "0: Loop Back Mode disabled"]
    DISABLED = 0,
    #[doc = "1: Loop Back Mode enabled"]
    ENABLED = 1,
}
impl From<LBKM_A> for bool {
    #[inline(always)]
    fn from(variant: LBKM_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `LBKM`"]
pub type LBKM_R = crate::R<bool, LBKM_A>;
impl LBKM_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> LBKM_A {
        match self.bits {
            false => LBKM_A::DISABLED,
            true => LBKM_A::ENABLED,
        }
    }
    #[doc = "Checks if the value of the field is `DISABLED`"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == LBKM_A::DISABLED
    }
    #[doc = "Checks if the value of the field is `ENABLED`"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == LBKM_A::ENABLED
    }
}
#[doc = "Write proxy for field `LBKM`"]
pub struct LBKM_W<'a> {
    w: &'a mut W,
}
impl<'a> LBKM_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: LBKM_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Loop Back Mode disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut W {
        self.variant(LBKM_A::DISABLED)
    }
    #[doc = "Loop Back Mode enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut W {
        self.variant(LBKM_A::ENABLED)
    }
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
        self.w.bits = (self.w.bits & !(0x01 << 30)) | (((value as u32) & 0x01) << 30);
        self.w
    }
}
#[doc = "Reader of field `SJW`"]
pub type SJW_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SJW`"]
pub struct SJW_W<'a> {
    w: &'a mut W,
}
impl<'a> SJW_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 24)) | (((value as u32) & 0x03) << 24);
        self.w
    }
}
#[doc = "Reader of field `TS2`"]
pub type TS2_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `TS2`"]
pub struct TS2_W<'a> {
    w: &'a mut W,
}
impl<'a> TS2_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x07 << 20)) | (((value as u32) & 0x07) << 20);
        self.w
    }
}
#[doc = "Reader of field `TS1`"]
pub type TS1_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `TS1`"]
pub struct TS1_W<'a> {
    w: &'a mut W,
}
impl<'a> TS1_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 16)) | (((value as u32) & 0x0f) << 16);
        self.w
    }
}
#[doc = "Reader of field `BRP`"]
pub type BRP_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `BRP`"]
pub struct BRP_W<'a> {
    w: &'a mut W,
}
impl<'a> BRP_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03ff) | ((value as u32) & 0x03ff);
        self.w
    }
}
impl R {
    #[doc = "Bit 31 - SILM"]
    #[inline(always)]
    pub fn silm(&self) -> SILM_R {
        SILM_R::new(((self.bits >> 31) & 0x01) != 0)
    }
    #[doc = "Bit 30 - LBKM"]
    #[inline(always)]
    pub fn lbkm(&self) -> LBKM_R {
        LBKM_R::new(((self.bits >> 30) & 0x01) != 0)
    }
    #[doc = "Bits 24:25 - SJW"]
    #[inline(always)]
    pub fn sjw(&self) -> SJW_R {
        SJW_R::new(((self.bits >> 24) & 0x03) as u8)
    }
    #[doc = "Bits 20:22 - TS2"]
    #[inline(always)]
    pub fn ts2(&self) -> TS2_R {
        TS2_R::new(((self.bits >> 20) & 0x07) as u8)
    }
    #[doc = "Bits 16:19 - TS1"]
    #[inline(always)]
    pub fn ts1(&self) -> TS1_R {
        TS1_R::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bits 0:9 - BRP"]
    #[inline(always)]
    pub fn brp(&self) -> BRP_R {
        BRP_R::new((self.bits & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bit 31 - SILM"]
    #[inline(always)]
    pub fn silm(&mut self) -> SILM_W {
        SILM_W { w: self }
    }
    #[doc = "Bit 30 - LBKM"]
    #[inline(always)]
    pub fn lbkm(&mut self) -> LBKM_W {
        LBKM_W { w: self }
    }
    #[doc = "Bits 24:25 - SJW"]
    #[inline(always)]
    pub fn sjw(&mut self) -> SJW_W {
        SJW_W { w: self }
    }
    #[doc = "Bits 20:22 - TS2"]
    #[inline(always)]
    pub fn ts2(&mut self) -> TS2_W {
        TS2_W { w: self }
    }
    #[doc = "Bits 16:19 - TS1"]
    #[inline(always)]
    pub fn ts1(&mut self) -> TS1_W {
        TS1_W { w: self }
    }
    #[doc = "Bits 0:9 - BRP"]
    #[inline(always)]
    pub fn brp(&mut self) -> BRP_W {
        BRP_W { w: self }
    }
}
