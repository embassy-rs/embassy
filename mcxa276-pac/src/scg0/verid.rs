#[doc = "Register `VERID` reader"]
pub type R = crate::R<VeridSpec>;
#[doc = "Field `VERSION` reader - SCG Version Number"]
pub type VersionR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - SCG Version Number"]
    #[inline(always)]
    pub fn version(&self) -> VersionR {
        VersionR::new(self.bits)
    }
}
#[doc = "Version ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`verid::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct VeridSpec;
impl crate::RegisterSpec for VeridSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`verid::R`](R) reader structure"]
impl crate::Readable for VeridSpec {}
#[doc = "`reset()` method sets VERID to value 0"]
impl crate::Resettable for VeridSpec {}
