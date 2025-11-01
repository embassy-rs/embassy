#[doc = "Register `UPOSH` reader"]
pub type R = crate::R<UposhSpec>;
#[doc = "Field `POSH` reader - POSH"]
pub type PoshR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - POSH"]
    #[inline(always)]
    pub fn posh(&self) -> PoshR {
        PoshR::new(self.bits)
    }
}
#[doc = "Upper Position Hold Register\n\nYou can [`read`](crate::Reg::read) this register and get [`uposh::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct UposhSpec;
impl crate::RegisterSpec for UposhSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`uposh::R`](R) reader structure"]
impl crate::Readable for UposhSpec {}
#[doc = "`reset()` method sets UPOSH to value 0"]
impl crate::Resettable for UposhSpec {}
