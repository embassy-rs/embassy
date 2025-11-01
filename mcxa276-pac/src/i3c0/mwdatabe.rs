#[doc = "Register `MWDATABE` writer"]
pub type W = crate::W<MwdatabeSpec>;
#[doc = "Field `VALUE` writer - Data"]
pub type ValueW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl W {
    #[doc = "Bits 0:7 - Data"]
    #[inline(always)]
    pub fn value(&mut self) -> ValueW<MwdatabeSpec> {
        ValueW::new(self, 0)
    }
}
#[doc = "Controller Write Data Byte End\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mwdatabe::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MwdatabeSpec;
impl crate::RegisterSpec for MwdatabeSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`mwdatabe::W`](W) writer structure"]
impl crate::Writable for MwdatabeSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MWDATABE to value 0"]
impl crate::Resettable for MwdatabeSpec {}
