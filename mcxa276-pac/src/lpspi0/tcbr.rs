#[doc = "Register `TCBR` writer"]
pub type W = crate::W<TcbrSpec>;
#[doc = "Field `DATA` writer - Command Data"]
pub type DataW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - Command Data"]
    #[inline(always)]
    pub fn data(&mut self) -> DataW<TcbrSpec> {
        DataW::new(self, 0)
    }
}
#[doc = "Transmit Command Burst\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcbr::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TcbrSpec;
impl crate::RegisterSpec for TcbrSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`tcbr::W`](W) writer structure"]
impl crate::Writable for TcbrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCBR to value 0"]
impl crate::Resettable for TcbrSpec {}
