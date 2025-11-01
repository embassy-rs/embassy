#[doc = "Register `LCOMP0` reader"]
pub type R = crate::R<Lcomp0Spec>;
#[doc = "Register `LCOMP0` writer"]
pub type W = crate::W<Lcomp0Spec>;
#[doc = "Field `LCOMP0` reader - LCOMP0"]
pub type Lcomp0R = crate::FieldReader<u16>;
#[doc = "Field `LCOMP0` writer - LCOMP0"]
pub type Lcomp0W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - LCOMP0"]
    #[inline(always)]
    pub fn lcomp0(&self) -> Lcomp0R {
        Lcomp0R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - LCOMP0"]
    #[inline(always)]
    pub fn lcomp0(&mut self) -> Lcomp0W<Lcomp0Spec> {
        Lcomp0W::new(self, 0)
    }
}
#[doc = "Lower Position Compare Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`lcomp0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcomp0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Lcomp0Spec;
impl crate::RegisterSpec for Lcomp0Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`lcomp0::R`](R) reader structure"]
impl crate::Readable for Lcomp0Spec {}
#[doc = "`write(|w| ..)` method takes [`lcomp0::W`](W) writer structure"]
impl crate::Writable for Lcomp0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LCOMP0 to value 0"]
impl crate::Resettable for Lcomp0Spec {}
