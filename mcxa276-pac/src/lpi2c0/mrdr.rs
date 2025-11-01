#[doc = "Register `MRDR` reader"]
pub type R = crate::R<MrdrSpec>;
#[doc = "Field `DATA` reader - Receive Data"]
pub type DataR = crate::FieldReader;
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
impl R {
    #[doc = "Bits 0:7 - Receive Data"]
    #[inline(always)]
    pub fn data(&self) -> DataR {
        DataR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bit 14 - Receive Empty"]
    #[inline(always)]
    pub fn rxempty(&self) -> RxemptyR {
        RxemptyR::new(((self.bits >> 14) & 1) != 0)
    }
}
#[doc = "Controller Receive Data\n\nYou can [`read`](crate::Reg::read) this register and get [`mrdr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrdrSpec;
impl crate::RegisterSpec for MrdrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrdr::R`](R) reader structure"]
impl crate::Readable for MrdrSpec {}
#[doc = "`reset()` method sets MRDR to value 0x4000"]
impl crate::Resettable for MrdrSpec {
    const RESET_VALUE: u32 = 0x4000;
}
