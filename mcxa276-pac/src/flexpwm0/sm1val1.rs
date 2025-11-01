#[doc = "Register `SM1VAL1` reader"]
pub type R = crate::R<Sm1val1Spec>;
#[doc = "Register `SM1VAL1` writer"]
pub type W = crate::W<Sm1val1Spec>;
#[doc = "Field `VAL1` reader - Value 1"]
pub type Val1R = crate::FieldReader<u16>;
#[doc = "Field `VAL1` writer - Value 1"]
pub type Val1W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Value 1"]
    #[inline(always)]
    pub fn val1(&self) -> Val1R {
        Val1R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - Value 1"]
    #[inline(always)]
    pub fn val1(&mut self) -> Val1W<Sm1val1Spec> {
        Val1W::new(self, 0)
    }
}
#[doc = "Value Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1val1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1val1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm1val1Spec;
impl crate::RegisterSpec for Sm1val1Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm1val1::R`](R) reader structure"]
impl crate::Readable for Sm1val1Spec {}
#[doc = "`write(|w| ..)` method takes [`sm1val1::W`](W) writer structure"]
impl crate::Writable for Sm1val1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM1VAL1 to value 0"]
impl crate::Resettable for Sm1val1Spec {}
