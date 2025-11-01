#[doc = "Register `SUB1` writer"]
pub type W = crate::W<Sub1Spec>;
#[doc = "Field `SB1` writer - Subtract 1"]
pub type Sb1W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - Subtract 1"]
    #[inline(always)]
    pub fn sb1(&mut self) -> Sb1W<Sub1Spec> {
        Sb1W::new(self, 0)
    }
}
#[doc = "SUB1 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sub1::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sub1Spec;
impl crate::RegisterSpec for Sub1Spec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`sub1::W`](W) writer structure"]
impl crate::Writable for Sub1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SUB1 to value 0"]
impl crate::Resettable for Sub1Spec {}
