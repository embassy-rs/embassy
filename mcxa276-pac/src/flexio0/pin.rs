#[doc = "Register `PIN` reader"]
pub type R = crate::R<PinSpec>;
#[doc = "Field `PDI` reader - Pin Data Input"]
pub type PdiR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - Pin Data Input"]
    #[inline(always)]
    pub fn pdi(&self) -> PdiR {
        PdiR::new(self.bits)
    }
}
#[doc = "Pin State\n\nYou can [`read`](crate::Reg::read) this register and get [`pin::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PinSpec;
impl crate::RegisterSpec for PinSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pin::R`](R) reader structure"]
impl crate::Readable for PinSpec {}
#[doc = "`reset()` method sets PIN to value 0"]
impl crate::Resettable for PinSpec {}
