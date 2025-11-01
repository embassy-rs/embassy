#[doc = "Register `ENT0` reader"]
pub type R = crate::R<Ent0Spec>;
#[doc = "Field `ENT` reader - Entropy Value"]
pub type EntR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - Entropy Value"]
    #[inline(always)]
    pub fn ent(&self) -> EntR {
        EntR::new(self.bits)
    }
}
#[doc = "Entropy Read Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ent0::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ent0Spec;
impl crate::RegisterSpec for Ent0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ent0::R`](R) reader structure"]
impl crate::Readable for Ent0Spec {}
#[doc = "`reset()` method sets ENT0 to value 0"]
impl crate::Resettable for Ent0Spec {}
