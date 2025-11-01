#[doc = "Register `TOTSAM` reader"]
pub type R = crate::R<TotsamTotsamSpec>;
#[doc = "Field `TOT_SAM` reader - Total Samples"]
pub type TotSamR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:19 - Total Samples"]
    #[inline(always)]
    pub fn tot_sam(&self) -> TotSamR {
        TotSamR::new(self.bits & 0x000f_ffff)
    }
}
#[doc = "Total Samples Register\n\nYou can [`read`](crate::Reg::read) this register and get [`totsam_totsam::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TotsamTotsamSpec;
impl crate::RegisterSpec for TotsamTotsamSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`totsam_totsam::R`](R) reader structure"]
impl crate::Readable for TotsamTotsamSpec {}
#[doc = "`reset()` method sets TOTSAM to value 0"]
impl crate::Resettable for TotsamTotsamSpec {}
