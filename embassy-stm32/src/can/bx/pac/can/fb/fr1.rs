#[doc = "Reader of register FR1"]
pub type R = crate::R<u32, super::FR1>;
#[doc = "Writer for register FR1"]
pub type W = crate::W<u32, super::FR1>;
#[doc = "Register FR1 `reset()`'s with value 0"]
impl crate::ResetValue for super::FR1 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `FB`"]
pub type FB_R = crate::R<u32, u32>;
#[doc = "Write proxy for field `FB`"]
pub struct FB_W<'a> {
    w: &'a mut W,
}
impl<'a> FB_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u32) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xffff_ffff) | ((value as u32) & 0xffff_ffff);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:31 - Filter bits"]
    #[inline(always)]
    pub fn fb(&self) -> FB_R {
        FB_R::new((self.bits & 0xffff_ffff) as u32)
    }
}
impl W {
    #[doc = "Bits 0:31 - Filter bits"]
    #[inline(always)]
    pub fn fb(&mut self) -> FB_W {
        FB_W { w: self }
    }
}
