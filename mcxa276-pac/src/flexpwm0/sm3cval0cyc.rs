#[doc = "Register `SM3CVAL0CYC` reader"]
pub type R = crate::R<Sm3cval0cycSpec>;
#[doc = "Field `CVAL0CYC` reader - Capture Value 0 Cycle"]
pub type Cval0cycR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:3 - Capture Value 0 Cycle"]
    #[inline(always)]
    pub fn cval0cyc(&self) -> Cval0cycR {
        Cval0cycR::new((self.bits & 0x0f) as u8)
    }
}
#[doc = "Capture Value 0 Cycle Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3cval0cyc::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm3cval0cycSpec;
impl crate::RegisterSpec for Sm3cval0cycSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm3cval0cyc::R`](R) reader structure"]
impl crate::Readable for Sm3cval0cycSpec {}
#[doc = "`reset()` method sets SM3CVAL0CYC to value 0"]
impl crate::Resettable for Sm3cval0cycSpec {}
