#[doc = "Register `SASR` reader"]
pub type R = crate::R<SasrSpec>;
#[doc = "Field `RADDR` reader - Received Address"]
pub type RaddrR = crate::FieldReader<u16>;
#[doc = "Address Not Valid\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anv {
    #[doc = "0: Valid"]
    Valid = 0,
    #[doc = "1: Not valid"]
    NotValid = 1,
}
impl From<Anv> for bool {
    #[inline(always)]
    fn from(variant: Anv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ANV` reader - Address Not Valid"]
pub type AnvR = crate::BitReader<Anv>;
impl AnvR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Anv {
        match self.bits {
            false => Anv::Valid,
            true => Anv::NotValid,
        }
    }
    #[doc = "Valid"]
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        *self == Anv::Valid
    }
    #[doc = "Not valid"]
    #[inline(always)]
    pub fn is_not_valid(&self) -> bool {
        *self == Anv::NotValid
    }
}
impl R {
    #[doc = "Bits 0:10 - Received Address"]
    #[inline(always)]
    pub fn raddr(&self) -> RaddrR {
        RaddrR::new((self.bits & 0x07ff) as u16)
    }
    #[doc = "Bit 14 - Address Not Valid"]
    #[inline(always)]
    pub fn anv(&self) -> AnvR {
        AnvR::new(((self.bits >> 14) & 1) != 0)
    }
}
#[doc = "Target Address Status\n\nYou can [`read`](crate::Reg::read) this register and get [`sasr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SasrSpec;
impl crate::RegisterSpec for SasrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sasr::R`](R) reader structure"]
impl crate::Readable for SasrSpec {}
#[doc = "`reset()` method sets SASR to value 0x4000"]
impl crate::Resettable for SasrSpec {
    const RESET_VALUE: u32 = 0x4000;
}
