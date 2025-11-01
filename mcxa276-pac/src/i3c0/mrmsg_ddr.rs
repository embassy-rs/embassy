#[doc = "Register `MRMSG_DDR` reader"]
pub type R = crate::R<MrmsgDdrSpec>;
#[doc = "Field `DATA` reader - Data"]
pub type DataR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Data"]
    #[inline(always)]
    pub fn data(&self) -> DataR {
        DataR::new((self.bits & 0xffff) as u16)
    }
}
#[doc = "Controller Read Message in DDR mode\n\nYou can [`read`](crate::Reg::read) this register and get [`mrmsg_ddr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrmsgDdrSpec;
impl crate::RegisterSpec for MrmsgDdrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrmsg_ddr::R`](R) reader structure"]
impl crate::Readable for MrmsgDdrSpec {}
#[doc = "`reset()` method sets MRMSG_DDR to value 0"]
impl crate::Resettable for MrmsgDdrSpec {}
