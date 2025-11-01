#[doc = "Register `SRDROR` reader"]
pub type R = crate::R<SrdrorSpec>;
#[doc = "Field `DATA` reader - Receive Data"]
pub type DataR = crate::FieldReader;
#[doc = "Field `RADDR` reader - Received Address"]
pub type RaddrR = crate::FieldReader;
#[doc = "Receive Empty\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxempty {
    #[doc = "0: Not empty"]
    NotEmpty = 0,
    #[doc = "1: Empty"]
    Empty = 1,
}
impl From<Rxempty> for bool {
    #[inline(always)]
    fn from(variant: Rxempty) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXEMPTY` reader - Receive Empty"]
pub type RxemptyR = crate::BitReader<Rxempty>;
impl RxemptyR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxempty {
        match self.bits {
            false => Rxempty::NotEmpty,
            true => Rxempty::Empty,
        }
    }
    #[doc = "Not empty"]
    #[inline(always)]
    pub fn is_not_empty(&self) -> bool {
        *self == Rxempty::NotEmpty
    }
    #[doc = "Empty"]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        *self == Rxempty::Empty
    }
}
#[doc = "Start of Frame\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sof {
    #[doc = "0: Not the first"]
    NotFirstDataWord = 0,
    #[doc = "1: First"]
    FirstDataWord = 1,
}
impl From<Sof> for bool {
    #[inline(always)]
    fn from(variant: Sof) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOF` reader - Start of Frame"]
pub type SofR = crate::BitReader<Sof>;
impl SofR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sof {
        match self.bits {
            false => Sof::NotFirstDataWord,
            true => Sof::FirstDataWord,
        }
    }
    #[doc = "Not the first"]
    #[inline(always)]
    pub fn is_not_first_data_word(&self) -> bool {
        *self == Sof::NotFirstDataWord
    }
    #[doc = "First"]
    #[inline(always)]
    pub fn is_first_data_word(&self) -> bool {
        *self == Sof::FirstDataWord
    }
}
impl R {
    #[doc = "Bits 0:7 - Receive Data"]
    #[inline(always)]
    pub fn data(&self) -> DataR {
        DataR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:10 - Received Address"]
    #[inline(always)]
    pub fn raddr(&self) -> RaddrR {
        RaddrR::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bit 14 - Receive Empty"]
    #[inline(always)]
    pub fn rxempty(&self) -> RxemptyR {
        RxemptyR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Start of Frame"]
    #[inline(always)]
    pub fn sof(&self) -> SofR {
        SofR::new(((self.bits >> 15) & 1) != 0)
    }
}
#[doc = "Target Receive Data Read Only\n\nYou can [`read`](crate::Reg::read) this register and get [`srdror::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SrdrorSpec;
impl crate::RegisterSpec for SrdrorSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`srdror::R`](R) reader structure"]
impl crate::Readable for SrdrorSpec {}
#[doc = "`reset()` method sets SRDROR to value 0x4000"]
impl crate::Resettable for SrdrorSpec {
    const RESET_VALUE: u32 = 0x4000;
}
