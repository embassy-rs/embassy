#[doc = "Register `SM3CVAL1` reader"]
pub type R = crate::R<Sm3cval1Spec>;
#[doc = "Field `CAPTVAL1` reader - Capture Value 1"]
pub type Captval1R = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Capture Value 1"]
    #[inline(always)]
    pub fn captval1(&self) -> Captval1R {
        Captval1R::new(self.bits)
    }
}
#[doc = "Capture Value 1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3cval1::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm3cval1Spec;
impl crate::RegisterSpec for Sm3cval1Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm3cval1::R`](R) reader structure"]
impl crate::Readable for Sm3cval1Spec {}
#[doc = "`reset()` method sets SM3CVAL1 to value 0"]
impl crate::Resettable for Sm3cval1Spec {}
