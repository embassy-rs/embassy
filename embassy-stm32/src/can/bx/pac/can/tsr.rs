#[doc = "Reader of register TSR"]
pub type R = crate::R<u32, super::TSR>;
#[doc = "Writer for register TSR"]
pub type W = crate::W<u32, super::TSR>;
#[doc = "Register TSR `reset()`'s with value 0"]
impl crate::ResetValue for super::TSR {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `LOW2`"]
pub type LOW2_R = crate::R<bool, bool>;
#[doc = "Reader of field `LOW1`"]
pub type LOW1_R = crate::R<bool, bool>;
#[doc = "Reader of field `LOW0`"]
pub type LOW0_R = crate::R<bool, bool>;
#[doc = "Reader of field `TME2`"]
pub type TME2_R = crate::R<bool, bool>;
#[doc = "Reader of field `TME1`"]
pub type TME1_R = crate::R<bool, bool>;
#[doc = "Reader of field `TME0`"]
pub type TME0_R = crate::R<bool, bool>;
#[doc = "Reader of field `CODE`"]
pub type CODE_R = crate::R<u8, u8>;
#[doc = "Reader of field `ABRQ2`"]
pub type ABRQ2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ABRQ2`"]
pub struct ABRQ2_W<'a> {
    w: &'a mut W,
}
impl<'a> ABRQ2_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 23)) | (((value as u32) & 0x01) << 23);
        self.w
    }
}
#[doc = "Reader of field `TERR2`"]
pub type TERR2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TERR2`"]
pub struct TERR2_W<'a> {
    w: &'a mut W,
}
impl<'a> TERR2_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 19)) | (((value as u32) & 0x01) << 19);
        self.w
    }
}
#[doc = "Reader of field `ALST2`"]
pub type ALST2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ALST2`"]
pub struct ALST2_W<'a> {
    w: &'a mut W,
}
impl<'a> ALST2_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 18)) | (((value as u32) & 0x01) << 18);
        self.w
    }
}
#[doc = "Reader of field `TXOK2`"]
pub type TXOK2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TXOK2`"]
pub struct TXOK2_W<'a> {
    w: &'a mut W,
}
impl<'a> TXOK2_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 17)) | (((value as u32) & 0x01) << 17);
        self.w
    }
}
#[doc = "Reader of field `RQCP2`"]
pub type RQCP2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RQCP2`"]
pub struct RQCP2_W<'a> {
    w: &'a mut W,
}
impl<'a> RQCP2_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 16)) | (((value as u32) & 0x01) << 16);
        self.w
    }
}
#[doc = "Reader of field `ABRQ1`"]
pub type ABRQ1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ABRQ1`"]
pub struct ABRQ1_W<'a> {
    w: &'a mut W,
}
impl<'a> ABRQ1_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 15)) | (((value as u32) & 0x01) << 15);
        self.w
    }
}
#[doc = "Reader of field `TERR1`"]
pub type TERR1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TERR1`"]
pub struct TERR1_W<'a> {
    w: &'a mut W,
}
impl<'a> TERR1_W<'a> {
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
#[doc = "Reader of field `ALST1`"]
pub type ALST1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ALST1`"]
pub struct ALST1_W<'a> {
    w: &'a mut W,
}
impl<'a> ALST1_W<'a> {
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
#[doc = "Reader of field `TXOK1`"]
pub type TXOK1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TXOK1`"]
pub struct TXOK1_W<'a> {
    w: &'a mut W,
}
impl<'a> TXOK1_W<'a> {
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
#[doc = "Reader of field `RQCP1`"]
pub type RQCP1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RQCP1`"]
pub struct RQCP1_W<'a> {
    w: &'a mut W,
}
impl<'a> RQCP1_W<'a> {
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
#[doc = "Reader of field `ABRQ0`"]
pub type ABRQ0_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ABRQ0`"]
pub struct ABRQ0_W<'a> {
    w: &'a mut W,
}
impl<'a> ABRQ0_W<'a> {
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
#[doc = "Reader of field `TERR0`"]
pub type TERR0_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TERR0`"]
pub struct TERR0_W<'a> {
    w: &'a mut W,
}
impl<'a> TERR0_W<'a> {
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
#[doc = "Reader of field `ALST0`"]
pub type ALST0_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ALST0`"]
pub struct ALST0_W<'a> {
    w: &'a mut W,
}
impl<'a> ALST0_W<'a> {
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
#[doc = "Reader of field `TXOK0`"]
pub type TXOK0_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TXOK0`"]
pub struct TXOK0_W<'a> {
    w: &'a mut W,
}
impl<'a> TXOK0_W<'a> {
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
#[doc = "Reader of field `RQCP0`"]
pub type RQCP0_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RQCP0`"]
pub struct RQCP0_W<'a> {
    w: &'a mut W,
}
impl<'a> RQCP0_W<'a> {
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
    #[doc = "Bit 31 - Lowest priority flag for mailbox 2"]
    #[inline(always)]
    pub fn low2(&self) -> LOW2_R {
        LOW2_R::new(((self.bits >> 31) & 0x01) != 0)
    }
    #[doc = "Bit 30 - Lowest priority flag for mailbox 1"]
    #[inline(always)]
    pub fn low1(&self) -> LOW1_R {
        LOW1_R::new(((self.bits >> 30) & 0x01) != 0)
    }
    #[doc = "Bit 29 - Lowest priority flag for mailbox 0"]
    #[inline(always)]
    pub fn low0(&self) -> LOW0_R {
        LOW0_R::new(((self.bits >> 29) & 0x01) != 0)
    }
    #[doc = "Bit 28 - Lowest priority flag for mailbox 2"]
    #[inline(always)]
    pub fn tme2(&self) -> TME2_R {
        TME2_R::new(((self.bits >> 28) & 0x01) != 0)
    }
    #[doc = "Bit 27 - Lowest priority flag for mailbox 1"]
    #[inline(always)]
    pub fn tme1(&self) -> TME1_R {
        TME1_R::new(((self.bits >> 27) & 0x01) != 0)
    }
    #[doc = "Bit 26 - Lowest priority flag for mailbox 0"]
    #[inline(always)]
    pub fn tme0(&self) -> TME0_R {
        TME0_R::new(((self.bits >> 26) & 0x01) != 0)
    }
    #[doc = "Bits 24:25 - CODE"]
    #[inline(always)]
    pub fn code(&self) -> CODE_R {
        CODE_R::new(((self.bits >> 24) & 0x03) as u8)
    }
    #[doc = "Bit 23 - ABRQ2"]
    #[inline(always)]
    pub fn abrq2(&self) -> ABRQ2_R {
        ABRQ2_R::new(((self.bits >> 23) & 0x01) != 0)
    }
    #[doc = "Bit 19 - TERR2"]
    #[inline(always)]
    pub fn terr2(&self) -> TERR2_R {
        TERR2_R::new(((self.bits >> 19) & 0x01) != 0)
    }
    #[doc = "Bit 18 - ALST2"]
    #[inline(always)]
    pub fn alst2(&self) -> ALST2_R {
        ALST2_R::new(((self.bits >> 18) & 0x01) != 0)
    }
    #[doc = "Bit 17 - TXOK2"]
    #[inline(always)]
    pub fn txok2(&self) -> TXOK2_R {
        TXOK2_R::new(((self.bits >> 17) & 0x01) != 0)
    }
    #[doc = "Bit 16 - RQCP2"]
    #[inline(always)]
    pub fn rqcp2(&self) -> RQCP2_R {
        RQCP2_R::new(((self.bits >> 16) & 0x01) != 0)
    }
    #[doc = "Bit 15 - ABRQ1"]
    #[inline(always)]
    pub fn abrq1(&self) -> ABRQ1_R {
        ABRQ1_R::new(((self.bits >> 15) & 0x01) != 0)
    }
    #[doc = "Bit 11 - TERR1"]
    #[inline(always)]
    pub fn terr1(&self) -> TERR1_R {
        TERR1_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 10 - ALST1"]
    #[inline(always)]
    pub fn alst1(&self) -> ALST1_R {
        ALST1_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 9 - TXOK1"]
    #[inline(always)]
    pub fn txok1(&self) -> TXOK1_R {
        TXOK1_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 8 - RQCP1"]
    #[inline(always)]
    pub fn rqcp1(&self) -> RQCP1_R {
        RQCP1_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 7 - ABRQ0"]
    #[inline(always)]
    pub fn abrq0(&self) -> ABRQ0_R {
        ABRQ0_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 3 - TERR0"]
    #[inline(always)]
    pub fn terr0(&self) -> TERR0_R {
        TERR0_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 2 - ALST0"]
    #[inline(always)]
    pub fn alst0(&self) -> ALST0_R {
        ALST0_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 1 - TXOK0"]
    #[inline(always)]
    pub fn txok0(&self) -> TXOK0_R {
        TXOK0_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 0 - RQCP0"]
    #[inline(always)]
    pub fn rqcp0(&self) -> RQCP0_R {
        RQCP0_R::new((self.bits & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 23 - ABRQ2"]
    #[inline(always)]
    pub fn abrq2(&mut self) -> ABRQ2_W {
        ABRQ2_W { w: self }
    }
    #[doc = "Bit 19 - TERR2"]
    #[inline(always)]
    pub fn terr2(&mut self) -> TERR2_W {
        TERR2_W { w: self }
    }
    #[doc = "Bit 18 - ALST2"]
    #[inline(always)]
    pub fn alst2(&mut self) -> ALST2_W {
        ALST2_W { w: self }
    }
    #[doc = "Bit 17 - TXOK2"]
    #[inline(always)]
    pub fn txok2(&mut self) -> TXOK2_W {
        TXOK2_W { w: self }
    }
    #[doc = "Bit 16 - RQCP2"]
    #[inline(always)]
    pub fn rqcp2(&mut self) -> RQCP2_W {
        RQCP2_W { w: self }
    }
    #[doc = "Bit 15 - ABRQ1"]
    #[inline(always)]
    pub fn abrq1(&mut self) -> ABRQ1_W {
        ABRQ1_W { w: self }
    }
    #[doc = "Bit 11 - TERR1"]
    #[inline(always)]
    pub fn terr1(&mut self) -> TERR1_W {
        TERR1_W { w: self }
    }
    #[doc = "Bit 10 - ALST1"]
    #[inline(always)]
    pub fn alst1(&mut self) -> ALST1_W {
        ALST1_W { w: self }
    }
    #[doc = "Bit 9 - TXOK1"]
    #[inline(always)]
    pub fn txok1(&mut self) -> TXOK1_W {
        TXOK1_W { w: self }
    }
    #[doc = "Bit 8 - RQCP1"]
    #[inline(always)]
    pub fn rqcp1(&mut self) -> RQCP1_W {
        RQCP1_W { w: self }
    }
    #[doc = "Bit 7 - ABRQ0"]
    #[inline(always)]
    pub fn abrq0(&mut self) -> ABRQ0_W {
        ABRQ0_W { w: self }
    }
    #[doc = "Bit 3 - TERR0"]
    #[inline(always)]
    pub fn terr0(&mut self) -> TERR0_W {
        TERR0_W { w: self }
    }
    #[doc = "Bit 2 - ALST0"]
    #[inline(always)]
    pub fn alst0(&mut self) -> ALST0_W {
        ALST0_W { w: self }
    }
    #[doc = "Bit 1 - TXOK0"]
    #[inline(always)]
    pub fn txok0(&mut self) -> TXOK0_W {
        TXOK0_W { w: self }
    }
    #[doc = "Bit 0 - RQCP0"]
    #[inline(always)]
    pub fn rqcp0(&mut self) -> RQCP0_W {
        RQCP0_W { w: self }
    }
}
