#[doc = "Register `RXFIR` reader"]
pub type R = crate::R<RxfirSpec>;
#[doc = "Field `IDHIT` reader - Identifier Acceptance Filter Hit Indicator"]
pub type IdhitR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:8 - Identifier Acceptance Filter Hit Indicator"]
    #[inline(always)]
    pub fn idhit(&self) -> IdhitR {
        IdhitR::new((self.bits & 0x01ff) as u16)
    }
}
#[doc = "Legacy RX FIFO Information\n\nYou can [`read`](crate::Reg::read) this register and get [`rxfir::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RxfirSpec;
impl crate::RegisterSpec for RxfirSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rxfir::R`](R) reader structure"]
impl crate::Readable for RxfirSpec {}
#[doc = "`reset()` method sets RXFIR to value 0"]
impl crate::Resettable for RxfirSpec {}
