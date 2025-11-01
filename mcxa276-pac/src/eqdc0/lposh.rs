#[doc = "Register `LPOSH` reader"]
pub type R = crate::R<LposhSpec>;
#[doc = "Field `LPOSH` reader - POSH"]
pub type LposhR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - POSH"]
    #[inline(always)]
    pub fn lposh(&self) -> LposhR {
        LposhR::new(self.bits)
    }
}
#[doc = "Lower Position Hold Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lposh::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LposhSpec;
impl crate::RegisterSpec for LposhSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`lposh::R`](R) reader structure"]
impl crate::Readable for LposhSpec {}
#[doc = "`reset()` method sets LPOSH to value 0"]
impl crate::Resettable for LposhSpec {}
