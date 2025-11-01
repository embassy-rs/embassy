#[doc = "Register `SM0DTCNT0` reader"]
pub type R = crate::R<Sm0dtcnt0Spec>;
#[doc = "Register `SM0DTCNT0` writer"]
pub type W = crate::W<Sm0dtcnt0Spec>;
#[doc = "Field `DTCNT0` reader - Deadtime Count Register 0"]
pub type Dtcnt0R = crate::FieldReader<u16>;
#[doc = "Field `DTCNT0` writer - Deadtime Count Register 0"]
pub type Dtcnt0W<'a, REG> = crate::FieldWriter<'a, REG, 11, u16>;
impl R {
    #[doc = "Bits 0:10 - Deadtime Count Register 0"]
    #[inline(always)]
    pub fn dtcnt0(&self) -> Dtcnt0R {
        Dtcnt0R::new(self.bits & 0x07ff)
    }
}
impl W {
    #[doc = "Bits 0:10 - Deadtime Count Register 0"]
    #[inline(always)]
    pub fn dtcnt0(&mut self) -> Dtcnt0W<Sm0dtcnt0Spec> {
        Dtcnt0W::new(self, 0)
    }
}
#[doc = "Deadtime Count Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0dtcnt0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0dtcnt0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm0dtcnt0Spec;
impl crate::RegisterSpec for Sm0dtcnt0Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm0dtcnt0::R`](R) reader structure"]
impl crate::Readable for Sm0dtcnt0Spec {}
#[doc = "`write(|w| ..)` method takes [`sm0dtcnt0::W`](W) writer structure"]
impl crate::Writable for Sm0dtcnt0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM0DTCNT0 to value 0x07ff"]
impl crate::Resettable for Sm0dtcnt0Spec {
    const RESET_VALUE: u16 = 0x07ff;
}
