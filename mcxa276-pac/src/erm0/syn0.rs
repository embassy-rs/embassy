#[doc = "Register `SYN0` reader"]
pub type R = crate::R<Syn0Spec>;
#[doc = "Field `SYNDROME` reader - SYNDROME"]
pub type SyndromeR = crate::FieldReader;
impl R {
    #[doc = "Bits 24:31 - SYNDROME"]
    #[inline(always)]
    pub fn syndrome(&self) -> SyndromeR {
        SyndromeR::new(((self.bits >> 24) & 0xff) as u8)
    }
}
#[doc = "ERM Memory 0 Syndrome Register\n\nYou can [`read`](crate::Reg::read) this register and get [`syn0::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Syn0Spec;
impl crate::RegisterSpec for Syn0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`syn0::R`](R) reader structure"]
impl crate::Readable for Syn0Spec {}
#[doc = "`reset()` method sets SYN0 to value 0"]
impl crate::Resettable for Syn0Spec {}
