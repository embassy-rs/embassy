#[doc = "Register `LASTEDGEH` reader"]
pub type R = crate::R<LastedgehSpec>;
#[doc = "Field `LASTEDGEH` reader - Last Edge Time Hold"]
pub type LastedgehR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Last Edge Time Hold"]
    #[inline(always)]
    pub fn lastedgeh(&self) -> LastedgehR {
        LastedgehR::new(self.bits)
    }
}
#[doc = "Last Edge Time Hold Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lastedgeh::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LastedgehSpec;
impl crate::RegisterSpec for LastedgehSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`lastedgeh::R`](R) reader structure"]
impl crate::Readable for LastedgehSpec {}
#[doc = "`reset()` method sets LASTEDGEH to value 0xffff"]
impl crate::Resettable for LastedgehSpec {
    const RESET_VALUE: u16 = 0xffff;
}
