#[doc = "Register `LCOMP3` writer"]
pub type W = crate::W<Lcomp3Lcomp3Spec>;
#[doc = "Field `LCOMP3` writer - LCOMP3"]
pub type Lcomp3W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl W {
    #[doc = "Bits 0:15 - LCOMP3"]
    #[inline(always)]
    pub fn lcomp3(&mut self) -> Lcomp3W<Lcomp3Lcomp3Spec> {
        Lcomp3W::new(self, 0)
    }
}
#[doc = "Lower Position Compare 3\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcomp3_lcomp3::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Lcomp3Lcomp3Spec;
impl crate::RegisterSpec for Lcomp3Lcomp3Spec {
    type Ux = u16;
}
#[doc = "`write(|w| ..)` method takes [`lcomp3_lcomp3::W`](W) writer structure"]
impl crate::Writable for Lcomp3Lcomp3Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LCOMP3 to value 0"]
impl crate::Resettable for Lcomp3Lcomp3Spec {}
