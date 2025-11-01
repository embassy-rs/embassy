#[doc = "Register `UCOMP1` writer"]
pub type W = crate::W<Ucomp1Ucomp1Spec>;
#[doc = "Field `UCOMP1` writer - UCOMP1"]
pub type Ucomp1W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl W {
    #[doc = "Bits 0:15 - UCOMP1"]
    #[inline(always)]
    pub fn ucomp1(&mut self) -> Ucomp1W<Ucomp1Ucomp1Spec> {
        Ucomp1W::new(self, 0)
    }
}
#[doc = "Upper Position Compare 1\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ucomp1_ucomp1::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ucomp1Ucomp1Spec;
impl crate::RegisterSpec for Ucomp1Ucomp1Spec {
    type Ux = u16;
}
#[doc = "`write(|w| ..)` method takes [`ucomp1_ucomp1::W`](W) writer structure"]
impl crate::Writable for Ucomp1Ucomp1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets UCOMP1 to value 0x8000"]
impl crate::Resettable for Ucomp1Ucomp1Spec {
    const RESET_VALUE: u16 = 0x8000;
}
