#[doc = "Register `SUB256` writer"]
pub type W = crate::W<Sub256Spec>;
#[doc = "Field `SB256` writer - Subtract 256"]
pub type Sb256W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - Subtract 256"]
    #[inline(always)]
    pub fn sb256(&mut self) -> Sb256W<Sub256Spec> {
        Sb256W::new(self, 0)
    }
}
#[doc = "SUB256 Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sub256::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sub256Spec;
impl crate::RegisterSpec for Sub256Spec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`sub256::W`](W) writer structure"]
impl crate::Writable for Sub256Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SUB256 to value 0"]
impl crate::Resettable for Sub256Spec {}
