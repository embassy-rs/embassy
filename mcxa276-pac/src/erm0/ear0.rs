#[doc = "Register `EAR0` reader"]
pub type R = crate::R<Ear0Spec>;
#[doc = "Field `EAR` reader - EAR"]
pub type EarR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - EAR"]
    #[inline(always)]
    pub fn ear(&self) -> EarR {
        EarR::new(self.bits)
    }
}
#[doc = "ERM Memory 0 Error Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ear0::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ear0Spec;
impl crate::RegisterSpec for Ear0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ear0::R`](R) reader structure"]
impl crate::Readable for Ear0Spec {}
#[doc = "`reset()` method sets EAR0 to value 0"]
impl crate::Resettable for Ear0Spec {}
