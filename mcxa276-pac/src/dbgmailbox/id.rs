#[doc = "Register `ID` reader"]
pub type R = crate::R<IdSpec>;
#[doc = "Field `ID` reader - Identification Value"]
pub type IdR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - Identification Value"]
    #[inline(always)]
    pub fn id(&self) -> IdR {
        IdR::new(self.bits)
    }
}
#[doc = "Identification\n\nYou can [`read`](crate::Reg::read) this register and get [`id::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IdSpec;
impl crate::RegisterSpec for IdSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`id::R`](R) reader structure"]
impl crate::Readable for IdSpec {}
#[doc = "`reset()` method sets ID to value 0x002a_0000"]
impl crate::Resettable for IdSpec {
    const RESET_VALUE: u32 = 0x002a_0000;
}
