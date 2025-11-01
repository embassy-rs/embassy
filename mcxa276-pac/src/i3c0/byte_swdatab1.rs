#[doc = "Register `SWDATAB1` writer"]
pub type W = crate::W<ByteSwdatab1Spec>;
#[doc = "Field `DATA` writer - Data"]
pub type DataW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl W {
    #[doc = "Bits 0:7 - Data"]
    #[inline(always)]
    pub fn data(&mut self) -> DataW<ByteSwdatab1Spec> {
        DataW::new(self, 0)
    }
}
#[doc = "Target Write Data Byte\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`byte_swdatab1::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ByteSwdatab1Spec;
impl crate::RegisterSpec for ByteSwdatab1Spec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`byte_swdatab1::W`](W) writer structure"]
impl crate::Writable for ByteSwdatab1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SWDATAB1 to value 0"]
impl crate::Resettable for ByteSwdatab1Spec {}
