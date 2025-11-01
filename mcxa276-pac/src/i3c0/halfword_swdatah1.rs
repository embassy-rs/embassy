#[doc = "Register `SWDATAH1` writer"]
pub type W = crate::W<HalfwordSwdatah1Spec>;
#[doc = "Field `DATA` writer - Data"]
pub type DataW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl W {
    #[doc = "Bits 0:15 - Data"]
    #[inline(always)]
    pub fn data(&mut self) -> DataW<HalfwordSwdatah1Spec> {
        DataW::new(self, 0)
    }
}
#[doc = "Target Write Data Halfword\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`halfword_swdatah1::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct HalfwordSwdatah1Spec;
impl crate::RegisterSpec for HalfwordSwdatah1Spec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`halfword_swdatah1::W`](W) writer structure"]
impl crate::Writable for HalfwordSwdatah1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SWDATAH1 to value 0"]
impl crate::Resettable for HalfwordSwdatah1Spec {}
