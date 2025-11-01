#[doc = "Register `SM2CVAL0CYC` reader"]
pub type R = crate::R<Sm2cval0cycSpec>;
#[doc = "Field `CVAL0CYC` reader - Capture Value 0 Cycle"]
pub type Cval0cycR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:3 - Capture Value 0 Cycle"]
    #[inline(always)]
    pub fn cval0cyc(&self) -> Cval0cycR {
        Cval0cycR::new((self.bits & 0x0f) as u8)
    }
}
#[doc = "Capture Value 0 Cycle Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2cval0cyc::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2cval0cycSpec;
impl crate::RegisterSpec for Sm2cval0cycSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2cval0cyc::R`](R) reader structure"]
impl crate::Readable for Sm2cval0cycSpec {}
#[doc = "`reset()` method sets SM2CVAL0CYC to value 0"]
impl crate::Resettable for Sm2cval0cycSpec {}
