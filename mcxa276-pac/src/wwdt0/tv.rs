#[doc = "Register `TV` reader"]
pub type R = crate::R<TvSpec>;
#[doc = "Field `COUNT` reader - Counter Timer Value"]
pub type CountR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:23 - Counter Timer Value"]
    #[inline(always)]
    pub fn count(&self) -> CountR {
        CountR::new(self.bits & 0x00ff_ffff)
    }
}
#[doc = "Timer Value\n\nYou can [`read`](crate::Reg::read) this register and get [`tv::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TvSpec;
impl crate::RegisterSpec for TvSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tv::R`](R) reader structure"]
impl crate::Readable for TvSpec {}
#[doc = "`reset()` method sets TV to value 0xff"]
impl crate::Resettable for TvSpec {
    const RESET_VALUE: u32 = 0xff;
}
