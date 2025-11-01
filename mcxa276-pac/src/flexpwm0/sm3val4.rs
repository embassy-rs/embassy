#[doc = "Register `SM3VAL4` reader"]
pub type R = crate::R<Sm3val4Spec>;
#[doc = "Register `SM3VAL4` writer"]
pub type W = crate::W<Sm3val4Spec>;
#[doc = "Field `VAL4` reader - Value 4"]
pub type Val4R = crate::FieldReader<u16>;
#[doc = "Field `VAL4` writer - Value 4"]
pub type Val4W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Value 4"]
    #[inline(always)]
    pub fn val4(&self) -> Val4R {
        Val4R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - Value 4"]
    #[inline(always)]
    pub fn val4(&mut self) -> Val4W<Sm3val4Spec> {
        Val4W::new(self, 0)
    }
}
#[doc = "Value Register 4\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3val4::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3val4::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm3val4Spec;
impl crate::RegisterSpec for Sm3val4Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm3val4::R`](R) reader structure"]
impl crate::Readable for Sm3val4Spec {}
#[doc = "`write(|w| ..)` method takes [`sm3val4::W`](W) writer structure"]
impl crate::Writable for Sm3val4Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM3VAL4 to value 0"]
impl crate::Resettable for Sm3val4Spec {}
