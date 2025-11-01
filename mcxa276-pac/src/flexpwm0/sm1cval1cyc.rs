#[doc = "Register `SM1CVAL1CYC` reader"]
pub type R = crate::R<Sm1cval1cycSpec>;
#[doc = "Field `CVAL1CYC` reader - Capture Value 1 Cycle"]
pub type Cval1cycR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:3 - Capture Value 1 Cycle"]
    #[inline(always)]
    pub fn cval1cyc(&self) -> Cval1cycR {
        Cval1cycR::new((self.bits & 0x0f) as u8)
    }
}
#[doc = "Capture Value 1 Cycle Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1cval1cyc::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm1cval1cycSpec;
impl crate::RegisterSpec for Sm1cval1cycSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm1cval1cyc::R`](R) reader structure"]
impl crate::Readable for Sm1cval1cycSpec {}
#[doc = "`reset()` method sets SM1CVAL1CYC to value 0"]
impl crate::Resettable for Sm1cval1cycSpec {}
