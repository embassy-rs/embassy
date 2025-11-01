#[doc = "Register `SM0CNT` reader"]
pub type R = crate::R<Sm0cntSpec>;
#[doc = "Field `CNT` reader - Counter Register Bits"]
pub type CntR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Counter Register Bits"]
    #[inline(always)]
    pub fn cnt(&self) -> CntR {
        CntR::new(self.bits)
    }
}
#[doc = "Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0cnt::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm0cntSpec;
impl crate::RegisterSpec for Sm0cntSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm0cnt::R`](R) reader structure"]
impl crate::Readable for Sm0cntSpec {}
#[doc = "`reset()` method sets SM0CNT to value 0"]
impl crate::Resettable for Sm0cntSpec {}
