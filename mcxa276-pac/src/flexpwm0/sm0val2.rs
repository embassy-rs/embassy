#[doc = "Register `SM0VAL2` reader"]
pub type R = crate::R<Sm0val2Spec>;
#[doc = "Register `SM0VAL2` writer"]
pub type W = crate::W<Sm0val2Spec>;
#[doc = "Field `VAL2` reader - Value 2"]
pub type Val2R = crate::FieldReader<u16>;
#[doc = "Field `VAL2` writer - Value 2"]
pub type Val2W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Value 2"]
    #[inline(always)]
    pub fn val2(&self) -> Val2R {
        Val2R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - Value 2"]
    #[inline(always)]
    pub fn val2(&mut self) -> Val2W<Sm0val2Spec> {
        Val2W::new(self, 0)
    }
}
#[doc = "Value Register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0val2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0val2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm0val2Spec;
impl crate::RegisterSpec for Sm0val2Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm0val2::R`](R) reader structure"]
impl crate::Readable for Sm0val2Spec {}
#[doc = "`write(|w| ..)` method takes [`sm0val2::W`](W) writer structure"]
impl crate::Writable for Sm0val2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM0VAL2 to value 0"]
impl crate::Resettable for Sm0val2Spec {}
