#[doc = "Register `STDR` writer"]
pub type W = crate::W<StdrSpec>;
#[doc = "Field `DATA` writer - Transmit Data"]
pub type DataW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl W {
    #[doc = "Bits 0:7 - Transmit Data"]
    #[inline(always)]
    pub fn data(&mut self) -> DataW<StdrSpec> {
        DataW::new(self, 0)
    }
}
#[doc = "Target Transmit Data\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stdr::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StdrSpec;
impl crate::RegisterSpec for StdrSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`stdr::W`](W) writer structure"]
impl crate::Writable for StdrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets STDR to value 0"]
impl crate::Resettable for StdrSpec {}
