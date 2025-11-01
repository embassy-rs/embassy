#[doc = "Register `RESTART` writer"]
pub type W = crate::W<RestartSpec>;
#[doc = "Field `RSTRT` writer - Restart command"]
pub type RstrtW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - Restart command"]
    #[inline(always)]
    pub fn rstrt(&mut self) -> RstrtW<RestartSpec> {
        RstrtW::new(self, 0)
    }
}
#[doc = "RESTART Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`restart::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RestartSpec;
impl crate::RegisterSpec for RestartSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`restart::W`](W) writer structure"]
impl crate::Writable for RestartSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RESTART to value 0"]
impl crate::Resettable for RestartSpec {}
