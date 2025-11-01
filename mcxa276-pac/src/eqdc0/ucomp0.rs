#[doc = "Register `UCOMP0` reader"]
pub type R = crate::R<Ucomp0Spec>;
#[doc = "Register `UCOMP0` writer"]
pub type W = crate::W<Ucomp0Spec>;
#[doc = "Field `UCOMP0` reader - UCOMP0"]
pub type Ucomp0R = crate::FieldReader<u16>;
#[doc = "Field `UCOMP0` writer - UCOMP0"]
pub type Ucomp0W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - UCOMP0"]
    #[inline(always)]
    pub fn ucomp0(&self) -> Ucomp0R {
        Ucomp0R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - UCOMP0"]
    #[inline(always)]
    pub fn ucomp0(&mut self) -> Ucomp0W<Ucomp0Spec> {
        Ucomp0W::new(self, 0)
    }
}
#[doc = "Upper Position Compare Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`ucomp0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ucomp0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ucomp0Spec;
impl crate::RegisterSpec for Ucomp0Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`ucomp0::R`](R) reader structure"]
impl crate::Readable for Ucomp0Spec {}
#[doc = "`write(|w| ..)` method takes [`ucomp0::W`](W) writer structure"]
impl crate::Writable for Ucomp0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets UCOMP0 to value 0x8000"]
impl crate::Resettable for Ucomp0Spec {
    const RESET_VALUE: u16 = 0x8000;
}
