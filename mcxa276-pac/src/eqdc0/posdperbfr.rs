#[doc = "Register `POSDPERBFR` reader"]
pub type R = crate::R<PosdperbfrSpec>;
#[doc = "Field `POSDPERBFR` reader - Position difference period buffer"]
pub type PosdperbfrR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Position difference period buffer"]
    #[inline(always)]
    pub fn posdperbfr(&self) -> PosdperbfrR {
        PosdperbfrR::new(self.bits)
    }
}
#[doc = "Position Difference Period Buffer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`posdperbfr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PosdperbfrSpec;
impl crate::RegisterSpec for PosdperbfrSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`posdperbfr::R`](R) reader structure"]
impl crate::Readable for PosdperbfrSpec {}
#[doc = "`reset()` method sets POSDPERBFR to value 0xffff"]
impl crate::Resettable for PosdperbfrSpec {
    const RESET_VALUE: u16 = 0xffff;
}
