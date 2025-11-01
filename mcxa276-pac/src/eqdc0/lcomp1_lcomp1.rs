#[doc = "Register `LCOMP1` writer"]
pub type W = crate::W<Lcomp1Lcomp1Spec>;
#[doc = "Field `LCOMP1` writer - LCOMP1"]
pub type Lcomp1W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl W {
    #[doc = "Bits 0:15 - LCOMP1"]
    #[inline(always)]
    pub fn lcomp1(&mut self) -> Lcomp1W<Lcomp1Lcomp1Spec> {
        Lcomp1W::new(self, 0)
    }
}
#[doc = "Lower Position Compare 1\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcomp1_lcomp1::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Lcomp1Lcomp1Spec;
impl crate::RegisterSpec for Lcomp1Lcomp1Spec {
    type Ux = u16;
}
#[doc = "`write(|w| ..)` method takes [`lcomp1_lcomp1::W`](W) writer structure"]
impl crate::Writable for Lcomp1Lcomp1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LCOMP1 to value 0"]
impl crate::Resettable for Lcomp1Lcomp1Spec {}
