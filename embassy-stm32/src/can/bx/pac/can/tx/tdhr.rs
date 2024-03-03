#[doc = "Reader of register TDHR"]
pub type R = crate::R<u32, super::TDHR>;
#[doc = "Writer for register TDHR"]
pub type W = crate::W<u32, super::TDHR>;
#[doc = "Register TDHR `reset()`'s with value 0"]
impl crate::ResetValue for super::TDHR {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `DATA7`"]
pub type DATA7_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `DATA7`"]
pub struct DATA7_W<'a> {
    w: &'a mut W,
}
impl<'a> DATA7_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0xff << 24)) | (((value as u32) & 0xff) << 24);
        self.w
    }
}
#[doc = "Reader of field `DATA6`"]
pub type DATA6_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `DATA6`"]
pub struct DATA6_W<'a> {
    w: &'a mut W,
}
impl<'a> DATA6_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0xff << 16)) | (((value as u32) & 0xff) << 16);
        self.w
    }
}
#[doc = "Reader of field `DATA5`"]
pub type DATA5_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `DATA5`"]
pub struct DATA5_W<'a> {
    w: &'a mut W,
}
impl<'a> DATA5_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0xff << 8)) | (((value as u32) & 0xff) << 8);
        self.w
    }
}
#[doc = "Reader of field `DATA4`"]
pub type DATA4_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `DATA4`"]
pub struct DATA4_W<'a> {
    w: &'a mut W,
}
impl<'a> DATA4_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u32) & 0xff);
        self.w
    }
}
impl R {
    #[doc = "Bits 24:31 - DATA7"]
    #[inline(always)]
    pub fn data7(&self) -> DATA7_R {
        DATA7_R::new(((self.bits >> 24) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - DATA6"]
    #[inline(always)]
    pub fn data6(&self) -> DATA6_R {
        DATA6_R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - DATA5"]
    #[inline(always)]
    pub fn data5(&self) -> DATA5_R {
        DATA5_R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 0:7 - DATA4"]
    #[inline(always)]
    pub fn data4(&self) -> DATA4_R {
        DATA4_R::new((self.bits & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 24:31 - DATA7"]
    #[inline(always)]
    pub fn data7(&mut self) -> DATA7_W {
        DATA7_W { w: self }
    }
    #[doc = "Bits 16:23 - DATA6"]
    #[inline(always)]
    pub fn data6(&mut self) -> DATA6_W {
        DATA6_W { w: self }
    }
    #[doc = "Bits 8:15 - DATA5"]
    #[inline(always)]
    pub fn data5(&mut self) -> DATA5_W {
        DATA5_W { w: self }
    }
    #[doc = "Bits 0:7 - DATA4"]
    #[inline(always)]
    pub fn data4(&mut self) -> DATA4_W {
        DATA4_W { w: self }
    }
}
