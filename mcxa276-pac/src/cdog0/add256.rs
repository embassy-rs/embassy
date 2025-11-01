#[doc = "Register `ADD256` writer"]
pub type W = crate::W<Add256Spec>;
#[doc = "Field `AD256` writer - ADD 256"]
pub type Ad256W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - ADD 256"]
    #[inline(always)]
    pub fn ad256(&mut self) -> Ad256W<Add256Spec> {
        Ad256W::new(self, 0)
    }
}
#[doc = "ADD256 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`add256::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Add256Spec;
impl crate::RegisterSpec for Add256Spec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`add256::W`](W) writer structure"]
impl crate::Writable for Add256Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ADD256 to value 0"]
impl crate::Resettable for Add256Spec {}
