#[doc = "Register `udf_wr_data` writer"]
pub type W = crate::W<UdfWrDataSpec>;
#[doc = "Field `i_dat` writer - no description available"]
pub type IDatW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - no description available"]
    #[inline(always)]
    pub fn i_dat(&mut self) -> IDatW<UdfWrDataSpec> {
        IDatW::new(self, 0)
    }
}
#[doc = "Data In Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`udf_wr_data::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct UdfWrDataSpec;
impl crate::RegisterSpec for UdfWrDataSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`udf_wr_data::W`](W) writer structure"]
impl crate::Writable for UdfWrDataSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets udf_wr_data to value 0"]
impl crate::Resettable for UdfWrDataSpec {}
