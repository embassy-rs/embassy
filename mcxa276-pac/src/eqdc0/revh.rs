#[doc = "Register `REVH` reader"]
pub type R = crate::R<RevhSpec>;
#[doc = "Field `REVH` reader - REVH"]
pub type RevhR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - REVH"]
    #[inline(always)]
    pub fn revh(&self) -> RevhR {
        RevhR::new(self.bits)
    }
}
#[doc = "Revolution Hold Register\n\nYou can [`read`](crate::Reg::read) this register and get [`revh::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RevhSpec;
impl crate::RegisterSpec for RevhSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`revh::R`](R) reader structure"]
impl crate::Readable for RevhSpec {}
#[doc = "`reset()` method sets REVH to value 0"]
impl crate::Resettable for RevhSpec {}
