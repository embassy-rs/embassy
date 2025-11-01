#[doc = "Register `udf_rd_data` reader"]
pub type R = crate::R<UdfRdDataSpec>;
#[doc = "Field `o_dat` reader - no description available"]
pub type ODatR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - no description available"]
    #[inline(always)]
    pub fn o_dat(&self) -> ODatR {
        ODatR::new(self.bits)
    }
}
#[doc = "Data Out Register\n\nYou can [`read`](crate::Reg::read) this register and get [`udf_rd_data::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct UdfRdDataSpec;
impl crate::RegisterSpec for UdfRdDataSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`udf_rd_data::R`](R) reader structure"]
impl crate::Readable for UdfRdDataSpec {}
#[doc = "`reset()` method sets udf_rd_data to value 0"]
impl crate::Resettable for UdfRdDataSpec {}
