#[doc = "Register `LCOMP2` writer"]
pub type W = crate::W<Lcomp2Lcomp2Spec>;
#[doc = "Field `LCOMP2` writer - LCOMP2"]
pub type Lcomp2W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl W {
    #[doc = "Bits 0:15 - LCOMP2"]
    #[inline(always)]
    pub fn lcomp2(&mut self) -> Lcomp2W<Lcomp2Lcomp2Spec> {
        Lcomp2W::new(self, 0)
    }
}
#[doc = "Lower Position Compare 2\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcomp2_lcomp2::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Lcomp2Lcomp2Spec;
impl crate::RegisterSpec for Lcomp2Lcomp2Spec {
    type Ux = u16;
}
#[doc = "`write(|w| ..)` method takes [`lcomp2_lcomp2::W`](W) writer structure"]
impl crate::Writable for Lcomp2Lcomp2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LCOMP2 to value 0"]
impl crate::Resettable for Lcomp2Lcomp2Spec {}
