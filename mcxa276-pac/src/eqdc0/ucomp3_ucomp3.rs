#[doc = "Register `UCOMP3` writer"]
pub type W = crate::W<Ucomp3Ucomp3Spec>;
#[doc = "Field `UCOMP3` writer - UCOMP3"]
pub type Ucomp3W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl W {
    #[doc = "Bits 0:15 - UCOMP3"]
    #[inline(always)]
    pub fn ucomp3(&mut self) -> Ucomp3W<Ucomp3Ucomp3Spec> {
        Ucomp3W::new(self, 0)
    }
}
#[doc = "Upper Position Compare 3\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ucomp3_ucomp3::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ucomp3Ucomp3Spec;
impl crate::RegisterSpec for Ucomp3Ucomp3Spec {
    type Ux = u16;
}
#[doc = "`write(|w| ..)` method takes [`ucomp3_ucomp3::W`](W) writer structure"]
impl crate::Writable for Ucomp3Ucomp3Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets UCOMP3 to value 0x8000"]
impl crate::Resettable for Ucomp3Ucomp3Spec {
    const RESET_VALUE: u16 = 0x8000;
}
