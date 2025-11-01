#[doc = "Register `RDBR[%s]` reader"]
pub type R = crate::R<RdbrSpec>;
#[doc = "Field `DATA` reader - Data"]
pub type DataR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - Data"]
    #[inline(always)]
    pub fn data(&self) -> DataR {
        DataR::new(self.bits)
    }
}
#[doc = "Receive Data Burst\n\nYou can [`read`](crate::Reg::read) this register and get [`rdbr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RdbrSpec;
impl crate::RegisterSpec for RdbrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rdbr::R`](R) reader structure"]
impl crate::Readable for RdbrSpec {}
#[doc = "`reset()` method sets RDBR[%s] to value 0"]
impl crate::Resettable for RdbrSpec {}
