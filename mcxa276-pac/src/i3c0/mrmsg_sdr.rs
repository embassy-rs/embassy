#[doc = "Register `MRMSG_SDR` reader"]
pub type R = crate::R<MrmsgSdrSpec>;
#[doc = "Field `DATA` reader - Data"]
pub type DataR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Data"]
    #[inline(always)]
    pub fn data(&self) -> DataR {
        DataR::new((self.bits & 0xffff) as u16)
    }
}
#[doc = "Controller Read Message in SDR mode\n\nYou can [`read`](crate::Reg::read) this register and get [`mrmsg_sdr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrmsgSdrSpec;
impl crate::RegisterSpec for MrmsgSdrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrmsg_sdr::R`](R) reader structure"]
impl crate::Readable for MrmsgSdrSpec {}
#[doc = "`reset()` method sets MRMSG_SDR to value 0"]
impl crate::Resettable for MrmsgSdrSpec {}
