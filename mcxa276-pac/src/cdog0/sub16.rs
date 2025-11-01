#[doc = "Register `SUB16` writer"]
pub type W = crate::W<Sub16Spec>;
#[doc = "Field `SB16` writer - Subtract 16"]
pub type Sb16W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - Subtract 16"]
    #[inline(always)]
    pub fn sb16(&mut self) -> Sb16W<Sub16Spec> {
        Sb16W::new(self, 0)
    }
}
#[doc = "SUB16 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sub16::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sub16Spec;
impl crate::RegisterSpec for Sub16Spec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`sub16::W`](W) writer structure"]
impl crate::Writable for Sub16Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SUB16 to value 0"]
impl crate::Resettable for Sub16Spec {}
