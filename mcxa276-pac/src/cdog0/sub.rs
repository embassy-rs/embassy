#[doc = "Register `SUB` writer"]
pub type W = crate::W<SubSpec>;
#[doc = "Field `SB` writer - Subtract Write Value"]
pub type SbW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - Subtract Write Value"]
    #[inline(always)]
    pub fn sb(&mut self) -> SbW<SubSpec> {
        SbW::new(self, 0)
    }
}
#[doc = "SUB Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sub::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SubSpec;
impl crate::RegisterSpec for SubSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`sub::W`](W) writer structure"]
impl crate::Writable for SubSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SUB to value 0"]
impl crate::Resettable for SubSpec {}
