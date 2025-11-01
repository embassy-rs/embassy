#[doc = "Register `PERID` reader"]
pub type R = crate::R<PeridSpec>;
#[doc = "Field `ID` reader - Peripheral Identification"]
pub type IdR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:5 - Peripheral Identification"]
    #[inline(always)]
    pub fn id(&self) -> IdR {
        IdR::new(self.bits & 0x3f)
    }
}
#[doc = "Peripheral ID\n\nYou can [`read`](crate::Reg::read) this register and get [`perid::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PeridSpec;
impl crate::RegisterSpec for PeridSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`perid::R`](R) reader structure"]
impl crate::Readable for PeridSpec {}
#[doc = "`reset()` method sets PERID to value 0x04"]
impl crate::Resettable for PeridSpec {
    const RESET_VALUE: u8 = 0x04;
}
