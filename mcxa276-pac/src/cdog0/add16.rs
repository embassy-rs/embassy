#[doc = "Register `ADD16` writer"]
pub type W = crate::W<Add16Spec>;
#[doc = "Field `AD16` writer - ADD 16"]
pub type Ad16W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - ADD 16"]
    #[inline(always)]
    pub fn ad16(&mut self) -> Ad16W<Add16Spec> {
        Ad16W::new(self, 0)
    }
}
#[doc = "ADD16 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`add16::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Add16Spec;
impl crate::RegisterSpec for Add16Spec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`add16::W`](W) writer structure"]
impl crate::Writable for Add16Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ADD16 to value 0"]
impl crate::Resettable for Add16Spec {}
