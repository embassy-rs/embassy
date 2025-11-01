#[doc = "Register `MRDATAH` reader"]
pub type R = crate::R<MrdatahSpec>;
#[doc = "Field `LSB` reader - Low Byte"]
pub type LsbR = crate::FieldReader;
#[doc = "Field `MSB` reader - High Byte"]
pub type MsbR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:7 - Low Byte"]
    #[inline(always)]
    pub fn lsb(&self) -> LsbR {
        LsbR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - High Byte"]
    #[inline(always)]
    pub fn msb(&self) -> MsbR {
        MsbR::new(((self.bits >> 8) & 0xff) as u8)
    }
}
#[doc = "Controller Read Data Halfword\n\nYou can [`read`](crate::Reg::read) this register and get [`mrdatah::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrdatahSpec;
impl crate::RegisterSpec for MrdatahSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrdatah::R`](R) reader structure"]
impl crate::Readable for MrdatahSpec {}
#[doc = "`reset()` method sets MRDATAH to value 0"]
impl crate::Resettable for MrdatahSpec {}
