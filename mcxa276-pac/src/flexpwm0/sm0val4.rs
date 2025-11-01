#[doc = "Register `SM0VAL4` reader"]
pub type R = crate::R<Sm0val4Spec>;
#[doc = "Register `SM0VAL4` writer"]
pub type W = crate::W<Sm0val4Spec>;
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
    pub fn val4(&mut self) -> Val4W<Sm0val4Spec> {
        Val4W::new(self, 0)
    }
}
#[doc = "Value Register 4\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0val4::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0val4::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm0val4Spec;
impl crate::RegisterSpec for Sm0val4Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm0val4::R`](R) reader structure"]
impl crate::Readable for Sm0val4Spec {}
#[doc = "`write(|w| ..)` method takes [`sm0val4::W`](W) writer structure"]
impl crate::Writable for Sm0val4Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM0VAL4 to value 0"]
impl crate::Resettable for Sm0val4Spec {}
