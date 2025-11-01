#[doc = "Register `SM2VAL0` reader"]
pub type R = crate::R<Sm2val0Spec>;
#[doc = "Register `SM2VAL0` writer"]
pub type W = crate::W<Sm2val0Spec>;
#[doc = "Field `VAL0` reader - Value 0"]
pub type Val0R = crate::FieldReader<u16>;
#[doc = "Field `VAL0` writer - Value 0"]
pub type Val0W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Value 0"]
    #[inline(always)]
    pub fn val0(&self) -> Val0R {
        Val0R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - Value 0"]
    #[inline(always)]
    pub fn val0(&mut self) -> Val0W<Sm2val0Spec> {
        Val0W::new(self, 0)
    }
}
#[doc = "Value Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2val0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2val0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2val0Spec;
impl crate::RegisterSpec for Sm2val0Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2val0::R`](R) reader structure"]
impl crate::Readable for Sm2val0Spec {}
#[doc = "`write(|w| ..)` method takes [`sm2val0::W`](W) writer structure"]
impl crate::Writable for Sm2val0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM2VAL0 to value 0"]
impl crate::Resettable for Sm2val0Spec {}
