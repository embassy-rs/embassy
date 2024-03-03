#[doc = "Reader of register FMR"]
pub type R = crate::R<u32, super::FMR>;
#[doc = "Writer for register FMR"]
pub type W = crate::W<u32, super::FMR>;
#[doc = "Register FMR `reset()`'s with value 0"]
impl crate::ResetValue for super::FMR {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `CAN2SB`"]
pub type CAN2SB_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `CAN2SB`"]
pub struct CAN2SB_W<'a> {
    w: &'a mut W,
}
impl<'a> CAN2SB_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x3f << 8)) | (((value as u32) & 0x3f) << 8);
        self.w
    }
}
#[doc = "Reader of field `FINIT`"]
pub type FINIT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FINIT`"]
pub struct FINIT_W<'a> {
    w: &'a mut W,
}
impl<'a> FINIT_W<'a> {
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
        self.w.bits = (self.w.bits & !0x01) | ((value as u32) & 0x01);
        self.w
    }
}
impl R {
    #[doc = "Bits 8:13 - CAN2SB"]
    #[inline(always)]
    pub fn can2sb(&self) -> CAN2SB_R {
        CAN2SB_R::new(((self.bits >> 8) & 0x3f) as u8)
    }
    #[doc = "Bit 0 - FINIT"]
    #[inline(always)]
    pub fn finit(&self) -> FINIT_R {
        FINIT_R::new((self.bits & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 8:13 - CAN2SB"]
    #[inline(always)]
    pub fn can2sb(&mut self) -> CAN2SB_W {
        CAN2SB_W { w: self }
    }
    #[doc = "Bit 0 - FINIT"]
    #[inline(always)]
    pub fn finit(&mut self) -> FINIT_W {
        FINIT_W { w: self }
    }
}
