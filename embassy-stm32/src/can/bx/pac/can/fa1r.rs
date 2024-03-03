#[doc = "Reader of register FA1R"]
pub type R = crate::R<u32, super::FA1R>;
#[doc = "Writer for register FA1R"]
pub type W = crate::W<u32, super::FA1R>;
#[doc = "Register FA1R `reset()`'s with value 0"]
impl crate::ResetValue for super::FA1R {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `FACT0`"]
pub type FACT0_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT0`"]
pub struct FACT0_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT0_W<'a> {
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
#[doc = "Reader of field `FACT1`"]
pub type FACT1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT1`"]
pub struct FACT1_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT1_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 1)) | (((value as u32) & 0x01) << 1);
        self.w
    }
}
#[doc = "Reader of field `FACT2`"]
pub type FACT2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT2`"]
pub struct FACT2_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT2_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u32) & 0x01) << 2);
        self.w
    }
}
#[doc = "Reader of field `FACT3`"]
pub type FACT3_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT3`"]
pub struct FACT3_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT3_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 3)) | (((value as u32) & 0x01) << 3);
        self.w
    }
}
#[doc = "Reader of field `FACT4`"]
pub type FACT4_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT4`"]
pub struct FACT4_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT4_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 4)) | (((value as u32) & 0x01) << 4);
        self.w
    }
}
#[doc = "Reader of field `FACT5`"]
pub type FACT5_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT5`"]
pub struct FACT5_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT5_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u32) & 0x01) << 5);
        self.w
    }
}
#[doc = "Reader of field `FACT6`"]
pub type FACT6_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT6`"]
pub struct FACT6_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT6_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 6)) | (((value as u32) & 0x01) << 6);
        self.w
    }
}
#[doc = "Reader of field `FACT7`"]
pub type FACT7_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT7`"]
pub struct FACT7_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT7_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 7)) | (((value as u32) & 0x01) << 7);
        self.w
    }
}
#[doc = "Reader of field `FACT8`"]
pub type FACT8_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT8`"]
pub struct FACT8_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT8_W<'a> {
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
#[doc = "Reader of field `FACT9`"]
pub type FACT9_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT9`"]
pub struct FACT9_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT9_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 9)) | (((value as u32) & 0x01) << 9);
        self.w
    }
}
#[doc = "Reader of field `FACT10`"]
pub type FACT10_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT10`"]
pub struct FACT10_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT10_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 10)) | (((value as u32) & 0x01) << 10);
        self.w
    }
}
#[doc = "Reader of field `FACT11`"]
pub type FACT11_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT11`"]
pub struct FACT11_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT11_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 11)) | (((value as u32) & 0x01) << 11);
        self.w
    }
}
#[doc = "Reader of field `FACT12`"]
pub type FACT12_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT12`"]
pub struct FACT12_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT12_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 12)) | (((value as u32) & 0x01) << 12);
        self.w
    }
}
#[doc = "Reader of field `FACT13`"]
pub type FACT13_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FACT13`"]
pub struct FACT13_W<'a> {
    w: &'a mut W,
}
impl<'a> FACT13_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 13)) | (((value as u32) & 0x01) << 13);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Filter active"]
    #[inline(always)]
    pub fn fact0(&self) -> FACT0_R {
        FACT0_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Filter active"]
    #[inline(always)]
    pub fn fact1(&self) -> FACT1_R {
        FACT1_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Filter active"]
    #[inline(always)]
    pub fn fact2(&self) -> FACT2_R {
        FACT2_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Filter active"]
    #[inline(always)]
    pub fn fact3(&self) -> FACT3_R {
        FACT3_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Filter active"]
    #[inline(always)]
    pub fn fact4(&self) -> FACT4_R {
        FACT4_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Filter active"]
    #[inline(always)]
    pub fn fact5(&self) -> FACT5_R {
        FACT5_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Filter active"]
    #[inline(always)]
    pub fn fact6(&self) -> FACT6_R {
        FACT6_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Filter active"]
    #[inline(always)]
    pub fn fact7(&self) -> FACT7_R {
        FACT7_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Filter active"]
    #[inline(always)]
    pub fn fact8(&self) -> FACT8_R {
        FACT8_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Filter active"]
    #[inline(always)]
    pub fn fact9(&self) -> FACT9_R {
        FACT9_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Filter active"]
    #[inline(always)]
    pub fn fact10(&self) -> FACT10_R {
        FACT10_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - Filter active"]
    #[inline(always)]
    pub fn fact11(&self) -> FACT11_R {
        FACT11_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Filter active"]
    #[inline(always)]
    pub fn fact12(&self) -> FACT12_R {
        FACT12_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Filter active"]
    #[inline(always)]
    pub fn fact13(&self) -> FACT13_R {
        FACT13_R::new(((self.bits >> 13) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Filter active"]
    #[inline(always)]
    pub fn fact0(&mut self) -> FACT0_W {
        FACT0_W { w: self }
    }
    #[doc = "Bit 1 - Filter active"]
    #[inline(always)]
    pub fn fact1(&mut self) -> FACT1_W {
        FACT1_W { w: self }
    }
    #[doc = "Bit 2 - Filter active"]
    #[inline(always)]
    pub fn fact2(&mut self) -> FACT2_W {
        FACT2_W { w: self }
    }
    #[doc = "Bit 3 - Filter active"]
    #[inline(always)]
    pub fn fact3(&mut self) -> FACT3_W {
        FACT3_W { w: self }
    }
    #[doc = "Bit 4 - Filter active"]
    #[inline(always)]
    pub fn fact4(&mut self) -> FACT4_W {
        FACT4_W { w: self }
    }
    #[doc = "Bit 5 - Filter active"]
    #[inline(always)]
    pub fn fact5(&mut self) -> FACT5_W {
        FACT5_W { w: self }
    }
    #[doc = "Bit 6 - Filter active"]
    #[inline(always)]
    pub fn fact6(&mut self) -> FACT6_W {
        FACT6_W { w: self }
    }
    #[doc = "Bit 7 - Filter active"]
    #[inline(always)]
    pub fn fact7(&mut self) -> FACT7_W {
        FACT7_W { w: self }
    }
    #[doc = "Bit 8 - Filter active"]
    #[inline(always)]
    pub fn fact8(&mut self) -> FACT8_W {
        FACT8_W { w: self }
    }
    #[doc = "Bit 9 - Filter active"]
    #[inline(always)]
    pub fn fact9(&mut self) -> FACT9_W {
        FACT9_W { w: self }
    }
    #[doc = "Bit 10 - Filter active"]
    #[inline(always)]
    pub fn fact10(&mut self) -> FACT10_W {
        FACT10_W { w: self }
    }
    #[doc = "Bit 11 - Filter active"]
    #[inline(always)]
    pub fn fact11(&mut self) -> FACT11_W {
        FACT11_W { w: self }
    }
    #[doc = "Bit 12 - Filter active"]
    #[inline(always)]
    pub fn fact12(&mut self) -> FACT12_W {
        FACT12_W { w: self }
    }
    #[doc = "Bit 13 - Filter active"]
    #[inline(always)]
    pub fn fact13(&mut self) -> FACT13_W {
        FACT13_W { w: self }
    }
}
