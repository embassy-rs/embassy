#[doc = "Register `PKRSQ` reader"]
pub type R = crate::R<PkrsqPkrsqSpec>;
#[doc = "Field `PKR_SQ` reader - Poker Square Calculation Result."]
pub type PkrSqR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:23 - Poker Square Calculation Result."]
    #[inline(always)]
    pub fn pkr_sq(&self) -> PkrSqR {
        PkrSqR::new(self.bits & 0x00ff_ffff)
    }
}
#[doc = "Poker Square Calculation Result Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrsq_pkrsq::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkrsqPkrsqSpec;
impl crate::RegisterSpec for PkrsqPkrsqSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkrsq_pkrsq::R`](R) reader structure"]
impl crate::Readable for PkrsqPkrsqSpec {}
#[doc = "`reset()` method sets PKRSQ to value 0"]
impl crate::Resettable for PkrsqPkrsqSpec {}
