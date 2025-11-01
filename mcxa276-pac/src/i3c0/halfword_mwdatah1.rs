#[doc = "Register `MWDATAH1` writer"]
pub type W = crate::W<HalfwordMwdatah1Spec>;
#[doc = "Field `VALUE` writer - Value"]
pub type ValueW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl W {
    #[doc = "Bits 0:15 - Value"]
    #[inline(always)]
    pub fn value(&mut self) -> ValueW<HalfwordMwdatah1Spec> {
        ValueW::new(self, 0)
    }
}
#[doc = "Controller Write Halfword Data (to Bus)\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`halfword_mwdatah1::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct HalfwordMwdatah1Spec;
impl crate::RegisterSpec for HalfwordMwdatah1Spec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`halfword_mwdatah1::W`](W) writer structure"]
impl crate::Writable for HalfwordMwdatah1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MWDATAH1 to value 0"]
impl crate::Resettable for HalfwordMwdatah1Spec {}
