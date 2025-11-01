#[doc = "Register `STOP` writer"]
pub type W = crate::W<StopSpec>;
#[doc = "Field `STP` writer - Stop command"]
pub type StpW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - Stop command"]
    #[inline(always)]
    pub fn stp(&mut self) -> StpW<StopSpec> {
        StpW::new(self, 0)
    }
}
#[doc = "STOP Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stop::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StopSpec;
impl crate::RegisterSpec for StopSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`stop::W`](W) writer structure"]
impl crate::Writable for StopSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets STOP to value 0"]
impl crate::Resettable for StopSpec {}
