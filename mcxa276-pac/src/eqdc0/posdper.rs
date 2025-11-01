#[doc = "Register `POSDPER` reader"]
pub type R = crate::R<PosdperSpec>;
#[doc = "Field `POSDPER` reader - Position difference period"]
pub type PosdperR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Position difference period"]
    #[inline(always)]
    pub fn posdper(&self) -> PosdperR {
        PosdperR::new(self.bits)
    }
}
#[doc = "Position Difference Period Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`posdper::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PosdperSpec;
impl crate::RegisterSpec for PosdperSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`posdper::R`](R) reader structure"]
impl crate::Readable for PosdperSpec {}
#[doc = "`reset()` method sets POSDPER to value 0xffff"]
impl crate::Resettable for PosdperSpec {
    const RESET_VALUE: u16 = 0xffff;
}
