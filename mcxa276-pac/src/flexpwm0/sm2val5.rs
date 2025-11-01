#[doc = "Register `SM2VAL5` reader"]
pub type R = crate::R<Sm2val5Spec>;
#[doc = "Register `SM2VAL5` writer"]
pub type W = crate::W<Sm2val5Spec>;
#[doc = "Field `VAL5` reader - Value 5"]
pub type Val5R = crate::FieldReader<u16>;
#[doc = "Field `VAL5` writer - Value 5"]
pub type Val5W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Value 5"]
    #[inline(always)]
    pub fn val5(&self) -> Val5R {
        Val5R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - Value 5"]
    #[inline(always)]
    pub fn val5(&mut self) -> Val5W<Sm2val5Spec> {
        Val5W::new(self, 0)
    }
}
#[doc = "Value Register 5\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2val5::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2val5::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2val5Spec;
impl crate::RegisterSpec for Sm2val5Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2val5::R`](R) reader structure"]
impl crate::Readable for Sm2val5Spec {}
#[doc = "`write(|w| ..)` method takes [`sm2val5::W`](W) writer structure"]
impl crate::Writable for Sm2val5Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM2VAL5 to value 0"]
impl crate::Resettable for Sm2val5Spec {}
