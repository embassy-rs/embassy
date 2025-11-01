#[doc = "Register `SP` reader"]
pub type R = crate::R<SpSpec>;
#[doc = "Field `SP` reader - Stack Pointer"]
pub type SpR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - Stack Pointer"]
    #[inline(always)]
    pub fn sp(&self) -> SpR {
        SpR::new(self.bits)
    }
}
#[doc = "Stack Pointer\n\nYou can [`read`](crate::Reg::read) this register and get [`sp::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SpSpec;
impl crate::RegisterSpec for SpSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sp::R`](R) reader structure"]
impl crate::Readable for SpSpec {}
#[doc = "`reset()` method sets SP to value 0"]
impl crate::Resettable for SpSpec {}
