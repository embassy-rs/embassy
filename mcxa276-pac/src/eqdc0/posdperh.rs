#[doc = "Register `POSDPERH` reader"]
pub type R = crate::R<PosdperhSpec>;
#[doc = "Field `POSDPERH` reader - Position difference period hold"]
pub type PosdperhR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Position difference period hold"]
    #[inline(always)]
    pub fn posdperh(&self) -> PosdperhR {
        PosdperhR::new(self.bits)
    }
}
#[doc = "Position Difference Period Hold Register\n\nYou can [`read`](crate::Reg::read) this register and get [`posdperh::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PosdperhSpec;
impl crate::RegisterSpec for PosdperhSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`posdperh::R`](R) reader structure"]
impl crate::Readable for PosdperhSpec {}
#[doc = "`reset()` method sets POSDPERH to value 0xffff"]
impl crate::Resettable for PosdperhSpec {
    const RESET_VALUE: u16 = 0xffff;
}
