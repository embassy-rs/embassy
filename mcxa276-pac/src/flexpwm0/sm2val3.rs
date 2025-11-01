#[doc = "Register `SM2VAL3` reader"]
pub type R = crate::R<Sm2val3Spec>;
#[doc = "Register `SM2VAL3` writer"]
pub type W = crate::W<Sm2val3Spec>;
#[doc = "Field `VAL3` reader - Value 3"]
pub type Val3R = crate::FieldReader<u16>;
#[doc = "Field `VAL3` writer - Value 3"]
pub type Val3W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Value 3"]
    #[inline(always)]
    pub fn val3(&self) -> Val3R {
        Val3R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - Value 3"]
    #[inline(always)]
    pub fn val3(&mut self) -> Val3W<Sm2val3Spec> {
        Val3W::new(self, 0)
    }
}
#[doc = "Value Register 3\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2val3::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2val3::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2val3Spec;
impl crate::RegisterSpec for Sm2val3Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2val3::R`](R) reader structure"]
impl crate::Readable for Sm2val3Spec {}
#[doc = "`write(|w| ..)` method takes [`sm2val3::W`](W) writer structure"]
impl crate::Writable for Sm2val3Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM2VAL3 to value 0"]
impl crate::Resettable for Sm2val3Spec {}
