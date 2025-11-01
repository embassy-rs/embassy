#[doc = "Register `SM2CNT` reader"]
pub type R = crate::R<Sm2cntSpec>;
#[doc = "Field `CNT` reader - Counter Register Bits"]
pub type CntR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Counter Register Bits"]
    #[inline(always)]
    pub fn cnt(&self) -> CntR {
        CntR::new(self.bits)
    }
}
#[doc = "Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2cnt::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2cntSpec;
impl crate::RegisterSpec for Sm2cntSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2cnt::R`](R) reader structure"]
impl crate::Readable for Sm2cntSpec {}
#[doc = "`reset()` method sets SM2CNT to value 0"]
impl crate::Resettable for Sm2cntSpec {}
