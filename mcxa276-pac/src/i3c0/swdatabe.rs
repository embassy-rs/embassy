#[doc = "Register `SWDATABE` writer"]
pub type W = crate::W<SwdatabeSpec>;
#[doc = "Field `DATA` writer - Data"]
pub type DataW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl W {
    #[doc = "Bits 0:7 - Data"]
    #[inline(always)]
    pub fn data(&mut self) -> DataW<SwdatabeSpec> {
        DataW::new(self, 0)
    }
}
#[doc = "Target Write Data Byte End\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swdatabe::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SwdatabeSpec;
impl crate::RegisterSpec for SwdatabeSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`swdatabe::W`](W) writer structure"]
impl crate::Writable for SwdatabeSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SWDATABE to value 0"]
impl crate::Resettable for SwdatabeSpec {}
