#[doc = "Register `ASSERT16` writer"]
pub type W = crate::W<Assert16Spec>;
#[doc = "Field `AST16` writer - ASSERT16 Command"]
pub type Ast16W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - ASSERT16 Command"]
    #[inline(always)]
    pub fn ast16(&mut self) -> Ast16W<Assert16Spec> {
        Ast16W::new(self, 0)
    }
}
#[doc = "ASSERT16 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`assert16::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Assert16Spec;
impl crate::RegisterSpec for Assert16Spec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`assert16::W`](W) writer structure"]
impl crate::Writable for Assert16Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ASSERT16 to value 0"]
impl crate::Resettable for Assert16Spec {}
