#[doc = "Reader of register ESR"]
pub type R = crate::R<u32, super::ESR>;
#[doc = "Writer for register ESR"]
pub type W = crate::W<u32, super::ESR>;
#[doc = "Register ESR `reset()`'s with value 0"]
impl crate::ResetValue for super::ESR {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `REC`"]
pub type REC_R = crate::R<u8, u8>;
#[doc = "Reader of field `TEC`"]
pub type TEC_R = crate::R<u8, u8>;
#[doc = "LEC\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum LEC_A {
    #[doc = "0: No Error"]
    NOERROR = 0,
    #[doc = "1: Stuff Error"]
    STUFF = 1,
    #[doc = "2: Form Error"]
    FORM = 2,
    #[doc = "3: Acknowledgment Error"]
    ACK = 3,
    #[doc = "4: Bit recessive Error"]
    BITRECESSIVE = 4,
    #[doc = "5: Bit dominant Error"]
    BITDOMINANT = 5,
    #[doc = "6: CRC Error"]
    CRC = 6,
    #[doc = "7: Set by software"]
    CUSTOM = 7,
}
impl From<LEC_A> for u8 {
    #[inline(always)]
    fn from(variant: LEC_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `LEC`"]
pub type LEC_R = crate::R<u8, LEC_A>;
impl LEC_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> LEC_A {
        match self.bits {
            0 => LEC_A::NOERROR,
            1 => LEC_A::STUFF,
            2 => LEC_A::FORM,
            3 => LEC_A::ACK,
            4 => LEC_A::BITRECESSIVE,
            5 => LEC_A::BITDOMINANT,
            6 => LEC_A::CRC,
            7 => LEC_A::CUSTOM,
            _ => unreachable!(),
        }
    }
    #[doc = "Checks if the value of the field is `NOERROR`"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == LEC_A::NOERROR
    }
    #[doc = "Checks if the value of the field is `STUFF`"]
    #[inline(always)]
    pub fn is_stuff(&self) -> bool {
        *self == LEC_A::STUFF
    }
    #[doc = "Checks if the value of the field is `FORM`"]
    #[inline(always)]
    pub fn is_form(&self) -> bool {
        *self == LEC_A::FORM
    }
    #[doc = "Checks if the value of the field is `ACK`"]
    #[inline(always)]
    pub fn is_ack(&self) -> bool {
        *self == LEC_A::ACK
    }
    #[doc = "Checks if the value of the field is `BITRECESSIVE`"]
    #[inline(always)]
    pub fn is_bit_recessive(&self) -> bool {
        *self == LEC_A::BITRECESSIVE
    }
    #[doc = "Checks if the value of the field is `BITDOMINANT`"]
    #[inline(always)]
    pub fn is_bit_dominant(&self) -> bool {
        *self == LEC_A::BITDOMINANT
    }
    #[doc = "Checks if the value of the field is `CRC`"]
    #[inline(always)]
    pub fn is_crc(&self) -> bool {
        *self == LEC_A::CRC
    }
    #[doc = "Checks if the value of the field is `CUSTOM`"]
    #[inline(always)]
    pub fn is_custom(&self) -> bool {
        *self == LEC_A::CUSTOM
    }
}
#[doc = "Write proxy for field `LEC`"]
pub struct LEC_W<'a> {
    w: &'a mut W,
}
impl<'a> LEC_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: LEC_A) -> &'a mut W {
        {
            self.bits(variant.into())
        }
    }
    #[doc = "No Error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut W {
        self.variant(LEC_A::NOERROR)
    }
    #[doc = "Stuff Error"]
    #[inline(always)]
    pub fn stuff(self) -> &'a mut W {
        self.variant(LEC_A::STUFF)
    }
    #[doc = "Form Error"]
    #[inline(always)]
    pub fn form(self) -> &'a mut W {
        self.variant(LEC_A::FORM)
    }
    #[doc = "Acknowledgment Error"]
    #[inline(always)]
    pub fn ack(self) -> &'a mut W {
        self.variant(LEC_A::ACK)
    }
    #[doc = "Bit recessive Error"]
    #[inline(always)]
    pub fn bit_recessive(self) -> &'a mut W {
        self.variant(LEC_A::BITRECESSIVE)
    }
    #[doc = "Bit dominant Error"]
    #[inline(always)]
    pub fn bit_dominant(self) -> &'a mut W {
        self.variant(LEC_A::BITDOMINANT)
    }
    #[doc = "CRC Error"]
    #[inline(always)]
    pub fn crc(self) -> &'a mut W {
        self.variant(LEC_A::CRC)
    }
    #[doc = "Set by software"]
    #[inline(always)]
    pub fn custom(self) -> &'a mut W {
        self.variant(LEC_A::CUSTOM)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x07 << 4)) | (((value as u32) & 0x07) << 4);
        self.w
    }
}
#[doc = "Reader of field `BOFF`"]
pub type BOFF_R = crate::R<bool, bool>;
#[doc = "Reader of field `EPVF`"]
pub type EPVF_R = crate::R<bool, bool>;
#[doc = "Reader of field `EWGF`"]
pub type EWGF_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bits 24:31 - REC"]
    #[inline(always)]
    pub fn rec(&self) -> REC_R {
        REC_R::new(((self.bits >> 24) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - TEC"]
    #[inline(always)]
    pub fn tec(&self) -> TEC_R {
        TEC_R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 4:6 - LEC"]
    #[inline(always)]
    pub fn lec(&self) -> LEC_R {
        LEC_R::new(((self.bits >> 4) & 0x07) as u8)
    }
    #[doc = "Bit 2 - BOFF"]
    #[inline(always)]
    pub fn boff(&self) -> BOFF_R {
        BOFF_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 1 - EPVF"]
    #[inline(always)]
    pub fn epvf(&self) -> EPVF_R {
        EPVF_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 0 - EWGF"]
    #[inline(always)]
    pub fn ewgf(&self) -> EWGF_R {
        EWGF_R::new((self.bits & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 4:6 - LEC"]
    #[inline(always)]
    pub fn lec(&mut self) -> LEC_W {
        LEC_W { w: self }
    }
}
