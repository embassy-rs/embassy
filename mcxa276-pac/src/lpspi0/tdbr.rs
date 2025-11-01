#[doc = "Register `TDBR[%s]` writer"]
pub type W = crate::W<TdbrSpec>;
#[doc = "Field `DATA` writer - Data"]
pub type DataW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - Data"]
    #[inline(always)]
    pub fn data(&mut self) -> DataW<TdbrSpec> {
        DataW::new(self, 0)
    }
}
#[doc = "Transmit Data Burst\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tdbr::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TdbrSpec;
impl crate::RegisterSpec for TdbrSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`tdbr::W`](W) writer structure"]
impl crate::Writable for TdbrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TDBR[%s] to value 0"]
impl crate::Resettable for TdbrSpec {}
