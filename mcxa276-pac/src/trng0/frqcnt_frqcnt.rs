#[doc = "Register `FRQCNT` reader"]
pub type R = crate::R<FrqcntFrqcntSpec>;
#[doc = "Field `FRQ_CT` reader - Frequency Count"]
pub type FrqCtR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:21 - Frequency Count"]
    #[inline(always)]
    pub fn frq_ct(&self) -> FrqCtR {
        FrqCtR::new(self.bits & 0x003f_ffff)
    }
}
#[doc = "Frequency Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`frqcnt_frqcnt::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FrqcntFrqcntSpec;
impl crate::RegisterSpec for FrqcntFrqcntSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`frqcnt_frqcnt::R`](R) reader structure"]
impl crate::Readable for FrqcntFrqcntSpec {}
#[doc = "`reset()` method sets FRQCNT to value 0"]
impl crate::Resettable for FrqcntFrqcntSpec {}
