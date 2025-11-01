#[doc = "Register `SM1CVAL0` reader"]
pub type R = crate::R<Sm1cval0Spec>;
#[doc = "Field `CAPTVAL0` reader - Capture Value 0"]
pub type Captval0R = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Capture Value 0"]
    #[inline(always)]
    pub fn captval0(&self) -> Captval0R {
        Captval0R::new(self.bits)
    }
}
#[doc = "Capture Value 0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1cval0::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm1cval0Spec;
impl crate::RegisterSpec for Sm1cval0Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm1cval0::R`](R) reader structure"]
impl crate::Readable for Sm1cval0Spec {}
#[doc = "`reset()` method sets SM1CVAL0 to value 0"]
impl crate::Resettable for Sm1cval0Spec {}
