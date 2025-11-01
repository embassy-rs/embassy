#[doc = "Register `UCOMP2` writer"]
pub type W = crate::W<Ucomp2Ucomp2Spec>;
#[doc = "Field `UCOMP2` writer - UCOMP2"]
pub type Ucomp2W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl W {
    #[doc = "Bits 0:15 - UCOMP2"]
    #[inline(always)]
    pub fn ucomp2(&mut self) -> Ucomp2W<Ucomp2Ucomp2Spec> {
        Ucomp2W::new(self, 0)
    }
}
#[doc = "Upper Position Compare 2\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ucomp2_ucomp2::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ucomp2Ucomp2Spec;
impl crate::RegisterSpec for Ucomp2Ucomp2Spec {
    type Ux = u16;
}
#[doc = "`write(|w| ..)` method takes [`ucomp2_ucomp2::W`](W) writer structure"]
impl crate::Writable for Ucomp2Ucomp2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets UCOMP2 to value 0x8000"]
impl crate::Resettable for Ucomp2Ucomp2Spec {
    const RESET_VALUE: u16 = 0x8000;
}
