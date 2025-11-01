#[doc = "Register `SM2DTCNT1` reader"]
pub type R = crate::R<Sm2dtcnt1Spec>;
#[doc = "Register `SM2DTCNT1` writer"]
pub type W = crate::W<Sm2dtcnt1Spec>;
#[doc = "Field `DTCNT1` reader - Deadtime Count Register 1"]
pub type Dtcnt1R = crate::FieldReader<u16>;
#[doc = "Field `DTCNT1` writer - Deadtime Count Register 1"]
pub type Dtcnt1W<'a, REG> = crate::FieldWriter<'a, REG, 11, u16>;
impl R {
    #[doc = "Bits 0:10 - Deadtime Count Register 1"]
    #[inline(always)]
    pub fn dtcnt1(&self) -> Dtcnt1R {
        Dtcnt1R::new(self.bits & 0x07ff)
    }
}
impl W {
    #[doc = "Bits 0:10 - Deadtime Count Register 1"]
    #[inline(always)]
    pub fn dtcnt1(&mut self) -> Dtcnt1W<Sm2dtcnt1Spec> {
        Dtcnt1W::new(self, 0)
    }
}
#[doc = "Deadtime Count Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2dtcnt1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2dtcnt1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2dtcnt1Spec;
impl crate::RegisterSpec for Sm2dtcnt1Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2dtcnt1::R`](R) reader structure"]
impl crate::Readable for Sm2dtcnt1Spec {}
#[doc = "`write(|w| ..)` method takes [`sm2dtcnt1::W`](W) writer structure"]
impl crate::Writable for Sm2dtcnt1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM2DTCNT1 to value 0x07ff"]
impl crate::Resettable for Sm2dtcnt1Spec {
    const RESET_VALUE: u16 = 0x07ff;
}
