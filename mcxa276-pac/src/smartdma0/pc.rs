#[doc = "Register `PC` reader"]
pub type R = crate::R<PcSpec>;
#[doc = "Field `PC` reader - Program Counter"]
pub type PcR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - Program Counter"]
    #[inline(always)]
    pub fn pc(&self) -> PcR {
        PcR::new(self.bits)
    }
}
#[doc = "Program Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`pc::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PcSpec;
impl crate::RegisterSpec for PcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pc::R`](R) reader structure"]
impl crate::Readable for PcSpec {}
#[doc = "`reset()` method sets PC to value 0"]
impl crate::Resettable for PcSpec {}
