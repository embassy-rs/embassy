#[doc = "Register `POSDH` reader"]
pub type R = crate::R<PosdhSpec>;
#[doc = "Field `POSDH` reader - POSDH"]
pub type PosdhR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - POSDH"]
    #[inline(always)]
    pub fn posdh(&self) -> PosdhR {
        PosdhR::new(self.bits)
    }
}
#[doc = "Position Difference Hold Register\n\nYou can [`read`](crate::Reg::read) this register and get [`posdh::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PosdhSpec;
impl crate::RegisterSpec for PosdhSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`posdh::R`](R) reader structure"]
impl crate::Readable for PosdhSpec {}
#[doc = "`reset()` method sets POSDH to value 0"]
impl crate::Resettable for PosdhSpec {}
